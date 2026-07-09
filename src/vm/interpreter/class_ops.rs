use super::*;
use crate::compiler::CompiledModule;
use crate::compiler::Instruction;
use crate::errors::{Error, Result};
use crate::objects::Value;

impl Interpreter {
    fn handle_make_class(&mut self, class_info_idx: &u32, module: &CompiledModule) -> Result<()> {
        let class_info = module.class_infos[*class_info_idx as usize].clone();
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
                    super_obj.properties.get("prototype").cloned()
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
            self.gc.allocate(
                &mut self.heap,
                HeapValue::Function(JsFunction {
                    name: Some(class_info.name.clone()),
                    params: func_info.params,
                    rest_param: func_info.rest_param,
                    bytecode_index: func_info.bytecode_index,
                    local_count: func_info.local_count,
                    closure: Rc::new(RefCell::new(Vec::new())),
                    prototype: Some(proto_obj_idx),
                    super_class: Some(super_val.clone()),
                    properties: PropertyStorage::new(),
                    owner_module: owner,
                    module_scope: None,
                    is_generator: false,
                    source_file: src_file,
                    source_line: src_line,
                    is_arrow: false,
                    captured_this: None,
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
                    module_scope: None,
                    is_generator: false,
                    source_file: src_file,
                    source_line: src_line,
                    is_arrow: false,
                    captured_this: None,
                }),
            )
        };
        if let HeapValue::Object(proto_obj) = &mut self.heap[proto_obj_idx] {
            proto_obj
                .properties
                .insert("constructor".to_string(), Value::Function(ctor_heap_idx));
        }
        for method_info in &class_info.methods {
            let method_func_info = module.functions[method_info.func_idx as usize].clone();
            let method_proto_idx = self
                .gc
                .allocate(&mut self.heap, HeapValue::Object(JsObject::new()));
            let owner = self.current_module.clone();
            let src_file = self.current_module_path.clone();
            let src_line = self.current_source_line(self.current_pc);
            let method_heap_idx = self.gc.allocate(
                &mut self.heap,
                HeapValue::Function(JsFunction {
                    name: Some(method_info.name.clone()),
                    params: method_func_info.params,
                    rest_param: method_func_info.rest_param,
                    bytecode_index: method_func_info.bytecode_index,
                    local_count: method_func_info.local_count,
                    closure: Rc::new(RefCell::new(Vec::new())),
                    prototype: Some(method_proto_idx),
                    super_class: None,
                    properties: PropertyStorage::new(),
                    owner_module: owner,
                    module_scope: None,
                    is_generator: false,
                    source_file: src_file,
                    source_line: src_line,
                    is_arrow: false,
                    captured_this: None,
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
                            let proto_val = self.get_property(
                                &super_class,
                                &Value::String("prototype".to_string()),
                            )?;
                            let proto_idx = if let Value::Object(pi) = proto_val {
                                Some(pi)
                            } else {
                                None
                            };
                            let new_obj_heap_idx = self.gc.allocate(
                                &mut self.heap,
                                HeapValue::Object(JsObject::with_prototype(proto_idx)),
                            );
                            let _constructed = Value::Object(new_obj_heap_idx);
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
                                source_name: self.current_module_path.clone(),
                                generator_heap_idx: None,
                                source_line: self.current_source_line(*pc),
                                source_col: self.current_source_col(*pc),
                                exception_handlers_snapshot: if self.exception_handlers.is_empty() {
                                    Vec::new()
                                } else {
                                    self.exception_handlers.clone()
                                },
                                shared_closure_env: None,
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
