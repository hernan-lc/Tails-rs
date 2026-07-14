use super::*;
use crate::compiler::CompiledModule;
use crate::compiler::Instruction;
use crate::errors::{Error, Result};
use crate::objects::Value;
use crate::well_known as wk;

impl Interpreter {
    fn handle_make_class(&mut self, class_info_idx: &u32, module: &CompiledModule) -> Result<()> {
        let class_info = module.class_infos[*class_info_idx as usize].clone();
        // Capture the module scope so class methods/constructor resolve free
        // variables (e.g. top-level function declarations) against the module
        // that *defines* the class, even when the class is instantiated from a
        // different module (see exec_make_function's module_scope handling).
        let class_scope = self.module_globals_rc();
        let super_val = if class_info.superclass.is_some() {
            self.stack
                .pop()
                .ok_or_else(|| Error::RuntimeError(super::ERR_STACK_UNDERFLOW.into()))?
        } else {
            Value::Undefined
        };
        let proto_obj_idx = self
            .gc
            .allocate(&mut self.heap, HeapValue::Object(JsObject::new()));
        let super_proto = match &super_val {
            Value::Object(super_obj_idx) => {
                if let HeapValue::Object(super_obj) = &self.heap[*super_obj_idx] {
                    super_obj.properties.get(wk::PROTOTYPE).cloned()
                } else {
                    None
                }
            }
            Value::Function(func_idx) => {
                if let HeapValue::Function(f) = &self.heap[*func_idx] {
                    f.prototype.map(Value::Object)
                } else {
                    None
                }
            }
            _ => None,
        };
        if let Some(Value::Object(sp_idx)) = super_proto {
            self.heap[proto_obj_idx] = HeapValue::Object(JsObject::with_prototype(Some(sp_idx)));
        }
        let ctor_heap_idx = if let Some(ctor_func_idx) = class_info.constructor_func_idx {
            let func_info = module.functions[ctor_func_idx as usize].clone();
            let owner = self.current_module.clone();
            let src_file = self.current_module_path.clone();
            let src_line = self.current_source_line(self.current_pc);
            let ctor_base_pointer = self.call_stack.last().map(|f| f.base_pointer).unwrap_or(0);
            let ctor_closure: Rc<RefCell<Vec<Value>>> = if func_info.capture_slots.is_empty() {
                Rc::new(RefCell::new(Vec::new()))
            } else {
                let snapshot: Vec<Value> = func_info
                    .capture_slots
                    .iter()
                    .map(|slot| {
                        self.stack
                            .get(ctor_base_pointer + *slot as usize)
                            .cloned()
                            .unwrap_or(Value::Undefined)
                    })
                    .collect();
                Rc::new(RefCell::new(snapshot))
            };
            self.gc.allocate(
                &mut self.heap,
                HeapValue::Function(JsFunction {
                    name: Some(class_info.name.clone()),
                    params: func_info.params,
                    rest_param: func_info.rest_param,
                    bytecode_index: func_info.bytecode_index,
                    local_count: func_info.local_count,
                    closure: ctor_closure,
                    prototype: Some(proto_obj_idx),
                    super_class: Some(super_val.clone()),
                    properties: PropertyStorage::new(),
                    owner_module: owner,
                    module_scope: Some(class_scope.clone()),
                    is_generator: false,
                    source_file: src_file,
                    source_line: src_line,
                    is_arrow: false,
                    captured_this: None,
                    capture_slots: func_info.capture_slots,
                }),
            )
        } else {
            let src_file = self.current_module_path.clone();
            let src_line = self.current_source_line(self.current_pc);
            self.gc.allocate(
                &mut self.heap,
                HeapValue::Function(JsFunction {
                    name: Some(class_info.name.clone()),
                    params: Vec::new(),
                    rest_param: None,
                    bytecode_index: usize::MAX,
                    local_count: 0,
                    closure: Rc::new(RefCell::new(Vec::new())),
                    prototype: Some(proto_obj_idx),
                    super_class: Some(super_val.clone()),
                    properties: PropertyStorage::new(),
                    owner_module: None,
                    module_scope: Some(class_scope.clone()),
                    is_generator: false,
                    source_file: src_file,
                    source_line: src_line,
                    is_arrow: false,
                    captured_this: None,
                    capture_slots: Vec::new(),
                }),
            )
        };
        if let HeapValue::Object(proto_obj) = &mut self.heap[proto_obj_idx] {
            proto_obj
                .properties
                .insert(wk::CONSTRUCTOR.to_string(), Value::Function(ctor_heap_idx));
        }
        for method_info in &class_info.methods {
            let method_func_info = module.functions[method_info.func_idx as usize].clone();
            let method_proto_idx = self
                .gc
                .allocate(&mut self.heap, HeapValue::Object(JsObject::new()));
            let owner = self.current_module.clone();
            let src_file = self.current_module_path.clone();
            let src_line = self.current_source_line(self.current_pc);
            let method_base_pointer =
                self.call_stack.last().map(|f| f.base_pointer).unwrap_or(0);
            let method_closure: Rc<RefCell<Vec<Value>>> = if method_func_info.capture_slots.is_empty() {
                Rc::new(RefCell::new(Vec::new()))
            } else {
                let snapshot: Vec<Value> = method_func_info
                    .capture_slots
                    .iter()
                    .map(|slot| {
                        self.stack
                            .get(method_base_pointer + *slot as usize)
                            .cloned()
                            .unwrap_or(Value::Undefined)
                    })
                    .collect();
                Rc::new(RefCell::new(snapshot))
            };
            let method_heap_idx = self.gc.allocate(
                &mut self.heap,
                HeapValue::Function(JsFunction {
                    name: Some(method_info.name.clone()),
                    params: method_func_info.params,
                    rest_param: method_func_info.rest_param,
                    bytecode_index: method_func_info.bytecode_index,
                    local_count: method_func_info.local_count,
                    closure: method_closure,
                    prototype: Some(method_proto_idx),
                    super_class: None,
                    properties: PropertyStorage::new(),
                    owner_module: owner,
                    module_scope: Some(class_scope.clone()),
                    is_generator: false,
                    source_file: src_file,
                    source_line: src_line,
                    is_arrow: false,
                    captured_this: None,
                    capture_slots: method_func_info.capture_slots,
                }),
            );
            let method_val = Value::Function(method_heap_idx);
            if method_info.is_static {
                if let HeapValue::Function(ctor_func) = &mut self.heap[ctor_heap_idx] {
                    ctor_func
                        .properties
                        .insert(method_info.name.clone(), method_val);
                }
            } else {
                match &method_info.kind {
                    crate::compiler::ClassMethodKind::Getter => {
                        if let HeapValue::Object(proto_obj) = &mut self.heap[proto_obj_idx] {
                            proto_obj
                                .properties
                                .insert(format!("__getter_{}", method_info.name), method_val);
                        }
                    }
                    crate::compiler::ClassMethodKind::Setter => {
                        if let HeapValue::Object(proto_obj) = &mut self.heap[proto_obj_idx] {
                            proto_obj
                                .properties
                                .insert(format!("__setter_{}", method_info.name), method_val);
                        }
                    }
                    crate::compiler::ClassMethodKind::Method => {
                        if let HeapValue::Object(proto_obj) = &mut self.heap[proto_obj_idx] {
                            proto_obj
                                .properties
                                .insert(method_info.name.clone(), method_val);
                        }
                    }
                }
            }
        }
        self.stack.push(Value::Function(ctor_heap_idx));
        Ok(())
    }

