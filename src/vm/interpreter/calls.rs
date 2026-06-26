use super::*;
use crate::errors::{Error, Result};
use crate::objects::Value;
use crate::runtime_env::native_fns::NATIVE_TABLE;

impl Interpreter {
    pub fn call_value(&mut self, callee: &Value, this: &Value, args: &[Value]) -> Result<Value> {
        eprintln!("call_value: callee={:?} args={}", callee, args.len());
        match callee {
            Value::Function(func_idx) => {
                if let HeapValue::Function(f) = &self.heap[*func_idx] {
                    let f_clone = f.clone();
                    let return_address = self.current_module.as_ref()
                        .map(|m| m.instructions.len())
                        .unwrap_or(0);
                    let base_pointer = self.stack.len();
                    let closure_count = f_clone.closure.len();

                    self.call_stack.push(CallFrame {
                        return_address,
                        base_pointer,
                        closure_var_count: closure_count,
                        func_heap_idx: Some(*func_idx),
                        this_value: Some(this.clone()),
                        is_construct: false,
                    });

                    for closure_var in &f_clone.closure {
                        self.stack.push(closure_var.clone());
                    }
                    for arg in args {
                        self.stack.push(arg.clone());
                    }

                    if let Some(module) = self.current_module.clone() {
                        self.execute_from(&module, f_clone.bytecode_index)
                    } else {
                        Ok(Value::Undefined)
                    }
                } else {
                    Err(Error::TypeError("Not a function".into()))
                }
            }
            Value::NativeFunction(native_idx) => {
                self.call_native(*native_idx, this, args)
            }
            Value::Proxy(proxy_idx) => {
                if let HeapValue::Proxy(proxy) = &self.heap[*proxy_idx] {
                    let handler = proxy.handler.clone();
                    let target = proxy.target.clone();
                    let arr_idx = self.gc.allocate(&mut self.heap, HeapValue::Array(JsArray { elements: args.to_vec() }));
                    self.call_proxy_trap(&handler, "apply", &[target, this.clone(), Value::Array(arr_idx)])
                } else {
                    Err(Error::TypeError(format!("{} is not a function", self.value_to_string(callee))))
                }
            }
            _ => Err(Error::TypeError(format!("{} is not a function", self.value_to_string(callee)))),
        }
    }

    pub(crate) fn call_native(&mut self, idx: usize, this: &Value, args: &[Value]) -> Result<Value> {
        if idx < NATIVE_TABLE.len() {
            NATIVE_TABLE[idx](self, this, args)
        } else {
            Err(Error::RuntimeError(format!("Unknown native function index: {}", idx)))
        }
    }

    pub(crate) fn find_native_prototype(&self, native_idx: usize) -> Option<usize> {
        let ctor_name = match native_idx {
            72 => "Error",
            73 => "TypeError",
            74 => "ReferenceError",
            75 => "SyntaxError",
            76 => "RangeError",
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
