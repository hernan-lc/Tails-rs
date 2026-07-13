use crate::errors::Result;
use crate::objects::Value;
use crate::vm::interpreter::Interpreter;

pub(super) fn native_set_immediate(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let callback = args.first().cloned().unwrap_or(Value::Undefined);
    let id = interp.async_runtime.enqueue_macrotask(callback, 0.0);
    Ok(Value::Float(id as f64))
}

pub(super) fn native_clear_immediate(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    if let Some(Value::Float(id)) = args.first() {
        interp.async_runtime.cancel_timer(*id as u32);
    }
    Ok(Value::Undefined)
}

pub(super) fn native_queue_microtask(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let callback = args.first().cloned().unwrap_or(Value::Undefined);
    interp.async_runtime.enqueue_microtask(callback);
    Ok(Value::Undefined)
}
