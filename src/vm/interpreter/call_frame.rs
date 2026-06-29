use crate::objects::Value;

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
}

#[derive(Debug, Clone)]
pub(crate) struct ExceptionHandler {
    pub(crate) catch_pc: u32,
    pub(crate) finally_pc: u32,
    pub(crate) stack_depth: usize,
}