    /// After a class is stored in its local slot via StoreLocal, snapshot
    /// closures for the constructor and all methods from the now-populated
    /// enclosing frame. This is necessary because MakeClass runs BEFORE
    /// StoreLocal, so the enclosing frame's slots (including the class's
    /// own slot) are not yet populated when methods are created.
    fn handle_snapshot_method_closures(
        &mut self,
        class_info_idx: u32,
        local_slot: u16,
        module: &CompiledModule,
    ) -> Result<()> {
        let class_info = module.class_infos[class_info_idx as usize].clone();
        let base_pointer = self.call_stack.last().map(|f| f.base_pointer).unwrap_or(0);
        let abs_slot = base_pointer + local_slot as usize;

        // Get the class constructor function from its local slot
        let ctor_heap_idx = match self.stack.get(abs_slot).cloned() {
            Some(Value::Function(idx)) => idx,
            _ => return Ok(()), // Class not found; nothing to snapshot
        };

        // Collect the prototype object index for instance method lookup
        let proto_obj_idx = if let HeapValue::Function(ctor_func) = &self.heap[ctor_heap_idx] {
            ctor_func.prototype
        } else {
            None
        };

        // Collect all method heap indices that need snapshotting
        let mut targets: Vec<usize> = Vec::new();

        // Constructor: only if it has capture slots
        if class_info.constructor_func_idx.is_some() {
            if let HeapValue::Function(ctor_func) = &self.heap[ctor_heap_idx] {
                if !ctor_func.capture_slots.is_empty() {
                    targets.push(ctor_heap_idx);
                }
            }
        }

        // Methods: find their heap objects and check if they have captures
        for method_info in &class_info.methods {
            let method_func_info = &module.functions[method_info.func_idx as usize];
            if method_func_info.capture_slots.is_empty() {
                continue;
            }

            let method_heap_idx = if method_info.is_static {
                // Static methods are properties of the constructor
                if let HeapValue::Function(ctor_func) = &self.heap[ctor_heap_idx] {
                    ctor_func
                        .properties
                        .get(&method_info.name)
                        .and_then(|v| {
                            if let Value::Function(idx) = v {
                                Some(*idx)
                            } else {
                                None
                            }
                        })
                } else {
                    None
                }
            } else if let Some(proto_idx) = proto_obj_idx {
                // Instance methods are properties of the prototype
                if let HeapValue::Object(proto_obj) = &self.heap[proto_idx] {
                    let lookup_name = match method_info.kind {
                        crate::compiler::ClassMethodKind::Getter => {
                            format!("__getter_{}", method_info.name)
                        }
                        crate::compiler::ClassMethodKind::Setter => {
                            format!("__setter_{}", method_info.name)
                        }
                        crate::compiler::ClassMethodKind::Method => method_info.name.clone(),
                    };
                    proto_obj.properties.get(&lookup_name).and_then(|v| {
                        if let Value::Function(idx) = v {
                            Some(*idx)
                        } else {
                            None
                        }
                    })
                } else {
                    None
                }
            } else {
                None
            };

            if let Some(idx) = method_heap_idx {
                targets.push(idx);
            }
        }

        // Now apply the snapshot to each collected target
        for method_heap_idx in targets {
            let capture_slots = if let HeapValue::Function(f) = &self.heap[method_heap_idx] {
                f.capture_slots.clone()
            } else {
                Vec::new()
            };
            if capture_slots.is_empty() {
                continue;
            }
            let new_values: Vec<Value> = capture_slots
                .iter()
                .map(|slot| {
                    let s = base_pointer + *slot as usize;
                    self.stack.get(s).cloned().unwrap_or(Value::Undefined)
                })
                .collect();
            if let HeapValue::Function(f) = &mut self.heap[method_heap_idx] {
                *f.closure.borrow_mut() = new_values;
            }
        }

        Ok(())
    }

