use super::*;
use crate::errors::{Error, Result};
use crate::objects::Value;
use crate::runtime_env::native_fns::NATIVE_TABLE;
use crate::well_known as wk;
use std::rc::Rc;

impl Interpreter {
    /// Ensure the operand stack has room for all of a function's local slots
    /// (captures + params + body locals) starting at `base_pointer`. Evaluation
    /// temps then sit above locals, so StoreLocal cannot clobber them.
    #[inline]
    pub(crate) fn reserve_frame_locals(&mut self, base_pointer: usize, local_count: usize) {
        let needed = base_pointer.saturating_add(local_count);
        if self.stack.len() < needed {
            self.stack.resize(needed, Value::Undefined);
        }
    }

    pub fn call_value(&mut self, callee: &Value, this: &Value, args: &[Value]) -> Result<Value> {
        match callee {
            Value::Function(func_idx) => {
                match &self.heap[*func_idx] {
                    HeapValue::DeferredResolve(promise_idx) => {
                        let value = args.first().cloned().unwrap_or(Value::Undefined);
                        self.resolve_promise(*promise_idx, value);
                        #[allow(clippy::needless_return)]
                        return Ok(Value::Undefined);
                    }
                    HeapValue::DeferredReject(promise_idx) => {
                        let reason = args.first().cloned().unwrap_or(Value::Undefined);
                        self.reject_promise(*promise_idx, reason);
                        #[allow(clippy::needless_return)]
                        return Ok(Value::Undefined);
                    }
                    HeapValue::Function(f) => {
                        // Quick path: resolve/reject promise callbacks
                        if f.bytecode_index == usize::MAX {
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
                                let value = args.first().cloned().unwrap_or(Value::Undefined);
                                if f.name.as_deref() == Some("resolve") {
                                    self.resolve_promise(promise_idx, value);
                                } else if f.name.as_deref() == Some("reject") {
                                    self.reject_promise(promise_idx, value);
                                }
                                return Ok(Value::Undefined);
                            }
                        }

                        // Extract only the fields we need (avoids cloning entire JsFunction)
                        let bytecode_index = f.bytecode_index;
                        let closure = if f.closure.borrow().is_empty() {
                            Rc::new(RefCell::new(Vec::new()))
                        } else {
                            f.closure.clone()
                        };
                        let owner_module = f.owner_module.clone();
                        let module_scope = f.module_scope.clone();
                        let is_arrow = f.is_arrow;
                        let captured_this = if is_arrow {
                            f.captured_this.clone()
                        } else {
                            None
                        };
                        let source_file = f.source_file.clone();
                        let source_line = f.source_line;
                        let rest_param = f.rest_param.is_some();
                        let param_count = f.params.len();
                        let local_count = f.local_count;

                        let func_module: Option<Rc<CompiledModule>> =
                            owner_module.or_else(|| self.current_module.clone());
                        let return_address = func_module
                            .as_ref()
                            .map(|m| m.instructions.len())
                            .unwrap_or(0);
                        let base_pointer = self.stack.len();
                        let closure_count = closure.borrow().len();

                        let saved_mg = self.module_globals.take();
                        let saved_mg_rc = self.module_globals_rc.take();
                        if let Some(ref scope) = module_scope {
                            self.module_globals = Some(scope.clone());
                            self.module_globals_rc = Some(scope.clone());
                        }

                        let saved_module = self.current_module.clone();
                        let saved_path = self.current_module_path.clone();
                        let saved_exception_handlers = if self.exception_handlers.is_empty() {
                            Vec::new()
                        } else {
                            self.exception_handlers.clone()
                        };
                        let exception_handlers_snapshot = saved_exception_handlers.clone();
                        if let Some(ref mod_ref) = func_module {
                            self.current_module = Some(mod_ref.clone());
                        }
                        if source_file.is_some() {
                            self.current_module_path = source_file.clone();
                        }

                        let this_for_frame = if is_arrow {
                            captured_this.unwrap_or_else(|| this.clone())
                        } else {
                            this.clone()
                        };
                        if self.call_stack.len() >= self.max_call_stack_depth {
                            let msg = "Maximum call stack size exceeded".to_string();
                            return Err(crate::errors::Error::RuntimeError(msg));
                        }
                        self.call_stack.push(CallFrame {
                            return_address,
                            base_pointer,
                            closure_var_count: closure_count,
                            func_heap_idx: Some(*func_idx),
                            this_value: Some(this_for_frame),
                            is_construct: false,
                            source_name: source_file.or_else(|| self.current_module_path.clone()),
                            generator_heap_idx: None,
                            source_line,
                            source_col: None,
                            exception_handlers_snapshot,
                        });

                        for closure_var in closure.borrow().iter().cloned() {
                            self.stack.push(closure_var);
                        }
                        if rest_param {
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
                                self.stack.push(arg.clone());
                            }
                        }
                        self.reserve_frame_locals(base_pointer, local_count);

                        let result = if let Some(module) = func_module {
                            self.execute_from(&module, bytecode_index)
                        } else {
                            Ok(Value::Undefined)
                        };

                        self.current_module = saved_module;
                        self.current_module_path = saved_path;
                        self.module_globals = saved_mg;
                        self.module_globals_rc = saved_mg_rc;
                        self.exception_handlers = saved_exception_handlers;
                        result
                    }
                    _ => Err(self.err_at_location(Error::TypeError("Not a function".into()))),
                }
            }
            Value::NativeFunction(native_idx) => self.call_native(*native_idx, this, args),
            Value::Proxy(proxy_idx) => {
                if let HeapValue::Proxy(proxy) = &self.heap[*proxy_idx] {
                    let handler = proxy.handler.clone();
                    let target = proxy.target.clone();
                    let arr_idx = self.gc.allocate(
                        &mut self.heap,
                        HeapValue::Array(JsArray {
                            elements: args.to_vec(),
                        }),
                    );
                    self.call_proxy_trap(
                        &handler,
                        "apply",
                        &[target, this.clone(), Value::Array(arr_idx)],
                    )
                } else {
                    Err(self.err_at_location(Error::TypeError(format!(
                        "{} is not a function",
                        self.value_to_string(callee)
                    ))))
                }
            }
            _ => Err(self.err_at_location(Error::TypeError(format!(
                "{} is not a function",
                self.value_to_string(callee)
            )))),
        }
    }

    pub(crate) fn call_native(
        &mut self,
        idx: usize,
        this: &Value,
        args: &[Value],
    ) -> Result<Value> {
        if idx < NATIVE_TABLE.len() {
            NATIVE_TABLE[idx](self, this, args)
        } else {
            // Check for dynamic native functions (from loaded .so/.dylib modules)
            let dynamic_idx = idx - NATIVE_TABLE.len();
            if let Some(&func_ptr) = self.dynamic_native_fns.get(dynamic_idx) {
                // The func_ptr is a C ABI function pointer stored as usize
                // We need to call it with the C ABI signature
                // C ABI: extern "C" fn(interp: *mut c_void, this: NativeValue, args: *const NativeValue, argc: i32) -> NativeValue
                // Safety: func_ptr is guaranteed to have the correct signature because:
                // 1. It was registered through the native function registration system
                // 2. The registration process validates function signatures
                // 3. The pointer comes from a known-safe source (libloading or static registration)
                let c_func: extern "C" fn(
                    *mut std::ffi::c_void,
                    tails_abi::NativeValue,
                    *const tails_abi::NativeValue,
                    i32,
                ) -> tails_abi::NativeValue = unsafe { std::mem::transmute(func_ptr) };

                // Convert this value to NativeValue
                let native_this = match this {
                    Value::NativeObject(obj_id) => tails_abi::NativeValue {
                        tag: 5,
                        data: obj_id.0 as u64,
                    },
                    Value::Object(_) => tails_abi::NativeValue { tag: 5, data: 0 },
                    Value::String(s) => tails_abi::string(s),
                    Value::Cons(c) => {
                        let flat = c.flatten();
                        tails_abi::string(&flat)
                    }
                    Value::Integer(n) => tails_abi::integer(*n),
                    Value::Float(n) => tails_abi::number(*n),
                    Value::Boolean(b) => tails_abi::boolean(*b),
                    Value::Null => tails_abi::null(),
                    Value::Undefined => tails_abi::undefined(),
                    _ => tails_abi::undefined(),
                };

                // Convert args to NativeValue array
                let native_args: Vec<tails_abi::NativeValue> = args
                    .iter()
                    .map(|v| match v {
                        Value::String(s) => tails_abi::string(s),
                        Value::Cons(c) => {
                            let flat = c.flatten();
                            tails_abi::string(&flat)
                        }
                        Value::Integer(n) => tails_abi::integer(*n),
                        Value::Float(n) => tails_abi::number(*n),
                        Value::Boolean(b) => tails_abi::boolean(*b),
                        Value::Null => tails_abi::null(),
                        Value::Undefined => tails_abi::undefined(),
                        Value::NativeObject(obj_id) => tails_abi::NativeValue {
                            tag: 5,
                            data: obj_id.0 as u64,
                        },
                        Value::Array(arr_idx) => {
                            if let HeapValue::Array(arr) = &self.heap[*arr_idx] {
                                let json_val = arr
                                    .elements
                                    .iter()
                                    .map(|v| self.value_to_json(v))
                                    .collect::<Vec<_>>();
                                let sv = simd_json::OwnedValue::Array(Box::new(json_val));
                                tails_abi::store_handle(sv)
                            } else {
                                tails_abi::null()
                            }
                        }
                        Value::Object(obj_idx) => {
                            if let HeapValue::Object(obj) = &self.heap[*obj_idx] {
                                let mut json_obj = simd_json::value::owned::Object::new();
                                for (k, v) in &obj.properties {
                                    json_obj.insert(k.to_string(), self.value_to_json(v));
                                }
                                let sv = simd_json::OwnedValue::Object(Box::new(json_obj));
                                tails_abi::store_handle(sv)
                            } else {
                                tails_abi::null()
                            }
                        }
                        _ => tails_abi::undefined(),
                    })
                    .collect();

                // Call the C ABI function
                let result = c_func(
                    std::ptr::null_mut(),
                    native_this,
                    native_args.as_ptr(),
                    native_args.len() as i32,
                );

                // Convert NativeValue back to interpreter Value
                match result.tag {
                    0 => Ok(Value::Undefined),
                    1 => Ok(Value::Null),
                    2 => Ok(Value::Boolean(result.data != 0)),
                    3 => Ok(Value::Float(f64::from_bits(result.data))),
                    4 => {
                        let s = tails_abi::get_string(result);
                        Ok(Value::from_string(s.into()))
                    }
                    5 => {
                        // Native object - create NativeObject value with the ID
                        let obj_id = result.data as u32;

                        // Look up class methods from the registry
                        // The constructor function name tells us the class name
                        // We need to find the constructor name from the dynamic_native_fns index
                        let class_name = self.find_class_name_for_native(idx);
                        if let Some(class_name) = class_name {
                            if let Some(methods) = self.native_class_registry.get(&class_name) {
                                self.native_object_methods.insert(obj_id, methods.clone());
                            }
                        }

                        Ok(Value::NativeObject(crate::objects::NativeObjectId(obj_id)))
                    }
                    _ => Ok(Value::Undefined),
                }
            } else {
                Err(Error::RuntimeError(format!(
                    "Unknown native function index: {}",
                    idx
                )))
            }
        }
    }

    pub(crate) fn find_native_prototype(&self, native_idx: usize) -> Option<usize> {
        let ctor_name = match native_idx {
            72 => wk::ERROR,
            73 => wk::TYPE_ERROR,
            74 => wk::REFERENCE_ERROR,
            75 => wk::SYNTAX_ERROR,
            76 => wk::RANGE_ERROR,
            170 => return self.date_proto_idx,
            214 => return self.regexp_proto_idx,
            312 => {
                // EventEmitter - search module registry for events module prototype
                if let Some(events_props) = self.module_registry.get(wk::MOD_EVENTS) {
                    if let Some(Value::Object(proto_idx)) = events_props.get(wk::PROTOTYPE) {
                        return Some(*proto_idx);
                    }
                }
                return None;
            }
            _ => return None,
        };
        for (i, hv) in self.heap.iter().enumerate() {
            if let HeapValue::Object(obj) = hv {
                if let Some(Value::String(name)) = obj.properties.get(wk::NAME) {
                    if **name == *ctor_name {
                        return Some(i);
                    }
                }
            }
        }
        None
    }

    fn find_class_name_for_native(&self, native_idx: usize) -> Option<String> {
        for props in self.module_registry.values() {
            for (func_name, value) in props {
                if let Value::NativeFunction(idx) = value {
                    if *idx == native_idx {
                        for class_name in self.native_class_registry.keys() {
                            if func_name == class_name {
                                return Some(class_name.clone());
                            }
                        }
                    }
                }
            }
        }
        None
    }

    fn value_to_json(&self, value: &Value) -> simd_json::OwnedValue {
        match value {
            Value::String(s) => simd_json::OwnedValue::String(s.to_string()),
            Value::Cons(c) => simd_json::OwnedValue::String(c.flatten()),
            Value::Integer(n) => simd_json::OwnedValue::Static(simd_json::StaticNode::I64(*n)),
            Value::Float(n) => simd_json::OwnedValue::Static(simd_json::StaticNode::F64(*n)),
            Value::Boolean(b) => simd_json::OwnedValue::Static(simd_json::StaticNode::Bool(*b)),
            Value::Null => simd_json::OwnedValue::Static(simd_json::StaticNode::Null),
            Value::Undefined => simd_json::OwnedValue::Static(simd_json::StaticNode::Null),
            Value::NativeObject(obj_id) => tails_abi::get_handle(obj_id.0 as u64)
                .unwrap_or(simd_json::OwnedValue::Static(simd_json::StaticNode::Null)),
            Value::Array(arr_idx) => {
                if let HeapValue::Array(arr) = &self.heap[*arr_idx] {
                    simd_json::OwnedValue::Array(Box::new(
                        arr.elements.iter().map(|v| self.value_to_json(v)).collect(),
                    ))
                } else {
                    simd_json::OwnedValue::Static(simd_json::StaticNode::Null)
                }
            }
            Value::Object(obj_idx) => {
                if let HeapValue::Object(obj) = &self.heap[*obj_idx] {
                    let mut json_obj = simd_json::value::owned::Object::new();
                    for (k, v) in &obj.properties {
                        json_obj.insert(k.to_string(), self.value_to_json(v));
                    }
                    simd_json::OwnedValue::Object(Box::new(json_obj))
                } else {
                    simd_json::OwnedValue::Static(simd_json::StaticNode::Null)
                }
            }
            _ => simd_json::OwnedValue::Static(simd_json::StaticNode::Null),
        }
    }
}
