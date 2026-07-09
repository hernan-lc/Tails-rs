use crate::compiler::CompiledModule;
use crate::objects::Value;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub(crate) struct CallFrame {
    pub(crate) return_address: usize,
    pub(crate) base_pointer: usize,
    pub(crate) closure_var_count: usize,
    pub(crate) func_heap_idx: Option<usize>,
    pub(crate) this_value: Option<Value>,
    pub(crate) is_construct: bool,
    pub(crate) source_name: Option<String>,
    pub(crate) generator_heap_idx: Option<usize>,
    pub(crate) source_line: Option<usize>,
    pub(crate) source_col: Option<usize>,
    pub(crate) exception_handlers_snapshot: Vec<ExceptionHandler>,
    /// Non-arrow functions expose the `arguments` object (array-like).
    pub(crate) arguments: Option<Value>,
}

#[derive(Debug, Clone)]
pub(crate) struct ExceptionHandler {
    pub(crate) catch_pc: u32,
    pub(crate) finally_pc: u32,
    pub(crate) stack_depth: usize,
}

#[derive(Clone)]
pub(crate) struct SuspendedFrame {
    pub(crate) promise_idx: usize,
    pub(crate) resume_pc: usize,
    pub(crate) stack_snapshot: Vec<Value>,
    pub(crate) call_stack_snapshot: Vec<CallFrame>,
    pub(crate) module: Option<Rc<CompiledModule>>,
    pub(crate) module_path: Option<String>,
    pub(crate) exception_handlers_snapshot: Vec<ExceptionHandler>,
    pub(crate) block_scope_stack_snapshot: Vec<usize>,
}
