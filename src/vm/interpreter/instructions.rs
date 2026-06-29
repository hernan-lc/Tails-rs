use super::*;
use crate::compiler::Instruction;
use crate::errors::{Error, Result};
use crate::objects::Value;

impl Interpreter {
    pub(crate) fn exec_load_store(
        &mut self,
        instruction: &Instruction,
        module: &CompiledModule,
    ) -> Result<bool> {
        match instruction {
            Instruction::LoadConst(idx) => {
                let value = module.constants[*idx as usize].clone();
                self.stack.push(value);
            }
            Instruction::LoadNull => {
                self.stack.push(Value::Null);
            }
            Instruction::LoadUndefined => {
                self.stack.push(Value::Undefined);
            }
            Instruction::LoadTrue => {
                self.stack.push(Value::Boolean(true));
            }
            Instruction::LoadFalse => {
                self.stack.push(Value::Boolean(false));
            }
            Instruction::LoadGlobal(name) => {
                let value = self.globals.get(name).cloned().or_else(|| {
                    self.module_globals
                        .as_ref()
                        .and_then(|mg| mg.get(name).cloned())
                });
                match value {
                    Some(v) => self.stack.push(v),
                    None => {
                        return Err(self.err_at_location(Error::ReferenceError(format!(
                            "{} is not defined",
                            name
                        ))))
                    }
                }
            }
            Instruction::LoadGlobalOrUndefined(name) => {
                let value = self
                    .globals
                    .get(name)
                    .cloned()
                    .or_else(|| {
                        self.module_globals
                            .as_ref()
                            .and_then(|mg| mg.get(name).cloned())
                    })
                    .unwrap_or(Value::Undefined);
                self.stack.push(value);
            }
            Instruction::TypeOfGlobal(name) => {
                let value = self
                    .globals
                    .get(name)
                    .cloned()
                    .or_else(|| {
                        self.module_globals
                            .as_ref()
                            .and_then(|mg| mg.get(name).cloned())
                    })
                    .unwrap_or(Value::Undefined);
                let type_str = match &value {
                    Value::Undefined => "undefined",
                    Value::Null => "object",
                    Value::Boolean(_) => "boolean",
                    Value::Integer(_) | Value::Float(_) => "number",
                    Value::String(_) => "string",
                    Value::BigInt(_) => "bigint",
                    Value::Symbol(_) => "symbol",
                    Value::Function(_) | Value::NativeFunction(_) => "function",
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
                    | Value::Buffer(_) => "object",
                    Value::Date(_) | Value::RegExp(_) => "object",
                };
                self.stack.push(Value::String(type_str.to_string()));
            }
            Instruction::StoreGlobal(name) => {
                let value = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                self.globals.insert(name.clone(), value);
            }
            Instruction::LoadLocal(slot) => {
                let base = self.call_stack.last().map(|f| f.base_pointer).unwrap_or(0);
                let idx = base + *slot as usize;
                let value = self.stack.get(idx).cloned().unwrap_or(Value::Undefined);
                self.stack.push(value);
            }
            Instruction::StoreLocal(slot) => {
                let value = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let base = self.call_stack.last().map(|f| f.base_pointer).unwrap_or(0);
                let idx = base + *slot as usize;
                if idx >= self.stack.len() {
                    self.stack.resize(idx + 1, Value::Undefined);
                }
                self.stack[idx] = value;
            }
            Instruction::Pop => {
                self.stack.pop();
            }
            Instruction::Dup => {
                let val = self.stack.last().cloned().unwrap_or(Value::Undefined);
                self.stack.push(val);
            }
            Instruction::Rot3Right => {
                let len = self.stack.len();
                if len >= 3 {
                    let a = self.stack[len - 3].clone();
                    let b = self.stack[len - 2].clone();
                    let c = self.stack[len - 1].clone();
                    self.stack[len - 3] = b;
                    self.stack[len - 2] = c;
                    self.stack[len - 1] = a;
                }
            }
            Instruction::LoadThis => {
                let this = self
                    .call_stack
                    .last()
                    .and_then(|f| f.this_value.clone())
                    .unwrap_or(Value::Undefined);
                self.stack.push(this);
            }
            Instruction::BlockEnter => {
                // Save the current stack length as the block scope base
                self.block_scope_stack.push(self.stack.len());
            }
            Instruction::BlockExit => {
                // Restore stack to the block scope base, but preserve the top value if any
                if let Some(block_base) = self.block_scope_stack.pop() {
                    if self.stack.len() > block_base {
                        // Save the top value (result of last expression in block)
                        let top_value = self.stack.pop().unwrap_or(Value::Undefined);
                        // Truncate stack to block base
                        self.stack.truncate(block_base);
                        // Push the saved value back
                        self.stack.push(top_value);
                    } else {
                        // Stack is at or below block base, just truncate
                        self.stack.truncate(block_base);
                    }
                }
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    pub(crate) fn exec_arithmetic(&mut self, instruction: &Instruction) -> Result<bool> {
        match instruction {
            Instruction::Add => {
                let right = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let left = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let result = self.add(left, right)?;
                self.stack.push(result);
            }
            Instruction::Sub => {
                let right = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let left = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let result = self.sub(left, right)?;
                self.stack.push(result);
            }
            Instruction::Mul => {
                let right = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let left = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let result = self.mul(left, right)?;
                self.stack.push(result);
            }
            Instruction::Div => {
                let right = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let left = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let result = self.div(left, right)?;
                self.stack.push(result);
            }
            Instruction::Mod => {
                let right = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let left = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let result = self.modulo(left, right)?;
                self.stack.push(result);
            }
            Instruction::Power => {
                let right = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let left = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let result = self.power(left, right)?;
                self.stack.push(result);
            }
            Instruction::Negate => {
                let value = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let result = self.negate(value)?;
                self.stack.push(result);
            }
            Instruction::Not => {
                let value = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let result = Value::Boolean(!self.is_truthy(&value));
                self.stack.push(result);
            }
            Instruction::BitNot => {
                let value = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let num = self.to_number(&value)?;
                self.stack.push(Value::Integer(!(num as i64)));
            }
            Instruction::UnaryPlus => {
                let value = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let num = self.to_number(&value)?;
                if num.fract() == 0.0 && num.abs() < i64::MAX as f64 {
                    self.stack.push(Value::Integer(num as i64));
                } else {
                    self.stack.push(Value::Float(num));
                }
            }
            Instruction::BitAnd => {
                let right = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let left = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let l = self.to_number(&left)? as i64;
                let r = self.to_number(&right)? as i64;
                self.stack.push(Value::Integer(l & r));
            }
            Instruction::BitOr => {
                let right = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let left = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let l = self.to_number(&left)? as i64;
                let r = self.to_number(&right)? as i64;
                self.stack.push(Value::Integer(l | r));
            }
            Instruction::BitXor => {
                let right = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let left = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let l = self.to_number(&left)? as i64;
                let r = self.to_number(&right)? as i64;
                self.stack.push(Value::Integer(l ^ r));
            }
            Instruction::ShiftLeft => {
                let right = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let left = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let l = self.to_number(&left)? as i64;
                let r = self.to_number(&right)? as u32;
                self.stack.push(Value::Integer(l << r));
            }
            Instruction::ShiftRight => {
                let right = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let left = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let l = self.to_number(&left)? as i64;
                let r = self.to_number(&right)? as u32;
                self.stack.push(Value::Integer(l >> r));
            }
            Instruction::Void => {
                self.stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                self.stack.push(Value::Undefined);
            }
            Instruction::TypeOf => {
                let value = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let type_str = match &value {
                    Value::Undefined => "undefined",
                    Value::Null => "object",
                    Value::Boolean(_) => "boolean",
                    Value::Integer(_) | Value::Float(_) => "number",
                    Value::String(_) => "string",
                    Value::BigInt(_) => "bigint",
                    Value::Symbol(_) => "symbol",
                    Value::Function(_) | Value::NativeFunction(_) => "function",
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
                    | Value::Buffer(_) => "object",
                    Value::Date(_) | Value::RegExp(_) => "object",
                };
                self.stack.push(Value::String(type_str.to_string()));
            }
            Instruction::ToString => {
                let value = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
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
                let right = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let left = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                self.stack
                    .push(Value::Boolean(self.is_equal(&left, &right)));
            }
            Instruction::StrictEq => {
                let right = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let left = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let eq = match (&left, &right) {
                    (Value::Float(a), Value::Float(b)) => a == b && !a.is_nan() && !b.is_nan(),
                    (Value::Integer(a), Value::Float(b)) => *a as f64 == *b && !b.is_nan(),
                    (Value::Float(a), Value::Integer(b)) => *a == *b as f64 && !a.is_nan(),
                    _ => left == right,
                };
                self.stack.push(Value::Boolean(eq));
            }
            Instruction::NotEqual => {
                let right = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let left = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                self.stack
                    .push(Value::Boolean(!self.is_equal(&left, &right)));
            }
            Instruction::StrictNotEqual => {
                let right = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let left = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                self.stack.push(Value::Boolean(left != right));
            }
            Instruction::Less => {
                let right = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let left = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let result = self.less_than(&left, &right)?;
                self.stack.push(Value::Boolean(result));
            }
            Instruction::Greater => {
                let right = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let left = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let result = self.greater_than(&left, &right)?;
                self.stack.push(Value::Boolean(result));
            }
            Instruction::LessEqual => {
                let right = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let left = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let result = self.less_than_or_equal(&left, &right)?;
                self.stack.push(Value::Boolean(result));
            }
            Instruction::GreaterEqual => {
                let right = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let left = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let result = self.greater_than_or_equal(&left, &right)?;
                self.stack.push(Value::Boolean(result));
            }
            Instruction::And => {
                let right = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let left = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                if self.is_truthy(&left) {
                    self.stack.push(right);
                } else {
                    self.stack.push(left);
                }
            }
            Instruction::Or => {
                let right = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let left = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                if self.is_truthy(&left) {
                    self.stack.push(left);
                } else {
                    self.stack.push(right);
                }
            }
            Instruction::InstanceOf => {
                let right = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let left = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let result = self.instanceof_check(&left, &right)?;
                self.stack.push(result);
            }
            Instruction::In => {
                let right = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let left = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
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
                let value = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let key = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let object = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;

                match &object {
                    Value::Proxy(proxy_idx) => {
                        if let HeapValue::Proxy(proxy) = &self.heap[*proxy_idx] {
                            let handler = proxy.handler.clone();
                            let target = proxy.target.clone();
                            let trap =
                                self.get_property(&handler, &Value::String("set".to_string()));
                            if let Ok(Value::Function(_)) | Ok(Value::NativeFunction(_)) = &trap {
                                let trap_result = self.call_value(
                                    &trap?,
                                    &handler,
                                    &[target, key.clone(), value, object.clone()],
                                );
                                trap_result?;
                            } else {
                                if let Value::Object(target_obj_idx) = &target {
                                    if let HeapValue::Object(obj) = &mut self.heap[*target_obj_idx]
                                    {
                                        if let Value::String(key_str) = &key {
                                            obj.properties.insert(key_str.clone(), value);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Value::Object(obj_idx) => {
                        if let HeapValue::Object(obj) = &mut self.heap[*obj_idx] {
                            if let Value::String(key_str) = &key {
                                let setter_key = format!("__setter_{}", key_str);
                                if let Some(setter_val) = obj.properties.get(&setter_key).cloned() {
                                    let _ = obj;
                                    self.call_value(&setter_val, &object, &[value])?;
                                } else {
                                    obj.properties.insert(key_str.clone(), value);
                                }
                            }
                        }
                    }
                    Value::Function(func_idx) => {
                        if let HeapValue::Function(f) = &mut self.heap[*func_idx] {
                            if let Value::String(key_str) = &key {
                                f.properties.insert(key_str.clone(), value);
                            }
                        }
                    }
                    _ => {}
                }
                self.stack.push(object);
            }
            Instruction::GetProperty => {
                let key = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let object = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let result = self.get_property(&object, &key)?;
                self.stack.push(result);
            }
            Instruction::OptionalGetProperty => {
                let key = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let object = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
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
                    args.push(
                        self.stack
                            .pop()
                            .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?,
                    );
                }
                args.reverse();
                let this = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let callee = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                if matches!(callee, Value::Undefined | Value::Null) {
                    self.stack.push(Value::Undefined);
                } else {
                    let result = self.call_value(&callee, &this, &args)?;
                    self.stack.push(result);
                }
            }
            Instruction::NullishCoalescing => {
                let right = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let left = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
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
                let source = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let target = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
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
                let source = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let target = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                if let Value::Object(target_idx) = target {
                    if let Value::Object(source_idx) = source {
                        if let HeapValue::Object(source_obj) = &self.heap[source_idx] {
                            let props: Vec<(String, Value)> = source_obj
                                .properties
                                .iter()
                                .map(|(k, v)| (k.clone(), v.clone()))
                                .collect();
                            if let HeapValue::Object(target_obj) = &mut self.heap[target_idx] {
                                for (k, v) in props {
                                    target_obj.properties.insert(k, v);
                                }
                            }
                        }
                    }
                }
                self.stack.push(target);
            }
            Instruction::ArrayPush => {
                let value = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let array = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                if let Value::Array(arr_idx) = array {
                    if let HeapValue::Array(arr) = &mut self.heap[arr_idx] {
                        arr.elements.push(value.clone());
                    }
                }
                self.stack.push(array);
            }
            Instruction::GetKeys => {
                let obj = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let keys: Vec<Value> = match &obj {
                    Value::Object(idx) => {
                        if let HeapValue::Object(o) = &self.heap[*idx] {
                            o.properties
                                .keys()
                                .map(|k| Value::String(k.clone()))
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
                let key = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                let object = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                match &object {
                    Value::Proxy(proxy_idx) => {
                        if let HeapValue::Proxy(proxy) = &self.heap[*proxy_idx] {
                            let handler = proxy.handler.clone();
                            let target = proxy.target.clone();
                            let trap = self.get_property(
                                &handler,
                                &Value::String("deleteProperty".to_string()),
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

    pub(crate) fn exec_make_function(
        &mut self,
        instruction: &Instruction,
        module: &CompiledModule,
        pc: usize,
    ) -> Result<bool> {
        match instruction {
            Instruction::MakeFunction(func_idx) => {
                let func_info = module.functions[*func_idx as usize].clone();
                let proto_obj_idx = self
                    .gc
                    .allocate(&mut self.heap, HeapValue::Object(JsObject::new()));
                let owner = self.current_module.clone();
                let scope = std::rc::Rc::new(self.globals.clone());
                let src_file = self.current_module_path.clone();
                let src_line = self.current_source_line(pc);
                let heap_idx = self.gc.allocate(
                    &mut self.heap,
                    HeapValue::Function(JsFunction {
                        name: func_info.name,
                        params: func_info.params,
                        bytecode_index: func_info.bytecode_index,
                        closure: Vec::new(),
                        prototype: Some(proto_obj_idx),
                        super_class: None,
                        properties: HashMap::new(),
                        owner_module: owner,
                        module_scope: Some(scope),
                        is_generator: func_info.is_generator,
                        source_file: src_file,
                        source_line: src_line,
                    }),
                );
                self.stack.push(Value::Function(heap_idx));
            }
            Instruction::MakeClosure(func_idx, _capture_slots) => {
                let func_info = module.functions[*func_idx as usize].clone();
                let mut closure_vars = Vec::new();
                let base = self.call_stack.last().map(|f| f.base_pointer).unwrap_or(0);
                for slot in _capture_slots {
                    let abs_slot = base + *slot as usize;
                    let value = self
                        .stack
                        .get(abs_slot)
                        .cloned()
                        .unwrap_or(Value::Undefined);
                    closure_vars.push(value);
                }
                let proto_obj_idx = self
                    .gc
                    .allocate(&mut self.heap, HeapValue::Object(JsObject::new()));
                let owner = self.current_module.clone();
                let scope = std::rc::Rc::new(self.globals.clone());
                let src_file = self.current_module_path.clone();
                let src_line = self.current_source_line(pc);
                let heap_idx = self.gc.allocate(
                    &mut self.heap,
                    HeapValue::Function(JsFunction {
                        name: func_info.name,
                        params: func_info.params,
                        bytecode_index: func_info.bytecode_index,
                        closure: closure_vars,
                        prototype: Some(proto_obj_idx),
                        super_class: None,
                        properties: HashMap::new(),
                        owner_module: owner,
                        module_scope: Some(scope),
                        is_generator: func_info.is_generator,
                        source_file: src_file,
                        source_line: src_line,
                    }),
                );
                self.stack.push(Value::Function(heap_idx));
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    pub(crate) fn exec_exception(
        &mut self,
        instruction: &Instruction,
        pc: &mut usize,
    ) -> Result<bool> {
        match instruction {
            Instruction::Throw => {
                let value = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                self.pending_exception = Some(value.clone());
                while let Some(handler) = self.exception_handlers.last().cloned() {
                    if handler.catch_pc != 0 {
                        self.exception_handlers.pop();
                        self.stack.truncate(handler.stack_depth);
                        *pc = handler.catch_pc as usize;
                        return Ok(true);
                    } else if handler.finally_pc != 0 {
                        self.exception_handlers.pop();
                        self.stack.truncate(handler.stack_depth);
                        *pc = handler.finally_pc as usize;
                        return Ok(true);
                    } else {
                        self.exception_handlers.pop();
                    }
                }
                return Err(Error::RuntimeError(format!(
                    "Thrown: {}",
                    self.value_to_string(&value)
                )));
            }
            Instruction::TryJump(catch_pc, finally_pc) => {
                let handler = super::call_frame::ExceptionHandler {
                    catch_pc: *catch_pc,
                    finally_pc: *finally_pc,
                    stack_depth: self.stack.len(),
                };
                self.exception_handlers.push(handler);
            }
            Instruction::PopTryHandler => {
                self.exception_handlers.pop();
                if self.pending_exception.is_some() {
                    while let Some(handler) = self.exception_handlers.last().cloned() {
                        if handler.catch_pc != 0 {
                            self.exception_handlers.pop();
                            self.stack.truncate(handler.stack_depth);
                            *pc = handler.catch_pc as usize;
                            return Ok(true);
                        } else if handler.finally_pc != 0 {
                            self.exception_handlers.pop();
                            self.stack.truncate(handler.stack_depth);
                            *pc = handler.finally_pc as usize;
                            return Ok(true);
                        } else {
                            self.exception_handlers.pop();
                        }
                    }
                    let exc = self.pending_exception.take().unwrap_or(Value::Undefined);
                    return Err(Error::RuntimeError(format!(
                        "Thrown: {}",
                        self.value_to_string(&exc)
                    )));
                }
            }
            Instruction::LoadException => {
                let exc = self.pending_exception.take().unwrap_or(Value::Undefined);
                self.stack.push(exc);
            }
            Instruction::ReThrowIfPending => {
                if self.pending_exception.is_some() {
                    while let Some(handler) = self.exception_handlers.last().cloned() {
                        if handler.catch_pc != 0 {
                            self.exception_handlers.pop();
                            self.stack.truncate(handler.stack_depth);
                            *pc = handler.catch_pc as usize;
                            return Ok(true);
                        } else if handler.finally_pc != 0 {
                            self.exception_handlers.pop();
                            self.stack.truncate(handler.stack_depth);
                            *pc = handler.finally_pc as usize;
                            return Ok(true);
                        } else {
                            self.exception_handlers.pop();
                        }
                    }
                    let exc = self.pending_exception.take().unwrap_or(Value::Undefined);
                    return Err(Error::RuntimeError(format!(
                        "Thrown: {}",
                        self.value_to_string(&exc)
                    )));
                }
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}
