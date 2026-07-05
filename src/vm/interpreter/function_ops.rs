use super::*;
use crate::compiler::CompiledModule;
use crate::compiler::Instruction;
use crate::errors::Result;
use crate::objects::Value;
use rustc_hash::FxHashMap;

impl Interpreter {
    fn module_globals_rc(&mut self) -> std::rc::Rc<std::cell::RefCell<FxHashMap<String, Value>>> {
        self.module_globals_rc
            .clone()
            .get_or_insert_with(|| std::rc::Rc::new(std::cell::RefCell::new(self.globals.clone())))
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
                    let cached: Option<Rc<RefCell<Vec<Value>>>> =
                        if let Some(frame) = self.call_stack.last() {
                            frame
                                .shared_closure_env
                                .as_ref()
                                .and_then(|env| env.get(func_idx).cloned())
                        } else {
                            None
                        };
                    match cached {
                        Some(shared) => shared,
                        None => {
                            let rc = Rc::new(RefCell::new(snapshot));
                            if let Some(frame) = self.call_stack.last_mut() {
                                frame
                                    .get_or_init_closure_env()
                                    .insert(*func_idx, rc.clone());
                            }
                            rc
                        }
                    }
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
            _ => return Ok(false),
        }
        Ok(true)
    }
}
