use super::*;
use crate::compiler::CompiledModule;
use crate::compiler::Instruction;
use crate::errors::Result;
use crate::objects::Value;
use rustc_hash::FxHashMap;

impl Interpreter {
    fn module_globals_rc(&self) -> std::rc::Rc<FxHashMap<String, Value>> {
        self.module_globals_rc
            .clone()
            .unwrap_or_else(|| std::rc::Rc::new(self.globals.clone()))
    }

    pub(crate) fn exec_make_function(
        &mut self,
        instruction: &Instruction,
        module: &CompiledModule,
        _pc: usize,
    ) -> Result<bool> {
        match instruction {
            Instruction::MakeFunction(func_idx) => {
                let func_info = module.functions[*func_idx as usize].clone();
                let proto_obj_idx = self
                    .gc
                    .allocate(&mut self.heap, HeapValue::Object(JsObject::new()));
                let owner = self.current_module.clone();
                let scope = self.module_globals_rc();
                let src_file = self.current_module_path.clone();
                let src_line = func_info.source_line;
                let heap_idx = self.gc.allocate(
                    &mut self.heap,
                    HeapValue::Function(JsFunction {
                        name: func_info.name,
                        params: func_info.params,
                        rest_param: func_info.rest_param,
                        bytecode_index: func_info.bytecode_index,
                        closure: Vec::new(),
                        prototype: Some(proto_obj_idx),
                        super_class: None,
                        properties: FxHashMap::default(),
                        owner_module: owner,
                        module_scope: Some(scope),
                        is_generator: func_info.is_generator,
                        source_file: src_file,
                        source_line: src_line,
                        is_arrow: func_info.is_arrow,
                        captured_this: if func_info.is_arrow {
                            self.call_stack.last().and_then(|f| f.this_value.clone())
                        } else {
                            None
                        },
                    }),
                );
                self.stack.push(Value::Function(heap_idx));
            }
            Instruction::MakeClosure(func_idx, _capture_slots) => {
                let func_info = module.functions[*func_idx as usize].clone();
                let mut closure_vars = Vec::new();
                let base = self.call_stack.last().map(|f| f.base_pointer).unwrap_or(0);
                for slot in _capture_slots {
                    let abs_slot = base + *slot as usize;
                    let value = self
                        .stack
                        .get(abs_slot)
                        .cloned()
                        .unwrap_or(Value::Undefined);
                    closure_vars.push(value);
                }
                let proto_obj_idx = self
                    .gc
                    .allocate(&mut self.heap, HeapValue::Object(JsObject::new()));
                let owner = self.current_module.clone();
                let scope = self.module_globals_rc();
                let src_file = self.current_module_path.clone();
                let src_line = func_info.source_line;
                let heap_idx = self.gc.allocate(
                    &mut self.heap,
                    HeapValue::Function(JsFunction {
                        name: func_info.name,
                        params: func_info.params,
                        rest_param: func_info.rest_param,
                        bytecode_index: func_info.bytecode_index,
                        closure: closure_vars,
                        prototype: Some(proto_obj_idx),
                        super_class: None,
                        properties: FxHashMap::default(),
                        owner_module: owner,
                        module_scope: Some(scope),
                        is_generator: func_info.is_generator,
                        source_file: src_file,
                        source_line: src_line,
                        is_arrow: func_info.is_arrow,
                        captured_this: if func_info.is_arrow {
                            self.call_stack.last().and_then(|f| f.this_value.clone())
                        } else {
                            None
                        },
                    }),
                );
                self.stack.push(Value::Function(heap_idx));
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}
