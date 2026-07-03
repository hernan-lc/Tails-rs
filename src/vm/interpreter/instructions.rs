use super::*;
use crate::compiler::CompiledModule;
use crate::compiler::Instruction;
use crate::errors::{Error, Result};
use crate::objects::{ConsString, Value};

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
                    Value::String(_) | Value::Cons(_) => "string",
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
                    Value::NativeObject(_) => "object",
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
                // Hot path: avoid `.cloned().unwrap_or(...)` which always
                // creates a Value::Undefined even on success. Branching on the
                // bounds check first lets the compiler emit a single memcpy
                // on the happy path.
                //
                // When we have a non-empty call_stack, the call_stack.last()
                // check is taken; in the common case (always inside a function)
                // this is a single Option::Some branch and an unwrap.
                let loaded = if let Some(frame) = self.call_stack.last() {
                    self.stack.get(frame.base_pointer + *slot as usize).cloned()
                } else {
                    self.stack.get(*slot as usize).cloned()
                };
                self.stack.push(loaded.unwrap_or(Value::Undefined));
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
                // Direct index write — avoid the Vec::Index trait overhead.
                self.stack[idx] = value;
            }
            Instruction::IncLocal(slot, delta) => {
                if let Some(frame) = self.call_stack.last() {
                    let idx = frame.base_pointer + *slot as usize;
                    if idx < self.stack.len() {
                        match &self.stack[idx] {
                            Value::Integer(n) => {
                                self.stack[idx] = Value::Integer(n + delta);
                            }
                            Value::Float(n) => {
                                self.stack[idx] = Value::Float(n + *delta as f64);
                            }
                            _ => {}
                        }
                    }
                } else if (*slot as usize) < self.stack.len() {
                    let idx = *slot as usize;
                    match &self.stack[idx] {
                        Value::Integer(n) => {
                            self.stack[idx] = Value::Integer(n + delta);
                        }
                        Value::Float(n) => {
                            self.stack[idx] = Value::Float(n + *delta as f64);
                        }
                        _ => {}
                    }
                }
            }
            Instruction::AddLocal(dst, src) => {
                let base = self.call_stack.last().map(|f| f.base_pointer).unwrap_or(0);
                let dst_idx = base + *dst as usize;
                let src_idx = base + *src as usize;
                if dst_idx < self.stack.len() && src_idx < self.stack.len() {
                    // Specialized hot path: avoid cloning the 32-byte Value
                    // when both operands are primitives. The common case in
                    // numeric loops is Integer+Integer, which we handle
                    // in-place with only the discriminant being touched.
                    match (&self.stack[dst_idx], &self.stack[src_idx]) {
                        (Value::Integer(a), Value::Integer(b)) => {
                            if let Some(result) = a.checked_add(*b) {
                                self.stack[dst_idx] = Value::Integer(result);
                            } else {
                                self.stack[dst_idx] = Value::Float(*a as f64 + *b as f64);
                            }
                        }
                        (Value::Float(a), Value::Float(b)) => {
                            // Direct in-place write: no clone, no push/pop churn.
                            self.stack[dst_idx] = Value::Float(a + b);
                        }
                        (Value::Integer(a), Value::Float(b)) => {
                            self.stack[dst_idx] = Value::Float(*a as f64 + *b);
                        }
                        (Value::Float(a), Value::Integer(b)) => {
                            self.stack[dst_idx] = Value::Float(*a + *b as f64);
                        }
                        // Phase 1.7: String concat via ConsString rope.
                        // Avoids allocating a fresh String of
                        // `dst_str.len() + src_str.len()` bytes. Builds a
                        // lazy tree node instead.
                        (Value::String(dst_str), Value::String(src_str)) => {
                            self.stack[dst_idx] = Value::Cons(ConsString::new(
                                Value::String(dst_str.clone()),
                                Value::String(src_str.clone()),
                            ));
                        }
                        (Value::Cons(c), Value::String(src_str)) => {
                            self.stack[dst_idx] = Value::Cons(ConsString::new(
                                Value::Cons(c.clone()),
                                Value::String(src_str.clone()),
                            ));
                        }
                        (Value::String(dst_str), Value::Cons(c)) => {
                            self.stack[dst_idx] = Value::Cons(ConsString::new(
                                Value::String(dst_str.clone()),
                                Value::Cons(c.clone()),
                            ));
                        }
                        (Value::Cons(a), Value::Cons(b)) => {
                            self.stack[dst_idx] = Value::Cons(ConsString::new(
                                Value::Cons(a.clone()),
                                Value::Cons(b.clone()),
                            ));
                        }
                        // Phase 5F (String + Number): `"answer: " + 42` is
                        // extremely common. Avoid the `to_string_coerce`
                        // round-trip and the clone of the source `Value::String`
                        // by formatting the small primitive directly into a
                        // ConsString leaf.
                        (Value::String(dst_str), Value::Integer(b)) => {
                            let b_str = Value::String(b.to_string());
                            self.stack[dst_idx] = Value::Cons(ConsString::new(
                                Value::String(dst_str.clone()),
                                b_str,
                            ));
                        }
                        (Value::String(dst_str), Value::Float(b)) => {
                            // Match `to_string_coerce` for finite integers:
                            // "5" instead of "5.0" reads better.
                            let b_str = if b.is_finite() && *b == (*b as i64) as f64 {
                                Value::String((*b as i64).to_string())
                            } else {
                                Value::String(b.to_string())
                            };
                            self.stack[dst_idx] = Value::Cons(ConsString::new(
                                Value::String(dst_str.clone()),
                                b_str,
                            ));
                        }
                        (Value::Cons(c), Value::Integer(b)) => {
                            let right = Value::String(b.to_string());
                            self.stack[dst_idx] = Value::Cons(ConsString::new(
                                Value::Cons(c.clone()),
                                right,
                            ));
                        }
                        (Value::Cons(c), Value::Float(b)) => {
                            let right = if b.is_finite() && *b == (*b as i64) as f64 {
                                Value::String((*b as i64).to_string())
                            } else {
                                Value::String(b.to_string())
                            };
                            self.stack[dst_idx] = Value::Cons(ConsString::new(
                                Value::Cons(c.clone()),
                                right,
                            ));
                        }
                        _ => {
                            // Fallback: clone is required for self.add(left, right)
                            // which takes by-value. This is the cold path.
                            let dst_val = self.stack[dst_idx].clone();
                            let src_val = self.stack[src_idx].clone();
                            self.stack[dst_idx] = self.add(dst_val, src_val)?;
                        }
                    }
                }
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
                let is_generator = self
                    .call_stack
                    .last()
                    .map(|f| f.generator_heap_idx.is_some())
                    .unwrap_or(false);
                if is_generator {
                    // Skip BlockEnter in generators — block scope is managed by
                    // the yield/resume mechanism via saved_block_scope_stack.
                } else {
                    self.block_scope_stack.push(self.stack.len());
                }
            }
            Instruction::BlockExit => {
                let is_generator = self
                    .call_stack
                    .last()
                    .map(|f| f.generator_heap_idx.is_some())
                    .unwrap_or(false);
                if is_generator {
                    // Skip BlockExit in generators — block scope cleanup is
                    // handled by saved_stack truncation in native_generator_next.
                } else {
                    if let Some(block_base) = self.block_scope_stack.pop() {
                        if self.stack.len() > block_base {
                            let top_value = self.stack.pop().unwrap_or(Value::Undefined);
                            self.stack.truncate(block_base);
                            self.stack.push(top_value);
                        } else {
                            self.stack.truncate(block_base);
                        }
                    }
                }
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}
