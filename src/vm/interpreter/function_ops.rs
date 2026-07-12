use super::*;
use crate::compiler::CompiledModule;
use crate::compiler::Instruction;
use crate::errors::Result;
use crate::objects::Value;
use rustc_hash::FxHashMap;

impl Interpreter {
    /// Re-fill a function's closure after self-referential assignment
    /// (`var f = function(){ return f }`). Only runs when the function is
    /// stored into a local that it actually captures — never when the same
    /// function value is later assigned to an unrelated local (e.g. wrappy's
    /// `var ret = once(...)`), which would re-read wrong parent-frame slots.
    pub(crate) fn resnapshot_function_closure(
        &mut self,
        heap_idx: usize,
        base_pointer: usize,
        stored_to_slot: u16,
    ) {
        let slots = match &self.heap[heap_idx] {
            HeapValue::Function(f) if !f.capture_slots.is_empty() => f.capture_slots.clone(),
            _ => return,
        };
        if !slots.contains(&stored_to_slot) {
            return;
        }
        let new_values: Vec<Value> = slots
            .iter()
            .map(|slot| {
                let s = base_pointer + *slot as usize;
                self.stack.get(s).cloned().unwrap_or(Value::Undefined)
            })
            .collect();
        if let HeapValue::Function(f) = &mut self.heap[heap_idx] {
            *f.closure.borrow_mut() = new_values;
        }
    }

    pub(crate) fn module_globals_rc(&mut self) -> std::rc::Rc<std::cell::RefCell<FxHashMap<String, Value>>> {
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
                        local_count: fi.local_count,
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
                        capture_slots: Vec::new(),
                    }),
                );
                self.stack.push(Value::Function(heap_idx));
            }
            Instruction::MakeClosure(func_idx, capture_slots) => {
                let fi = &module.functions[*func_idx as usize];
                let base_pointer: usize =
                    self.call_stack.last().map(|f| f.base_pointer).unwrap_or(0);
                let slots: Vec<u16> = capture_slots.to_vec();
                let snapshot: Vec<Value> = slots
                    .iter()
                    .map(|slot| {
                        let abs_slot = base_pointer + *slot as usize;
                        self.stack
                            .get(abs_slot)
                            .cloned()
                            .unwrap_or(Value::Undefined)
                    })
                    .collect();
                // Each closure gets its own copy of the captured values.
                // Do not stash into `shared_closure_env`: that map was for a
                // sibling-share optimization that incorrectly made loop-body
                // closures capture only the first iteration's values. Keeping
                // the insert also allocated a HashMap on every MakeClosure
                // call site and held an extra Rc root until the frame returned.
                let closure_vars: Rc<RefCell<Vec<Value>>> = Rc::new(RefCell::new(snapshot));
                let owner = self.current_module.clone();
                let scope = self.module_globals_rc();
                let heap_idx = self.gc.allocate(
                    &mut self.heap,
                    HeapValue::Function(JsFunction {
                        name: fi.name.clone(),
                        params: fi.params.clone(),
                        rest_param: fi.rest_param.clone(),
                        bytecode_index: fi.bytecode_index,
                        local_count: fi.local_count,
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
                        capture_slots: slots,
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
                                self.stack.get(s).cloned().unwrap_or(Value::Undefined)
                            })
                            .collect();
                        f.capture_slots = capture_slots.to_vec();
                        *f.closure.borrow_mut() = new_values;
                    }
                }
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}
