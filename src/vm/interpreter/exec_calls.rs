use super::*;

impl Interpreter {
    pub(crate) fn exec_call(
        &mut self,
        argc: &u16,
        _module: &CompiledModule,
        pc: &mut usize,
    ) -> Result<bool> {
        let callee = self
            .stack
            .pop()
            .ok_or_else(|| Error::RuntimeError(super::ERR_STACK_UNDERFLOW.into()))?;
        let mut args = Vec::new();
        for _ in 0..*argc {
            args.push(
                self.stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError(super::ERR_STACK_UNDERFLOW.into()))?,
            );
        }
        args.reverse();
        match &callee {
            Value::Function(func_idx) => {
                let (is_generator, bytecode_index, has_promise_resolve) =
                    if let HeapValue::Function(f) = &self.heap[*func_idx] {
                        let has_promise = f.bytecode_index == usize::MAX
                            && f.closure
                                .borrow()
                                .first()
                                .is_some_and(|v| matches!(v, Value::Promise(_)));
                        (f.is_generator, f.bytecode_index, has_promise)
                    } else {
                        (false, 0, false)
                    };

                if is_generator {
                    if cfg!(debug_assertions) && std::env::var("GEN_TRACE").is_ok() {
                        eprintln!("[GEN_TRACE] Detected generator, creating object");
                    }
                    let gen_idx = self.heap.len();
                    self.heap.push(HeapValue::Generator(JsGenerator {
                        yield_value: Value::Undefined,
                        resume_pc: bytecode_index,
                        saved_stack: args.clone(),
                        saved_block_scope_stack: Vec::new(),
                        func_heap_idx: Some(*func_idx),
                        generator_yielded: false,
                    }));
                    self.stack.push(Value::Generator(gen_idx));
                    *pc += 1;
                    return Ok(true);
                } else if has_promise_resolve {
                    if let HeapValue::Function(f) = &self.heap[*func_idx] {
                        let promise_idx = {
                            let closure = f.closure.borrow();
                            closure.first().and_then(|v| {
                                if let Value::Promise(idx) = v {
                                    Some(*idx)
                                } else {
                                    None
                                }
                            })
                        };
                        if let Some(promise_idx) = promise_idx {
                            match f.name.as_deref() {
                                Some("resolve") => {
                                    let val = args.first().cloned().unwrap_or(Value::Undefined);
                                    self.resolve_promise(promise_idx, val);
                                    self.stack.push(Value::Undefined);
                                }
                                Some("reject") => {
                                    let reason = args.first().cloned().unwrap_or(Value::Undefined);
                                    self.reject_promise(promise_idx, reason);
                                    self.stack.push(Value::Undefined);
                                }
                                _ => {
                                    self.stack.push(Value::Undefined);
                                }
                            }
                            *pc += 1;
                            return Ok(true);
                        }
                    }
                } else if bytecode_index == usize::MAX {
                    if let HeapValue::Function(f) = &self.heap[*func_idx] {
                        let len = f.closure.borrow().len();
                        if f.name.as_deref() == Some("bound") && len >= 2 {
                            let (original_fn, bound_this, bound_args) = {
                                let closure = f.closure.borrow();
                                (
                                    closure[0].clone(),
                                    closure[1].clone(),
                                    closure[2..].to_vec(),
                                )
                            };
                            let mut combined_args = bound_args;
                            combined_args.extend(args);
                            let result =
                                self.call_value(&original_fn, &bound_this, &combined_args)?;
                            self.stack.push(result);
                            *pc += 1;
                            return Ok(true);
                        }
                    }
                    self.stack.push(Value::Undefined);
                } else {
                    let same_module = if let HeapValue::Function(f) = &self.heap[*func_idx] {
                        match (&f.owner_module, &self.current_module) {
                            (Some(om), Some(cm)) => Rc::ptr_eq(om, cm),
                            (None, None) => true,
                            _ => false,
                        }
                    } else {
                        false
                    };
                    if same_module {
                        let func_info = {
                            if let HeapValue::Function(f) = &self.heap[*func_idx] {
                                let closure_rc = if f.closure.borrow().is_empty() {
                                    Rc::new(RefCell::new(Vec::new()))
                                } else {
                                    f.closure.clone()
                                };
                                let captured_this = if f.is_arrow {
                                    f.captured_this.clone()
                                } else {
                                    None
                                };
                                Some((
                                    closure_rc,
                                    f.bytecode_index,
                                    f.is_arrow,
                                    captured_this,
                                    f.rest_param.is_some(),
                                    f.params.len(),
                                    f.local_count,
                                ))
                            } else {
                                None
                            }
                        };
                        if let Some((
                            closure_vars,
                            bytecode_index,
                            is_arrow,
                            captured_this,
                            has_rest,
                            param_count,
                            local_count,
                        )) = func_info
                        {
                            let return_address = *pc + 1;
                            let base_pointer = self.stack.len();
                            let closure_count = closure_vars.borrow().len();
                            let this_for_frame = if is_arrow {
                                captured_this.unwrap_or(Value::Undefined)
                            } else {
                                Value::Undefined
                            };
                            if self.call_stack.len() >= self.max_call_stack_depth {
                                self.throw_stack_overflow(pc)?;
                                return Ok(true);
                            }
                            self.call_stack.push(CallFrame {
                                return_address,
                                base_pointer,
                                closure_var_count: closure_count,
                                func_heap_idx: Some(*func_idx),
                                this_value: Some(this_for_frame),
                                is_construct: false,
                                source_name: self.current_module_path.clone(),
                                generator_heap_idx: None,
                                source_line: self.current_source_line(*pc),
                                source_col: self.current_source_col(*pc),
                                exception_handlers_snapshot: if self.exception_handlers.is_empty() {
                                    Vec::new()
                                } else {
                                    self.exception_handlers.clone()
                                },
                            });
                            for closure_var in closure_vars.borrow().iter().cloned() {
                                self.stack.push(closure_var);
                            }
                            if has_rest {
                                for arg in args.iter().take(param_count) {
                                    self.stack.push(arg.clone());
                                }
                                let rest_args: Vec<Value> = args[param_count..].to_vec();
                                let rest_arr_idx = self.gc.allocate(
                                    &mut self.heap,
                                    HeapValue::Array(JsArray {
                                        elements: rest_args,
                                    }),
                                );
                                self.stack.push(Value::Array(rest_arr_idx));
                            } else {
                                for arg in args {
                                    self.stack.push(arg);
                                }
                            }
                            self.reserve_frame_locals(base_pointer, local_count);
                            *pc = bytecode_index;
                            return Ok(true);
                        }
                    }
                }
                let result = self.call_value(&callee, &Value::Undefined, &args)?;
                self.stack.push(result);
            }
            Value::NativeFunction(native_idx) => {
                let result = self.call_native(*native_idx, &Value::Undefined, &args)?;
                self.stack.push(result);
            }
            Value::Proxy(proxy_idx) => {
                if let HeapValue::Proxy(proxy) = &self.heap[*proxy_idx] {
                    let handler = proxy.handler.clone();
                    let target = proxy.target.clone();
                    let arr_idx = self
                        .gc
                        .allocate(&mut self.heap, HeapValue::Array(JsArray { elements: args }));
                    let trap_result = self.call_proxy_trap(
                        &handler,
                        "apply",
                        &[target, Value::Undefined, Value::Array(arr_idx)],
                    );
                    match trap_result {
                        Ok(v) => self.stack.push(v),
                        Err(e) => return Err(e),
                    }
                } else {
                    return Err(self.err_at_location(Error::TypeError(format!(
                        "{} is not a function",
                        self.value_to_string(&callee)
                    ))));
                }
            }
            _ => {
                return Err(self.err_at_location(Error::TypeError(format!(
                    "{} is not a function",
                    self.value_to_string(&callee)
                ))));
            }
        }
        *pc += 1;
        Ok(true)
    }

    pub(crate) fn exec_call_method(&mut self, argc: &u16, pc: &mut usize) -> Result<bool> {
        let mut args = Vec::with_capacity(usize::from(*argc));
        for _ in 0..*argc {
            args.push(
                self.stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError(super::ERR_STACK_UNDERFLOW.into()))?,
            );
        }
        args.reverse();
        let key = self
            .stack
            .pop()
            .ok_or_else(|| Error::RuntimeError(super::ERR_STACK_UNDERFLOW.into()))?;
        let object = self
            .stack
            .pop()
            .ok_or_else(|| Error::RuntimeError(super::ERR_STACK_UNDERFLOW.into()))?;
        let method = self.get_property(&object, &key)?;
        match method {
            Value::Function(func_idx) => {
                if let HeapValue::Function(f) = &self.heap[func_idx] {
                    if f.bytecode_index == usize::MAX
                        && f.name.as_deref() == Some("bound")
                        && f.closure.borrow().len() >= 2
                    {
                        let (original_fn, bound_this, bound_args) = {
                            let closure = f.closure.borrow();
                            (
                                closure[0].clone(),
                                closure[1].clone(),
                                closure[2..].to_vec(),
                            )
                        };
                        let mut combined_args = bound_args;
                        combined_args.extend(args);
                        let result = self.call_value(&original_fn, &bound_this, &combined_args)?;
                        self.stack.push(result);
                        *pc += 1;
                        return Ok(true);
                    }

                    let closure_rc = if f.closure.borrow().is_empty() {
                        Rc::new(RefCell::new(Vec::new()))
                    } else {
                        f.closure.clone()
                    };
                    let captured_this = if f.is_arrow {
                        f.captured_this.clone()
                    } else {
                        None
                    };
                    let func_info = Some((
                        closure_rc,
                        f.bytecode_index,
                        f.is_arrow,
                        captured_this,
                        f.rest_param.is_some(),
                        f.params.len(),
                        f.local_count,
                    ));
                    let same_module = match (&f.owner_module, &self.current_module) {
                        (Some(om), Some(cm)) => Rc::ptr_eq(om, cm),
                        (None, None) => true,
                        _ => false,
                    };
                    if same_module {
                        if let Some((
                            closure_vars,
                            bytecode_index,
                            is_arrow,
                            captured_this,
                            has_rest,
                            param_count,
                            local_count,
                        )) = func_info
                        {
                            let return_address = *pc + 1;
                            let base_pointer = self.stack.len();
                            let closure_count = closure_vars.borrow().len();
                            let this_for_frame = if is_arrow {
                                captured_this.unwrap_or_else(|| object.clone())
                            } else {
                                object.clone()
                            };
                            if self.call_stack.len() >= self.max_call_stack_depth {
                                self.throw_stack_overflow(pc)?;
                                return Ok(true);
                            }
                            self.call_stack.push(CallFrame {
                                return_address,
                                base_pointer,
                                closure_var_count: closure_count,
                                func_heap_idx: Some(func_idx),
                                this_value: Some(this_for_frame),
                                is_construct: false,
                                source_name: self.current_module_path.clone(),
                                generator_heap_idx: None,
                                source_line: self.current_source_line(*pc),
                                source_col: self.current_source_col(*pc),
                                exception_handlers_snapshot: if self.exception_handlers.is_empty() {
                                    Vec::new()
                                } else {
                                    self.exception_handlers.clone()
                                },
                            });
                            for closure_var in closure_vars.borrow().iter().cloned() {
                                self.stack.push(closure_var);
                            }
                            if has_rest {
                                for arg in args.iter().take(param_count) {
                                    self.stack.push(arg.clone());
                                }
                                let rest_args: Vec<Value> = args[param_count..].to_vec();
                                let rest_arr_idx = self.gc.allocate(
                                    &mut self.heap,
                                    HeapValue::Array(JsArray {
                                        elements: rest_args,
                                    }),
                                );
                                self.stack.push(Value::Array(rest_arr_idx));
                            } else {
                                for arg in args {
                                    self.stack.push(arg);
                                }
                            }
                            self.reserve_frame_locals(base_pointer, local_count);
                            *pc = bytecode_index;
                            return Ok(true);
                        }
                    }
                    let result = self.call_value(&method, &object, &args)?;
                    self.stack.push(result);
                }
            }
            Value::NativeFunction(native_idx) => {
                let result = self.call_native(native_idx, &object, &args)?;
                self.stack.push(result);
            }
            _ => {
                return Err(self.err_at_location(Error::TypeError(format!(
                    "{} is not a function",
                    self.value_to_string(&method)
                ))));
            }
        }
        *pc += 1;
        Ok(true)
    }

    pub(crate) fn exec_construct_apply(
        &mut self,
        module: &CompiledModule,
        pc: &mut usize,
    ) -> Result<bool> {
        // Stack: [argsArray, ctor]
        let constructor = self
            .stack
            .pop()
            .ok_or_else(|| Error::RuntimeError(super::ERR_STACK_UNDERFLOW.into()))?;
        let args_val = self
            .stack
            .pop()
            .ok_or_else(|| Error::RuntimeError(super::ERR_STACK_UNDERFLOW.into()))?;
        let args = match args_val {
            Value::Array(arr_idx) => {
                if let HeapValue::Array(arr) = &self.heap[arr_idx] {
                    arr.elements.clone()
                } else {
                    Vec::new()
                }
            }
            Value::Undefined | Value::Null => Vec::new(),
            other => vec![other],
        };
        self.exec_construct_with_args(constructor, args, module, pc)
    }

    pub(crate) fn exec_construct(
        &mut self,
        argc: &u16,
        module: &CompiledModule,
        pc: &mut usize,
    ) -> Result<bool> {
        let mut args = Vec::new();
        for _ in 0..*argc {
            args.push(
                self.stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError(super::ERR_STACK_UNDERFLOW.into()))?,
            );
        }
        args.reverse();
        let constructor = self
            .stack
            .pop()
            .ok_or_else(|| Error::RuntimeError(super::ERR_STACK_UNDERFLOW.into()))?;
        self.exec_construct_with_args(constructor, args, module, pc)
    }

    fn exec_construct_with_args(
        &mut self,
        constructor: Value,
        args: Vec<Value>,
        _module: &CompiledModule,
        pc: &mut usize,
    ) -> Result<bool> {
        match &constructor {
            Value::Function(func_idx) => {
                let proto_idx = if let Value::Object(proto_obj_idx) =
                    self.get_property(&constructor, &Value::from_string("prototype".to_string()))?
                {
                    Some(proto_obj_idx)
                } else {
                    None
                };
                let new_obj_heap_idx = self.gc.allocate(
                    &mut self.heap,
                    HeapValue::Object(JsObject::with_prototype(proto_idx)),
                );
                let this_val = Value::Object(new_obj_heap_idx);
                if let HeapValue::Function(f) = &self.heap[*func_idx] {
                    if f.bytecode_index == usize::MAX {
                        if let Some(ref super_val) = f.super_class {
                            if let Value::Function(super_func_idx) = super_val {
                                if let HeapValue::Function(super_f) = &self.heap[*super_func_idx] {
                                    if super_f.bytecode_index != usize::MAX {
                                        let super_f_clone = super_f.clone();
                                        let return_address = *pc + 1;
                                        let base_pointer = self.stack.len();
                                        if self.call_stack.len() >= self.max_call_stack_depth {
                                            self.throw_stack_overflow(pc)?;
                                            return Ok(true);
                                        }
                                        let local_count = super_f_clone.local_count;
                                        self.call_stack.push(CallFrame {
                                            return_address,
                                            base_pointer,
                                            closure_var_count: 0,
                                            func_heap_idx: Some(*super_func_idx),
                                            this_value: Some(this_val.clone()),
                                            is_construct: true,
                                            source_name: self.current_module_path.clone(),
                                            generator_heap_idx: None,
                                            source_line: self.current_source_line(*pc),
                                            source_col: self.current_source_col(*pc),
                                            exception_handlers_snapshot: self
                                                .exception_handlers
                                                .clone(),
                                        });
                                        for arg in args {
                                            self.stack.push(arg);
                                        }
                                        self.reserve_frame_locals(base_pointer, local_count);
                                        *pc = super_f_clone.bytecode_index;
                                        return Ok(true);
                                    }
                                }
                            } else if let Value::NativeFunction(super_native_idx) = super_val {
                                let result =
                                    self.call_native(*super_native_idx, &this_val, &args)?;
                                match result {
                                    Value::Object(_)
                                    | Value::Array(_)
                                    | Value::Function(_)
                                    | Value::Promise(_)
                                    | Value::Proxy(_)
                                    | Value::Date(_)
                                    | Value::RegExp(_)
                                    | Value::Map(_)
                                    | Value::Set(_)
                                    | Value::TypedArray(_) => {
                                        self.stack.push(result);
                                    }
                                    _ => {
                                        self.stack.push(this_val);
                                    }
                                }
                                *pc += 1;
                                return Ok(true);
                            }
                        }
                        self.stack.push(this_val);
                    } else {
                        let func_info = {
                            if let HeapValue::Function(f) = &self.heap[*func_idx] {
                                Some((f.closure.clone(), f.bytecode_index, f.local_count))
                            } else {
                                None
                            }
                        };
                        if let Some((closure_vars, bytecode_index, local_count)) = func_info {
                            let same_module = if let HeapValue::Function(f) = &self.heap[*func_idx]
                            {
                                match (&f.owner_module, &self.current_module) {
                                    (Some(om), Some(cm)) => Rc::ptr_eq(om, cm),
                                    (None, None) => true,
                                    _ => false,
                                }
                            } else {
                                false
                            };
                            if same_module {
                                let return_address = *pc + 1;
                                let base_pointer = self.stack.len();
                                let closure_count = closure_vars.borrow().len();
                                if self.call_stack.len() >= self.max_call_stack_depth {
                                    self.throw_stack_overflow(pc)?;
                                    return Ok(true);
                                }
                                self.call_stack.push(CallFrame {
                                    return_address,
                                    base_pointer,
                                    closure_var_count: closure_count,
                                    func_heap_idx: Some(*func_idx),
                                    this_value: Some(this_val.clone()),
                                    is_construct: true,
                                    source_name: self.current_module_path.clone(),
                                    generator_heap_idx: None,
                                    source_line: self.current_source_line(*pc),
                                    source_col: self.current_source_col(*pc),
                                    exception_handlers_snapshot: self.exception_handlers.clone(),
                                });
                                for closure_var in closure_vars.borrow().iter().cloned() {
                                    self.stack.push(closure_var);
                                }
                                for arg in args {
                                    self.stack.push(arg);
                                }
                                self.reserve_frame_locals(base_pointer, local_count);
                                *pc = bytecode_index;
                                return Ok(true);
                            }
                        }
                        let result = self.call_value(&constructor, &this_val, &args)?;
                        match result {
                            Value::Object(_)
                            | Value::Array(_)
                            | Value::Function(_)
                            | Value::Promise(_)
                            | Value::Proxy(_) => {
                                self.stack.push(result);
                            }
                            _ => {
                                self.stack.push(this_val);
                            }
                        }
                    }
                }
            }
            Value::NativeFunction(native_idx) => {
                let final_args = args.clone();
                let proto_idx = self.find_native_prototype(*native_idx);
                let new_obj_heap_idx = self.gc.allocate(
                    &mut self.heap,
                    HeapValue::Object(JsObject::with_prototype(proto_idx)),
                );
                let this_val = Value::Object(new_obj_heap_idx);
                let result = self.call_native(*native_idx, &this_val, &final_args)?;
                match result {
                    Value::Object(_)
                    | Value::Array(_)
                    | Value::Function(_)
                    | Value::Promise(_)
                    | Value::Proxy(_)
                    | Value::Date(_)
                    | Value::RegExp(_)
                    | Value::Map(_)
                    | Value::Set(_)
                    | Value::WeakMap(_)
                    | Value::WeakSet(_)
                    | Value::TypedArray(_)
                    | Value::NativeObject(_) => {
                        self.stack.push(result);
                    }
                    _ => {
                        self.stack.push(this_val);
                    }
                }
            }
            Value::Proxy(proxy_idx) => {
                if let HeapValue::Proxy(proxy) = &self.heap[*proxy_idx] {
                    let handler = proxy.handler.clone();
                    let target = proxy.target.clone();
                    let args_arr_idx = self
                        .gc
                        .allocate(&mut self.heap, HeapValue::Array(JsArray { elements: args }));
                    let trap_result = self.call_proxy_trap(
                        &handler,
                        "construct",
                        &[target, Value::Array(args_arr_idx), constructor.clone()],
                    );
                    match trap_result {
                        Ok(v) => self.stack.push(v),
                        Err(e) => return Err(e),
                    }
                } else {
                    return Err(self.err_at_location(Error::TypeError(format!(
                        "{} is not a constructor",
                        self.value_to_string(&constructor)
                    ))));
                }
            }
            _ => {
                return Err(self.err_at_location(Error::TypeError(format!(
                    "{} is not a constructor",
                    self.value_to_string(&constructor)
                ))));
            }
        }
        *pc += 1;
        Ok(true)
    }
}
