use crate::objects::Value;
use crate::vm::gc::GarbageCollector;
use crate::vm::interpreter::heap_types::HeapValue;

/// Layout of a JIT frame passed to compiled native code.
///
/// The JIT-generated code accesses fields via `rbp`-relative offsets
/// after the prologue pushes `rbp` and saves callee-saved registers.
///
/// # Stack frame layout (after prologue)
///
/// ```text
/// [rbp+0x28] → saved r15  (unused, for alignment)
/// [rbp+0x20] → saved r14  (gc_ptr)
/// [rbp+0x18] → saved r13  (heap_ptr)
/// [rbp+0x10] → saved r12  (stack_base)
/// [rbp+0x08] → saved rbx  (module_ptr)
/// [rbp+0x00] → saved rbp  (caller's frame pointer)
/// ```
///
/// # C ABI
///
/// The JIT entry point has the signature:
/// ```text
/// extern "C" fn(frame: *mut JitFrame) -> Value
/// ```
///
/// The first argument is passed in `rdi`.  The trampoline stores it
/// into the JitFrame's `self_ptr` field and sets up `rbp` to point
/// at the saved register area.
#[repr(C)]
pub struct JitFrame {
    /// Pointer to the `Vec<Value>` data buffer (stack_base).
    pub stack_base: *mut Value,
    /// Current stack length (number of valid elements).
    pub stack_len: usize,
    /// Pointer to the `Vec<HeapValue>` heap.
    pub heap_ptr: *mut Vec<HeapValue>,
    /// Pointer to the garbage collector.
    pub gc_ptr: *mut GarbageCollector,
    /// Base pointer index (offset into stack for locals).
    pub base_pointer: usize,
    /// PC of the instruction following the JIT-compiled region.
    pub return_pc: usize,
    /// Back-pointer to the JitFrame itself (for self-referential
    /// prologue access).
    pub self_ptr: *mut JitFrame,
}

// Register assignments (x86-64 SysV ABI):
//
//   rbp  = frame pointer (points to saved regs area)
//   rbx  = module_ptr  (saved callee-register)
//   r12  = stack_base  (saved callee-register)
//   r13  = heap_ptr    (saved callee-register)
//   r14  = gc_ptr      (saved callee-register)
//   rax  = scratch / return value
//   rdi  = first argument (JitFrame pointer on entry)
//
// Offsets from rbp (after prologue):
pub const OFFSET_RBP: i32 = 0;
pub const OFFSET_RBX: i32 = 8;
pub const OFFSET_R12: i32 = 16;
pub const OFFSET_R13: i32 = 24;
pub const OFFSET_R14: i32 = 32;

// JitFrame field offsets from the self_ptr (rdi on entry):
pub const FRAME_STACK_BASE_OFFSET: i32 = 0;
pub const FRAME_STACK_LEN_OFFSET: i32 = 8;
pub const FRAME_HEAP_PTR_OFFSET: i32 = 16;
pub const FRAME_GC_PTR_OFFSET: i32 = 24;
pub const FRAME_BASE_POINTER_OFFSET: i32 = 32;
pub const FRAME_RETURN_PC_OFFSET: i32 = 40;
pub const FRAME_SELF_PTR_OFFSET: i32 = 48;

/// Size of a single `Value` on the stack (in bytes).
pub const VALUE_SIZE: i32 = std::mem::size_of::<Value>() as i32;
