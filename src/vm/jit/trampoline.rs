use crate::objects::Value;
use crate::vm::interpreter::Interpreter;

use super::frame::JitFrame;

/// Set up a `JitFrame` from the interpreter state and call the
/// JIT-compiled native code.
///
/// # Safety
///
/// The caller must ensure that `entry` points to valid x86-64 code
/// that was compiled by our JIT and that `interp` is in a consistent
/// state.
pub unsafe fn call_jit(
    interp: &mut Interpreter,
    entry: extern "C" fn(*mut JitFrame) -> i64,
) -> Value {
    let stack_base = interp.stack.as_mut_ptr();
    let stack_len = interp.stack.len();
    let heap_ptr = &mut interp.heap as *mut _;
    let gc_ptr = &mut interp.gc as *mut _;

    let mut frame = JitFrame {
        stack_base,
        stack_len,
        heap_ptr,
        gc_ptr,
        base_pointer: interp
            .call_stack
            .last()
            .map(|f| f.base_pointer)
            .unwrap_or(0),
        return_pc: 0,
        self_ptr: std::ptr::null_mut(),
    };
    frame.self_ptr = &mut frame as *mut JitFrame;

    // Call the native code. The return value is the raw stack offset
    // of the result Value.
    let _result_offset = entry(&mut frame);

    // Sync stack length back.
    let new_len = frame.stack_len;
    if new_len <= interp.stack.len() {
        interp.stack.truncate(new_len);
    } else {
        interp.stack.resize_with(new_len, || Value::Undefined);
    }

    // Pop the result from the JIT stack.
    if new_len > 0 {
        interp.stack.pop().unwrap_or(Value::Undefined)
    } else {
        Value::Undefined
    }
}
