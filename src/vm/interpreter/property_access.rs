use crate::errors::{Error, Result};
use crate::objects::Value;
use crate::objects::js_promise::PromiseState;
use super::{Interpreter, HeapValue};

impl Interpreter {
    pub(super) fn get_property(&mut self, object: &Value, key: &Value) -> Result<Value> {
        self.get_property_with_this(object, key, object)
    }

    pub(crate) fn get_property_with_this(&mut self, object: &Value, key: &Value, this: &Value) -> Result<Value> {
        match object {
            Value::Null | Value::Undefined => {
                return Err(Error::TypeError(format!(
                    "Cannot read properties of {} (reading '{}')",
                    self.value_to_string(object),
                    self.value_to_string(key)
                )));
            }
            Value::Object(obj_idx) => {
                if let HeapValue::Object(obj) = &self.heap[*obj_idx] {
                    if let Value::String(key_str) = key {
                        if let Some(val) = obj.properties.get(key_str) {
                            return Ok(val.clone());
                        }
                        let getter_key = format!("__getter_{}", key_str);
                        if let Some(getter_val) = obj.properties.get(&getter_key).cloned() {
                            return self.call_value(&getter_val, this, &[]);
                        }
                        if let Some(proto_idx) = obj.prototype {
                            let proto_val = Value::Object(proto_idx);
                            return self.get_property_with_this(&proto_val, key, this);
                        }
                    }
                }
            }
            Value::Array(arr_idx) => {
                if let HeapValue::Array(arr) = &self.heap[*arr_idx] {
                    match key {
                        Value::String(key_str) => {
                            if key_str == "length" {
                                return Ok(Value::Float(arr.elements.len() as f64));
                            }
                            if let Ok(index) = key_str.parse::<usize>() {
                                return Ok(arr.elements.get(index).cloned().unwrap_or(Value::Undefined));
                            }
                            return self.get_array_method(key_str);
                        }
                        Value::Integer(index) => {
                            let idx = *index as usize;
                            return Ok(arr.elements.get(idx).cloned().unwrap_or(Value::Undefined));
                        }
                        Value::Float(f) => {
                            let idx = *f as usize;
                            return Ok(arr.elements.get(idx).cloned().unwrap_or(Value::Undefined));
                        }
                        _ => {}
                    }
                }
            }
            Value::String(s) => {
                return self.get_property_from_primitive_string(s, key);
            }
            Value::Integer(_) | Value::Float(_) => {
                return self.get_property_from_primitive_number(object, key);
            }
            Value::Boolean(_) => {
                return self.get_property_from_primitive_boolean(object, key);
            }
            Value::Function(func_idx) => {
                if let Value::String(key_str) = key {
                    if key_str == "prototype" {
                        if let HeapValue::Function(f) = &self.heap[*func_idx] {
                            if let Some(proto_idx) = f.prototype {
                                return Ok(Value::Object(proto_idx));
                            }
                        }
                    }
                    if let HeapValue::Function(f) = &self.heap[*func_idx] {
                        if let Some(val) = f.properties.get(key_str) {
                            return Ok(val.clone());
                        }
                    }
                }
            }
            Value::Promise(promise_idx) => {
                if let Value::String(key_str) = key {
                    match key_str.as_str() {
                        "then" => return Ok(Value::NativeFunction(78)),
                        "catch" => return Ok(Value::NativeFunction(79)),
                        "finally" => return Ok(Value::NativeFunction(80)),
                        "state" => {
                            if let HeapValue::Promise(p) = &self.heap[*promise_idx] {
                                return Ok(Value::String(format!("{:?}", p.state)));
                            }
                        }
                        "value" => {
                            if let HeapValue::Promise(p) = &self.heap[*promise_idx] {
                                if let PromiseState::Fulfilled(v) = &p.state {
                                    return Ok(v.clone());
                                }
                            }
                        }
                        "reason" => {
                            if let HeapValue::Promise(p) = &self.heap[*promise_idx] {
                                if let PromiseState::Rejected(r) = &p.state {
                                    return Ok(r.clone());
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            Value::NativeFunction(idx) => {
                if let Value::String(key_str) = key {
                    if *idx == 77 {
                        match key_str.as_str() {
                            "resolve" => return Ok(Value::NativeFunction(81)),
                            "reject" => return Ok(Value::NativeFunction(82)),
                            "all" => return Ok(Value::NativeFunction(83)),
                            "race" => return Ok(Value::NativeFunction(84)),
                            _ => {}
                        }
                    }
                }
            }
            Value::Proxy(proxy_idx) => {
                if let HeapValue::Proxy(proxy) = &self.heap[*proxy_idx] {
                    let handler = proxy.handler.clone();
                    let target = proxy.target.clone();
                    let trap = self.get_property(&handler, &Value::String("get".to_string()));
                    match &trap {
                        Ok(Value::Function(_)) | Ok(Value::NativeFunction(_)) => {
                            let trap_val = trap.unwrap();
                            let trap_result = self.call_value(&trap_val, &handler, &[target, key.clone(), this.clone()]);
                            match trap_result {
                                Ok(v) => return Ok(v),
                                Err(_) => {}
                            }
                        }
                        _ => {
                            return self.get_property_with_this(&target, key, this);
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(Value::Undefined)
    }

    pub(super) fn get_array_method(&self, name: &str) -> Result<Value> {
        let idx = match name {
            "push" => 31,
            "pop" => 32,
            "shift" => 33,
            "unshift" => 34,
            "slice" => 35,
            "splice" => 36,
            "indexOf" => 37,
            "includes" => 38,
            "find" => 39,
            "findIndex" => 40,
            "map" => 41,
            "filter" => 42,
            "reduce" => 43,
            "forEach" => 44,
            "some" => 45,
            "every" => 46,
            "join" => 47,
            "reverse" => 48,
            "sort" => 49,
            "concat" => 50,
            "flat" => 51,
            _ => return Ok(Value::Undefined),
        };
        Ok(Value::NativeFunction(idx))
    }

    pub(super) fn get_property_from_primitive_string(&self, s: &str, key: &Value) -> Result<Value> {
        if let Value::String(key_str) = key {
            match key_str.as_str() {
                "length" => return Ok(Value::Float(s.len() as f64)),
                _ => {}
            }
            return self.get_string_method(key_str);
        }
        Ok(Value::Undefined)
    }

    pub(super) fn get_string_method(&self, name: &str) -> Result<Value> {
        let idx = match name {
            "charAt" => 52,
            "charCodeAt" => 53,
            "slice" => 54,
            "substring" => 55,
            "indexOf" => 56,
            "includes" => 57,
            "replace" => 58,
            "split" => 59,
            "trim" => 60,
            "toLowerCase" => 61,
            "toUpperCase" => 62,
            "startsWith" => 63,
            "endsWith" => 64,
            "repeat" => 65,
            "padStart" => 66,
            "padEnd" => 67,
            _ => return Ok(Value::Undefined),
        };
        Ok(Value::NativeFunction(idx))
    }

    pub(super) fn get_property_from_primitive_number(&self, _n: &Value, key: &Value) -> Result<Value> {
        if let Value::String(key_str) = key {
            match key_str.as_str() {
                "toString" | "toFixed" | "valueOf" => {
                    return Ok(self.make_native_number_method(key_str));
                }
                _ => {}
            }
        }
        Ok(Value::Undefined)
    }

    pub(super) fn get_property_from_primitive_boolean(&self, _b: &Value, key: &Value) -> Result<Value> {
        if let Value::String(key_str) = key {
            match key_str.as_str() {
                "toString" | "valueOf" => {
                    return Ok(self.make_native_boolean_method(key_str));
                }
                _ => {}
            }
        }
        Ok(Value::Undefined)
    }

    pub(super) fn make_native_number_method(&self, _name: &str) -> Value {
        Value::NativeFunction(0)
    }

    pub(super) fn make_native_boolean_method(&self, _name: &str) -> Value {
        Value::NativeFunction(0)
    }

    pub(crate) fn delete_property(&mut self, object: &Value, key: &Value) -> Value {
        match object {
            Value::Object(obj_idx) => {
                if let HeapValue::Object(obj) = &mut self.heap[*obj_idx] {
                    if let Value::String(key_str) = key {
                        if obj.properties.remove(key_str).is_some() {
                            return Value::Boolean(true);
                        }
                    }
                }
                Value::Boolean(false)
            }
            Value::Array(arr_idx) => {
                if let Value::String(key_str) = key {
                    if let Ok(index) = key_str.parse::<usize>() {
                        if let HeapValue::Array(arr) = &mut self.heap[*arr_idx] {
                            if index < arr.elements.len() {
                                arr.elements[index] = Value::Undefined;
                                return Value::Boolean(true);
                            }
                        }
                    }
                }
                Value::Boolean(false)
            }
            _ => Value::Boolean(true),
        }
    }

    pub(super) fn instanceof_check(&mut self, left: &Value, right: &Value) -> Result<Value> {
        let proto_key = Value::String("prototype".to_string());
        let right_proto = match self.get_property(right, &proto_key) {
            Ok(val) => val,
            Err(_) => return Ok(Value::Boolean(false)),
        };

        let proto_idx = match &right_proto {
            Value::Object(idx) => *idx,
            _ => return Ok(Value::Boolean(false)),
        };

        let mut current = left.clone();
        loop {
            match &current {
                Value::Object(obj_idx) => {
                    if let HeapValue::Object(obj) = &self.heap[*obj_idx] {
                        if obj.prototype == Some(proto_idx) {
                            return Ok(Value::Boolean(true));
                        }
                        if let Some(parent_idx) = obj.prototype {
                            current = Value::Object(parent_idx);
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                Value::Array(_arr_idx) => {
                    break;
                }
                _ => break,
            }
        }
        Ok(Value::Boolean(false))
    }

    pub(crate) fn in_check_mut(&mut self, key: &Value, object: &Value) -> Result<Value> {
        match object {
            Value::Object(obj_idx) => {
                if let HeapValue::Object(obj) = &self.heap[*obj_idx] {
                    if let Value::String(key_str) = key {
                        if obj.properties.contains_key(key_str) {
                            return Ok(Value::Boolean(true));
                        }
                        if let Some(proto_idx) = obj.prototype {
                            let proto_val = Value::Object(proto_idx);
                            return self.in_check_mut(key, &proto_val);
                        }
                    }
                }
                Ok(Value::Boolean(false))
            }
            Value::Array(arr_idx) => {
                if let HeapValue::Array(arr) = &self.heap[*arr_idx] {
                    if let Value::String(key_str) = key {
                        if key_str == "length" {
                            return Ok(Value::Boolean(true));
                        }
                        if let Ok(index) = key_str.parse::<usize>() {
                            return Ok(Value::Boolean(index < arr.elements.len()));
                        }
                    }
                }
                Ok(Value::Boolean(false))
            }
            Value::String(s) => {
                if let Value::String(key_str) = key {
                    if key_str == "length" {
                        return Ok(Value::Boolean(true));
                    }
                    if let Ok(index) = key_str.parse::<usize>() {
                        return Ok(Value::Boolean(index < s.len()));
                    }
                }
                Ok(Value::Boolean(false))
            }
            Value::Proxy(proxy_idx) => {
                if let HeapValue::Proxy(proxy) = &self.heap[*proxy_idx] {
                    let handler = proxy.handler.clone();
                    let target = proxy.target.clone();
                    let trap = self.get_property(&handler, &Value::String("has".to_string()));
                    if let Ok(Value::Function(_)) | Ok(Value::NativeFunction(_)) = &trap {
                        let trap_result = self.call_value(&trap?, &handler, &[target, key.clone()]);
                        match trap_result {
                            Ok(v) => return Ok(v),
                            Err(_) => {}
                        }
                    } else {
                        return self.in_check_mut(key, &target);
                    }
                }
                Ok(Value::Boolean(false))
            }
            _ => Ok(Value::Boolean(false)),
        }
    }

    pub(crate) fn call_proxy_trap(&mut self, handler: &Value, trap_name: &str, args: &[Value]) -> Result<Value> {
        let trap = self.get_property(handler, &Value::String(trap_name.to_string()))?;
        if matches!(trap, Value::Undefined) {
            return Err(Error::RuntimeError(format!("Proxy has no '{}' trap", trap_name)));
        }
        self.call_value(&trap, handler, args)
    }
}
