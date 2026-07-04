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
                        closure: Rc::new(RefCell::new(Vec::new())),
                        prototype: Some(proto_obj_idx),
                        super_class: None,
                        properties: PropertyStorage::new(),
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
                let base_pointer: usize =
                    self.call_stack.last().map(|f| f.base_pointer).unwrap_or(0);
                let snapshot: Vec<Value> = _capture_slots
                    .iter()
                    .map(|slot| {
                        let abs_slot = base_pointer + *slot as usize;
                        self.stack
                            .get(abs_slot)
                            .cloned()
                            .unwrap_or(Value::Undefined)
                    })
                    .collect();
                let closure_vars: Rc<RefCell<Vec<Value>>> = {
                    let cached: Option<Rc<RefCell<Vec<Value>>>> =
                        if let Some(frame) = self.call_stack.last() {
                            frame.shared_closure_env.get(func_idx).cloned()
                        } else {
                            None
                        };
                    match cached {
                        Some(shared) => shared,
                        None => {
                            let rc = Rc::new(RefCell::new(snapshot));
                            if let Some(frame) = self.call_stack.last_mut() {
                                frame.shared_closure_env.insert(*func_idx, rc.clone());
                            }
                            rc
                        }
                    }
                };
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
                        properties: PropertyStorage::new(),
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
