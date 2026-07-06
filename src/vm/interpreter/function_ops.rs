use super::*;
use crate::compiler::CompiledModule;
use crate::compiler::Instruction;
use crate::errors::Result;
use crate::objects::Value;
use rustc_hash::FxHashMap;

impl Interpreter {
    fn module_globals_rc(&mut self) -> std::rc::Rc<std::cell::RefCell<FxHashMap<String, Value>>> {
        self.module_globals_rc
            .get_or_insert_with(|| {
                if let Some(ref mg) = self.module_globals {
                    mg.clone()
                } else {
                    std::rc::Rc::new(std::cell::RefCell::new(self.globals.clone()))
                }
            })
            .clone()
    }

    pub(crate) fn exec_make_function(
        &mut self,
        instruction: &Instruction,
        module: &CompiledModule,
        _pc: usize,
    ) -> Result<bool> {
        match instruction {
            Instruction::MakeFunction(func_idx) => {
                let fi = &module.functions[*func_idx as usize];
                let owner = self.current_module.clone();
                let scope = self.module_globals_rc();
                let heap_idx = self.gc.allocate(
                    &mut self.heap,
                    HeapValue::Function(JsFunction {
                        name: fi.name.clone(),
                        params: fi.params.clone(),
                        rest_param: fi.rest_param.clone(),
                        bytecode_index: fi.bytecode_index,
                        closure: Rc::new(RefCell::new(Vec::new())),
                        prototype: None,
                        super_class: None,
                        properties: PropertyStorage::new(),
                        owner_module: owner,
                        module_scope: Some(scope),
                        is_generator: fi.is_generator,
                        source_file: self.current_module_path.clone(),
                        source_line: fi.source_line,
                        is_arrow: fi.is_arrow,
                        captured_this: if fi.is_arrow {
                            self.call_stack.last().and_then(|f| f.this_value.clone())
                        } else {
                            None
                        },
                    }),
                );
                self.stack.push(Value::Function(heap_idx));
            }
            Instruction::MakeClosure(func_idx, _capture_slots) => {
                let fi = &module.functions[*func_idx as usize];
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
                    // Each closure gets its own copy of the captured values.
                    // The shared_closure_env optimization was causing closures
                    // created in a loop body to share the same environment,
                    // meaning all closures captured only the first iteration's
                    // values instead of their respective per-iteration values.
                    let rc = Rc::new(RefCell::new(snapshot));
                    if let Some(frame) = self.call_stack.last_mut() {
                        frame
                            .get_or_init_closure_env()
                            .insert(*func_idx, rc.clone());
                    }
                    rc
                };
                let owner = self.current_module.clone();
                let scope = self.module_globals_rc();
                let heap_idx = self.gc.allocate(
                    &mut self.heap,
                    HeapValue::Function(JsFunction {
                        name: fi.name.clone(),
                        params: fi.params.clone(),
                        rest_param: fi.rest_param.clone(),
                        bytecode_index: fi.bytecode_index,
                        closure: closure_vars,
                        prototype: None,
                        super_class: None,
                        properties: PropertyStorage::new(),
                        owner_module: owner,
                        module_scope: Some(scope),
                        is_generator: fi.is_generator,
                        source_file: self.current_module_path.clone(),
                        source_line: fi.source_line,
                        is_arrow: fi.is_arrow,
                        captured_this: if fi.is_arrow {
                            self.call_stack.last().and_then(|f| f.this_value.clone())
                        } else {
                            None
                        },
                    }),
                );
                self.stack.push(Value::Function(heap_idx));
            }
            Instruction::SnapshotClosure(local_slot, capture_slots) => {
                let base_pointer: usize =
                    self.call_stack.last().map(|f| f.base_pointer).unwrap_or(0);
                let abs_slot = base_pointer + *local_slot as usize;
                if let Some(Value::Function(heap_idx)) = self.stack.get(abs_slot).cloned() {
                    if let HeapValue::Function(f) = &mut self.heap[heap_idx] {
                        let new_values: Vec<Value> = capture_slots
                            .iter()
                            .map(|slot| {
                                let s = base_pointer + *slot as usize;
                                self.stack
                                    .get(s)
                                    .cloned()
                                    .unwrap_or(Value::Undefined)
                            })
                            .collect();
                        *f.closure.borrow_mut() = new_values;
                    }
                }
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}
