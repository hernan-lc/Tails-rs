/// Fixed register assignments for JIT-compiled code.
///
/// These are callee-saved registers (except rax/rdi which are
/// caller-saved and used as scratch/argument registers).
pub use super::{R12, R13, R14, RAX, RBP, RBX, RDI, RSI};

/// Frame pointer.
pub const FP: u8 = RBP;

/// Module pointer (callee-saved).
pub const REG_MODULE: u8 = RBX;

/// Stack base pointer — `Vec<Value>` data pointer (callee-saved).
pub const REG_STACK: u8 = R12;

/// Heap pointer — `Vec<HeapValue>` (callee-saved).
pub const REG_HEAP: u8 = R13;

/// GC pointer (callee-saved).
pub const REG_GC: u8 = R14;

/// General-purpose scratch register.
pub const REG_SCRATCH: u8 = RAX;

/// First argument register (used for JitFrame* on entry).
pub const REG_ARG0: u8 = RDI;
