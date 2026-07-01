use super::*;
use crate::errors::{Error, Result};
use crate::objects::Value;
use crate::runtime_env::native_fns::NATIVE_TABLE;
use std::rc::Rc;

impl Interpreter {
    pub fn call_value(&mut self, callee: &Value, this: &Value, args: &[Value]) -> Result<Value> {
        match callee {
            Value::Function(func_idx) => {
                if let HeapValue::Function(f) = &self.heap[*func_idx] {
                    let f_clone = f.clone();

                    if f_clone.bytecode_index == usize::MAX {
                        if let Some(Value::Promise(promise_idx)) = f_clone.closure.first() {
                            let value = args.first().cloned().unwrap_or(Value::Undefined);
                            if f_clone.name.as_deref() == Some("resolve") {
                                self.resolve_promise(*promise_idx, value);
                            } else if f_clone.name.as_deref() == Some("reject") {
                                self.reject_promise(*promise_idx, value);
                            }
                            return Ok(Value::Undefined);
                        }
                    }

                    let func_module: Option<Rc<CompiledModule>> = f_clone
                        .owner_module
                        .clone()
                        .or_else(|| self.current_module.clone());
                    let return_address = func_module
                        .as_ref()
                        .map(|m| m.instructions.len())
                        .unwrap_or(0);
                    let base_pointer = self.stack.len();
                    let closure_count = f_clone.closure.len();

                    let saved_mg = self.module_globals.take();
                    if let Some(ref scope) = f_clone.module_scope {
                        self.module_globals = Some((**scope).clone());
                    }

                    let saved_module = self.current_module.clone();
                    let saved_path = self.current_module_path.clone();
                    if let Some(ref mod_ref) = func_module {
                        self.current_module = Some(mod_ref.clone());
                    }
                    if f_clone.source_file.is_some() {
                        self.current_module_path = f_clone.source_file.clone();
                    }

                    let this_for_frame = if f_clone.is_arrow {
                        f_clone
                            .captured_this
                            .clone()
                            .unwrap_or_else(|| this.clone())
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
                        source_name: f_clone
                            .source_file
                            .clone()
                            .or_else(|| self.current_module_path.clone()),
                        generator_heap_idx: None,
                        source_line: f_clone.source_line,
                        source_col: None,
                    });

                    for closure_var in &f_clone.closure {
                        self.stack.push(closure_var.clone());
                    }
                    if f_clone.rest_param.is_some() {
                        let param_count = f_clone.params.len();
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

                    let result = if let Some(module) = func_module {
                        self.execute_from(&module, f_clone.bytecode_index)
                    } else {
                        Ok(Value::Undefined)
                    };

                    self.current_module = saved_module;
                    self.current_module_path = saved_path;
                    self.module_globals = saved_mg;
                    result
                } else {
                    Err(self.err_at_location(Error::TypeError("Not a function".into())))
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
                        Value::Integer(n) => tails_abi::integer(*n),
                        Value::Float(n) => tails_abi::number(*n),
                        Value::Boolean(b) => tails_abi::boolean(*b),
                        Value::Null => tails_abi::null(),
                        Value::Undefined => tails_abi::undefined(),
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
                        Ok(Value::String(s))
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
            72 => "Error",
            73 => "TypeError",
            74 => "ReferenceError",
            75 => "SyntaxError",
            76 => "RangeError",
            170 => return self.date_proto_idx,
            214 => return self.regexp_proto_idx,
            _ => return None,
        };
        for (i, hv) in self.heap.iter().enumerate() {
            if let HeapValue::Object(obj) = hv {
                if let Some(Value::String(name)) = obj.properties.get("name") {
                    if name == ctor_name {
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
}
