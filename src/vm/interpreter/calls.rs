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

                    self.call_stack.push(CallFrame {
                        return_address,
                        base_pointer,
                        closure_var_count: closure_count,
                        func_heap_idx: Some(*func_idx),
                        this_value: Some(this.clone()),
                        is_construct: false,
                        source_name: self.current_module_path.clone(),
                        generator_heap_idx: None,
                        source_line: self.current_source_line(0),
                    });

                    for closure_var in &f_clone.closure {
                        self.stack.push(closure_var.clone());
                    }
                    for arg in args {
                        self.stack.push(arg.clone());
                    }

                    let result = if let Some(module) = func_module {
                        self.execute_from(&module, f_clone.bytecode_index)
                    } else {
                        Ok(Value::Undefined)
                    };

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
            Err(Error::RuntimeError(format!(
                "Unknown native function index: {}",
                idx
            )))
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
}
