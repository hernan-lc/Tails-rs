use super::*;
use crate::compiler::Instruction;
use crate::errors::{Error, Result};
use crate::objects::Value;

const ERR_STACK_UNDERFLOW: &str = "Stack underflow";
const PROXY_SET_TRAP: &str = "set";
const PROXY_DELETE_TRAP: &str = "deleteProperty";
const SETTER_PREFIX: &str = "__setter_";
const GETTER_PREFIX: &str = "__getter_";
const METHOD_PREFIX: &str = "__method_";
const TYPE_UNDEFINED: &str = "undefined";
const TYPE_OBJECT: &str = "object";
const TYPE_BOOLEAN: &str = "boolean";
const TYPE_NUMBER: &str = "number";
const TYPE_STRING: &str = "string";
const TYPE_BIGINT: &str = "bigint";
const TYPE_SYMBOL: &str = "symbol";
const TYPE_FUNCTION: &str = "function";

impl Interpreter {
    fn stack_pop(&mut self) -> Result<Value> {
        self.stack
            .pop()
            .ok_or_else(|| Error::RuntimeError(ERR_STACK_UNDERFLOW.into()))
    }

    fn pop_binary(&mut self) -> Result<(Value, Value)> {
        let right = self.stack_pop()?;
        let left = self.stack_pop()?;
        Ok((left, right))
    }