    pub(crate) fn exec_class_ops(
        &mut self,
        instruction: &Instruction,
        pc: &mut usize,
        module: &CompiledModule,
    ) -> Result<bool> {
        match instruction {
            Instruction::MakeClass(class_info_idx) => {
                self.handle_make_class(class_info_idx, module)?;
                Ok(true)
            }
            Instruction::SnapshotMethodClosures(class_info_idx, local_slot) => {
                self.handle_snapshot_method_closures(*class_info_idx, *local_slot, module)?;
                Ok(true)
            }
            Instruction::SuperConstruct(argc) => {
                // Inline handling due to pc control flow
                let mut args = Vec::new();
                for _ in 0..*argc {
                    args.push(
                        self.stack.pop().ok_or_else(|| {
                            Error::RuntimeError(super::ERR_STACK_UNDERFLOW.into())
                        })?,
                    );
                }
                args.reverse();
                let this_val = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError(super::ERR_STACK_UNDERFLOW.into()))?;
                let super_class = {
                    let mut found = Value::Undefined;
                    for frame in self.call_stack.iter().rev() {
                        if let Some(func_idx) = frame.func_heap_idx {
                            if let HeapValue::Function(f) = &self.heap[func_idx] {
                                if let Some(ref sc) = f.super_class {
                                    found = sc.clone();
                                    break;
                                }
                            }
                        }
                    }
                    found
                };
                match super_class {
                    Value::Function(func_idx) => {
                        if let HeapValue::Function(f) = &self.heap[func_idx] {
                            let f_clone = f.clone();
                            let super_module: Option<std::rc::Rc<CompiledModule>> = f_clone
                                .owner_module
                                .clone()
                                .or_else(|| self.current_module.clone());
                            let cross_module = match super_module.as_ref() {
                                Some(m) => !std::ptr::eq(m.as_ref(), module),
                                None => false,
                            };
                            if cross_module {
                                if let Some(ref ctor_mod) = super_module {
                                    let result = self.construct_function_nested(
                                        func_idx,
                                        &f_clone.closure,
                                        f_clone.bytecode_index,
                                        f_clone.local_count,
                                        f_clone.module_scope.clone(),
                                        f_clone.source_file.clone(),
                                        ctor_mod,
                                        this_val.clone(),
                                        Value::Function(func_idx),
                                        args,
                                        *pc,
                                    )?;
                                    self.stack.push(result);
                                    *pc += 1;
                                    return Ok(true);
                                }
                            }
                            let return_address = *pc + 1;
                            let base_pointer = self.stack.len();
                            let closure_count = f_clone.closure.borrow().len();
                            let local_count = f_clone.local_count;
                            self.call_stack.push(CallFrame {
                                return_address,
                                base_pointer,
                                closure_var_count: closure_count,
                                func_heap_idx: Some(func_idx),
                                this_value: Some(this_val.clone()),
                                is_construct: true,
                                new_target: Some(Value::Function(func_idx)),
                                source_name: self.current_module_path.clone(),
                                generator_heap_idx: None,
                                source_line: self.current_source_line(*pc),
                                source_col: self.current_source_col(*pc),
                                exception_handlers_snapshot: if self.exception_handlers.is_empty() {
                                    Vec::new()
                                } else {
                                    self.exception_handlers.clone()
                                },
                                arguments: None,
                            });
                            for closure_var in f_clone.closure.borrow().iter().cloned() {
                                self.stack.push(closure_var);
                            }
                            for arg in args {
                                self.stack.push(arg);
                            }
                            self.reserve_frame_locals(base_pointer, local_count);
                            *pc = f_clone.bytecode_index;
                            return Ok(true);
                        }
                    }
                    Value::NativeFunction(native_idx) => {
                        let result = self.call_native(native_idx, &this_val, &args)?;
                        self.stack.push(result);
                    }
                    _ => {
                        return Err(self.err_at_location(Error::TypeError(
                            "Superclass is not a constructor".into(),
                        )));
                    }
                }
                Ok(true)
            }
            Instruction::SuperGet => {
                let key = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError(super::ERR_STACK_UNDERFLOW.into()))?;
                let _this = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError(super::ERR_STACK_UNDERFLOW.into()))?;
                let super_class = {
                    let mut found = Value::Undefined;
                    for frame in self.call_stack.iter().rev() {
                        if let Some(func_idx) = frame.func_heap_idx {
                            if let HeapValue::Function(f) = &self.heap[func_idx] {
                                if let Some(ref sc) = f.super_class {
                                    found = sc.clone();
                                    break;
                                }
                            }
                        }
                    }
                    found
                };
                if let Value::Function(func_idx) = &super_class {
                    if let HeapValue::Function(f) = &self.heap[*func_idx] {
                        if let Some(proto_idx) = f.prototype {
                            let proto_val = Value::Object(proto_idx);
                            let result = self.get_property(&proto_val, &key)?;
                            self.stack.push(result);
                            return Ok(true);
                        }
                    }
                }
                self.stack.push(Value::Undefined);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