    pub(crate) fn exec_arithmetic(&mut self, instruction: &Instruction) -> Result<bool> {
        match instruction {
            Instruction::Add => {
                let (left, right) = self.pop_binary()?;
                let result = self.add(left, right)?;
                self.stack.push(result);
            }
            Instruction::Sub => {
                let (left, right) = self.pop_binary()?;
                let result = self.sub(left, right)?;
                self.stack.push(result);
            }
            Instruction::Mul => {
                let (left, right) = self.pop_binary()?;
                let result = self.mul(left, right)?;
                self.stack.push(result);
            }
            Instruction::Div => {
                let (left, right) = self.pop_binary()?;
                let result = self.div(left, right)?;
                self.stack.push(result);
            }
            Instruction::Mod => {
                let (left, right) = self.pop_binary()?;
                let result = self.modulo(left, right)?;
                self.stack.push(result);
            }
            Instruction::Power => {
                let (left, right) = self.pop_binary()?;
                let result = self.power(left, right)?;
                self.stack.push(result);
            }
            Instruction::Negate => {
                let value = self.stack_pop()?;
                let result = self.negate(value)?;
                self.stack.push(result);
            }
            Instruction::Not => {
                let value = self.stack_pop()?;
                let result = Value::Boolean(!self.is_truthy(&value));
                self.stack.push(result);
            }
            Instruction::BitNot => {
                let value = self.stack_pop()?;
                let num = self.to_number(&value)?;
                self.stack.push(Value::Integer(!(num as i64)));
            }
            Instruction::UnaryPlus => {
                let value = self.stack_pop()?;
                let num = self.to_number(&value)?;
                if num.fract() == 0.0 && num.abs() < i64::MAX as f64 {
                    self.stack.push(Value::Integer(num as i64));
                } else {
                    self.stack.push(Value::Float(num));
                }
            }
            Instruction::BitAnd => {
                let (left, right) = self.pop_binary()?;
                let l = self.to_number(&left)? as i64;
                let r = self.to_number(&right)? as i64;
                self.stack.push(Value::Integer(l & r));
            }
            Instruction::BitOr => {
                let (left, right) = self.pop_binary()?;
                let l = self.to_number(&left)? as i64;
                let r = self.to_number(&right)? as i64;
                self.stack.push(Value::Integer(l | r));
            }
            Instruction::BitXor => {
                let (left, right) = self.pop_binary()?;
                let l = self.to_number(&left)? as i64;
                let r = self.to_number(&right)? as i64;
                self.stack.push(Value::Integer(l ^ r));
            }
            Instruction::ShiftLeft => {
                let (left, right) = self.pop_binary()?;
                let l = self.to_number(&left)? as i64;
                let r = self.to_number(&right)? as u32;
                self.stack.push(Value::Integer(l << r));
            }
            Instruction::ShiftRight => {
                let (left, right) = self.pop_binary()?;
                let l = self.to_number(&left)? as i64;
                let r = self.to_number(&right)? as u32;
                self.stack.push(Value::Integer(l >> r));
            }
            Instruction::Void => {
                self.stack_pop()?;
                self.stack.push(Value::Undefined);
            }
            Instruction::TypeOf => {
                let value = self.stack_pop()?;
                let type_str = match &value {
                    Value::Undefined => TYPE_UNDEFINED,
                    Value::Null => TYPE_OBJECT,
                    Value::Boolean(_) => TYPE_BOOLEAN,
                    Value::Integer(_) | Value::Float(_) => TYPE_NUMBER,
                    Value::String(_) | Value::Cons(_) => TYPE_STRING,
                    Value::BigInt(_) => TYPE_BIGINT,
                    Value::Symbol(_) => TYPE_SYMBOL,
                    Value::Function(_) | Value::NativeFunction(_) => TYPE_FUNCTION,
                    Value::Object(_)
                    | Value::Array(_)
                    | Value::Promise(_)
                    | Value::Proxy(_)
                    | Value::Generator(_)
                    | Value::TypedArray(_)
                    | Value::Map(_)
                    | Value::Set(_)
                    | Value::WeakMap(_)
                    | Value::WeakSet(_)
                    | Value::Buffer(_) => TYPE_OBJECT,
                    Value::Date(_) | Value::RegExp(_) => TYPE_OBJECT,
                    Value::NativeObject(_) => TYPE_OBJECT,
                };
                self.stack.push(Value::String(type_str.to_string()));
            }
            Instruction::ToString => {
                let value = self.stack_pop()?;
                let s = self.value_to_string_raw(&value);
                self.stack.push(Value::String(s));
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    pub(crate) fn exec_comparison(&mut self, instruction: &Instruction) -> Result<bool> {
        match instruction {
            Instruction::Eq => {
                let (left, right) = self.pop_binary()?;
                self.stack
                    .push(Value::Boolean(self.is_equal(&left, &right)));
            }
            Instruction::StrictEq => {
                let (left, right) = self.pop_binary()?;
                let eq = match (&left, &right) {
                    (Value::Float(a), Value::Float(b)) => a == b && !a.is_nan() && !b.is_nan(),
                    (Value::Integer(a), Value::Float(b)) => *a as f64 == *b && !b.is_nan(),
                    (Value::Float(a), Value::Integer(b)) => *a == *b as f64 && !a.is_nan(),
                    _ => left == right,
                };
                self.stack.push(Value::Boolean(eq));
            }
            Instruction::NotEqual => {
                let (left, right) = self.pop_binary()?;
                self.stack
                    .push(Value::Boolean(!self.is_equal(&left, &right)));
            }
            Instruction::StrictNotEqual => {
                let (left, right) = self.pop_binary()?;
                self.stack.push(Value::Boolean(left != right));
            }
            Instruction::Less => {
                let (left, right) = self.pop_binary()?;
                let result = self.less_than(&left, &right)?;
                self.stack.push(Value::Boolean(result));
            }
            Instruction::Greater => {
                let (left, right) = self.pop_binary()?;
                let result = self.greater_than(&left, &right)?;
                self.stack.push(Value::Boolean(result));
            }
            Instruction::LessEqual => {
                let (left, right) = self.pop_binary()?;
                let result = self.less_than_or_equal(&left, &right)?;
                self.stack.push(Value::Boolean(result));
            }
            Instruction::GreaterEqual => {
                let (left, right) = self.pop_binary()?;
                let result = self.greater_than_or_equal(&left, &right)?;
                self.stack.push(Value::Boolean(result));
            }
            Instruction::And => {
                let (left, right) = self.pop_binary()?;
                if self.is_truthy(&left) {
                    self.stack.push(right);
                } else {
                    self.stack.push(left);
                }
            }
            Instruction::Or => {
                let (left, right) = self.pop_binary()?;
                if self.is_truthy(&left) {
                    self.stack.push(left);
                } else {
                    self.stack.push(right);
                }
            }
            Instruction::InstanceOf => {
                let (left, right) = self.pop_binary()?;
                let result = self.instanceof_check(&left, &right)?;
                self.stack.push(result);
            }
            Instruction::In => {
                let (left, right) = self.pop_binary()?;
                let result = self.in_check_mut(&left, &right)?;
                self.stack.push(result);
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    pub(crate) fn exec_property_ops(&mut self, instruction: &Instruction) -> Result<bool> {
        match instruction {
            Instruction::NewObject => {
                let heap_idx = self
                    .gc
                    .allocate(&mut self.heap, HeapValue::Object(JsObject::new()));
                self.stack.push(Value::Object(heap_idx));
            }
            Instruction::SetProperty => {
                let value = self.stack_pop()?;
                let key = self.stack_pop()?;
                let object = self.stack_pop()?;

                let key_owned;
                let key_str: &str = match &key {
                    Value::String(s) => s.as_str(),
                    Value::Cons(c) => {
                        key_owned = c.flatten();
                        &key_owned
                    }
                    _ => "",
                };
                if key_str.is_empty() && !matches!(&key, Value::String(_)) {
                } else {
                    match &object {
                        Value::Proxy(proxy_idx) => {
                            if let HeapValue::Proxy(proxy) = &self.heap[*proxy_idx] {
                                let handler = proxy.handler.clone();
                                let target = proxy.target.clone();
                                let trap = self.get_property(
                                    &handler,
                                    &Value::String(PROXY_SET_TRAP.to_string()),
                                );
                                if let Ok(Value::Function(_)) | Ok(Value::NativeFunction(_)) = &trap
                                {
                                    let trap_result = self.call_value(
                                        &trap?,
                                        &handler,
                                        &[target, key.clone(), value, object.clone()],
                                    );
                                    trap_result?;
                                } else {
                                    if let Value::Object(target_obj_idx) = &target {
                                        if let HeapValue::Object(obj) =
                                            &mut self.heap[*target_obj_idx]
                                        {
                                            obj.properties.insert(key_str.to_string(), value);
                                        }
                                    }
                                }
                            }
                        }
                        Value::Object(obj_idx) => {
                            if let HeapValue::Object(obj) = &mut self.heap[*obj_idx] {
                                let setter_key = format!("{}{}", SETTER_PREFIX, key_str);
                                if let Some(setter_val) = obj.properties.get(&setter_key).cloned() {
                                    let _ = obj;
                                    self.call_value(&setter_val, &object, &[value])?;
                                } else {
                                    obj.properties.insert(key_str.to_string(), value);
                                }
                            }
                        }
                        Value::Function(func_idx) => {
                            if let HeapValue::Function(f) = &mut self.heap[*func_idx] {
                                f.properties.insert(key_str.to_string(), value);
                            }
                        }
                        _ => {}
                    }
                }
                self.stack.push(object);
            }
            Instruction::GetProperty => {
                let key = self.stack_pop()?;
                let object = self.stack_pop()?;
                let result = self.get_property(&object, &key)?;
                self.stack.push(result);
            }
            Instruction::OptionalGetProperty => {
                let key = self.stack_pop()?;
                let object = self.stack_pop()?;
                if matches!(object, Value::Undefined | Value::Null) {
                    self.stack.push(Value::Undefined);
                } else {
                    let result = self.get_property(&object, &key)?;
                    self.stack.push(result);
                }
            }
            Instruction::OptionalCall(argc) => {
                let mut args = Vec::new();
                for _ in 0..*argc {
                    args.push(self.stack_pop()?);
                }
                args.reverse();
                let this = self.stack_pop()?;
                let callee = self.stack_pop()?;
                if matches!(callee, Value::Undefined | Value::Null) {
                    self.stack.push(Value::Undefined);
                } else {
                    let result = self.call_value(&callee, &this, &args)?;
                    self.stack.push(result);
                }
            }
            Instruction::NullishCoalescing => {
                let (left, right) = self.pop_binary()?;
                if matches!(left, Value::Undefined | Value::Null) {
                    self.stack.push(right);
                } else {
                    self.stack.push(left);
                }
            }
            Instruction::NewArray(size) => {
                let mut elements = Vec::new();
                for _ in 0..*size {
                    elements.push(self.stack.pop().unwrap_or(Value::Undefined));
                }
                let heap_idx = self
                    .gc
                    .allocate(&mut self.heap, HeapValue::Array(JsArray { elements }));
                self.stack.push(Value::Array(heap_idx));
            }
            Instruction::SpreadArray => {
                let source = self.stack_pop()?;
                let target = self.stack_pop()?;
                if let Value::Array(target_idx) = target {
                    if let Value::Array(source_idx) = source {
                        if let HeapValue::Array(source_arr) = &self.heap[source_idx] {
                            let elements: Vec<Value> = source_arr.elements.clone();
                            if let HeapValue::Array(target_arr) = &mut self.heap[target_idx] {
                                for elem in elements {
                                    target_arr.elements.push(elem);
                                }
                            }
                        }
                    }
                }
                self.stack.push(target);
            }
            Instruction::SpreadObject => {
                let source = self.stack_pop()?;
                let target = self.stack_pop()?;
                if let Value::Object(target_idx) = target {
                    if let Value::Object(source_idx) = source {
                        let props: Vec<(String, Value)> =
                            if let HeapValue::Object(source_obj) = &self.heap[source_idx] {
                                source_obj
                                    .properties
                                    .iter()
                                    .map(|(k, v)| (k.to_string(), v.clone()))
                                    .collect()
                            } else {
                                Vec::new()
                            };
                        if let HeapValue::Object(target_obj) = &mut self.heap[target_idx] {
                            for (k, v) in props {
                                target_obj.properties.insert(k, v);
                            }
                        }
                    }
                }
                self.stack.push(target);
            }
            Instruction::ArrayPush => {
                let value = self.stack_pop()?;
                let array = self.stack_pop()?;
                if let Value::Array(arr_idx) = array {
                    if let HeapValue::Array(arr) = &mut self.heap[arr_idx] {
                        arr.elements.push(value.clone());
                    }
                }
                self.stack.push(array);
            }
            Instruction::GetKeys => {
                let obj = self.stack_pop()?;
                let keys: Vec<Value> = match &obj {
                    Value::Object(idx) => {
                        if let HeapValue::Object(o) = &self.heap[*idx] {
                            o.properties
                                .keys()
                                .filter(|k| {
                                    !k.starts_with(GETTER_PREFIX)
                                        && !k.starts_with(SETTER_PREFIX)
                                        && !k.starts_with(METHOD_PREFIX)
                                })
                                .map(|k| Value::String(k.to_string()))
                                .collect()
                        } else {
                            vec![]
                        }
                    }
                    Value::Array(idx) => {
                        if let HeapValue::Array(arr) = &self.heap[*idx] {
                            (0..arr.elements.len())
                                .map(|i| Value::Float(i as f64))
                                .collect()
                        } else {
                            vec![]
                        }
                    }
                    Value::String(s) => (0..s.len()).map(|i| Value::Float(i as f64)).collect(),
                    _ => vec![],
                };
                let heap_idx = self
                    .gc
                    .allocate(&mut self.heap, HeapValue::Array(JsArray { elements: keys }));
                self.stack.push(Value::Array(heap_idx));
            }
            Instruction::Delete => {
                let key = self.stack_pop()?;
                let object = self.stack_pop()?;
                match &object {
                    Value::Proxy(proxy_idx) => {
                        if let HeapValue::Proxy(proxy) = &self.heap[*proxy_idx] {
                            let handler = proxy.handler.clone();
                            let target = proxy.target.clone();
                            let trap = self.get_property(
                                &handler,
                                &Value::String(PROXY_DELETE_TRAP.to_string()),
                            );
                            if let Ok(Value::Function(_)) | Ok(Value::NativeFunction(_)) = &trap {
                                let trap_result =
                                    self.call_value(&trap?, &handler, &[target, key.clone()]);
                                match trap_result {
                                    Ok(v) => self.stack.push(v),
                                    Err(e) => return Err(e),
                                }
                            } else {
                                let result = self.delete_property(&target, &key);
                                self.stack.push(result);
                            }
                        } else {
                            let result = self.delete_property(&object, &key);
                            self.stack.push(result);
                        }
                    }
                    _ => {
                        let result = self.delete_property(&object, &key);
                        self.stack.push(result);
                    }
                }
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}
