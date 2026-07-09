use super::*;
use crate::compiler::CompiledModule;
use crate::compiler::Instruction;
use crate::errors::{Error, Result};
use crate::objects::{ConsString, Value};
use crate::well_known as wk;

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
                        .and_then(|mg| mg.borrow().get(name).cloned())
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
                            .and_then(|mg| mg.borrow().get(name).cloned())
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
                            .and_then(|mg| mg.borrow().get(name).cloned())
                    })
                    .unwrap_or(Value::Undefined);
                let type_str = match &value {
                    Value::Undefined => wk::UNDEFINED,
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
                self.stack.push(Value::from_string(type_str.to_string()));
            }
            Instruction::StoreGlobal(name) => {
                let value = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError(super::ERR_STACK_UNDERFLOW.into()))?;
                if self.module_globals.is_some() {
                    if let Some(ref mg) = self.module_globals {
                        mg.borrow_mut().insert(name.clone(), value.clone());
                    }
                    self.globals.insert(name.clone(), value);
                } else {
                    self.globals.insert(name.clone(), value);
                }
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
                    .ok_or_else(|| Error::RuntimeError(super::ERR_STACK_UNDERFLOW.into()))?;
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
                    // Phase 1.8: For string concat arms, move values out of
                    // the stack first to avoid borrow conflicts between the
                    // match (which borrows &self.stack) and mem::replace
                    // (which needs &mut self.stack). Numeric arms stay
                    // in-place for zero-allocation fast paths.
                    match (&self.stack[dst_idx], &self.stack[src_idx]) {
                        (Value::Integer(a), Value::Integer(b)) => {
                            if let Some(result) = a.checked_add(*b) {
                                self.stack[dst_idx] = Value::Integer(result);
                            } else {
                                self.stack[dst_idx] = Value::Float(*a as f64 + *b as f64);
                            }
                        }
                        (Value::Float(a), Value::Float(b)) => {
                            self.stack[dst_idx] = Value::Float(a + b);
                        }
                        (Value::Integer(a), Value::Float(b)) => {
                            self.stack[dst_idx] = Value::Float(*a as f64 + *b);
                        }
                        (Value::Float(a), Value::Integer(b)) => {
                            self.stack[dst_idx] = Value::Float(*a + *b as f64);
                        }
                        _ => {
                            // String/Cons/cold path: move both values out
                            // to avoid borrow conflicts, then handle.
                            let left =
                                std::mem::replace(&mut self.stack[dst_idx], Value::Undefined);
                            let right =
                                std::mem::replace(&mut self.stack[src_idx], Value::Undefined);
                            match (&left, &right) {
                                (Value::String(a), Value::String(b)) => {
                                    self.stack[dst_idx] = Value::Cons(ConsString::new_smart(
                                        Value::String(a.clone()),
                                        Value::String(b.clone()),
                                    ));
                                }
                                (Value::Cons(_), Value::String(_))
                                | (Value::String(_), Value::Cons(_))
                                | (Value::Cons(_), Value::Cons(_)) => {
                                    self.stack[dst_idx] = Value::Cons(ConsString::new(left, right));
                                }
                                (Value::String(_), Value::Integer(b)) => {
                                    let b_val = *b;
                                    self.stack[dst_idx] = Value::Cons(ConsString::new(
                                        left,
                                        Value::from_string(b_val.to_string()),
                                    ));
                                }
                                (Value::String(_), Value::Float(b)) => {
                                    let b_val = *b;
                                    let b_str =
                                        if b_val.is_finite() && b_val == (b_val as i64) as f64 {
                                            Value::from_string((b_val as i64).to_string().into())
                                        } else {
                                            Value::from_string(b_val.to_string())
                                        };
                                    self.stack[dst_idx] = Value::Cons(ConsString::new(left, b_str));
                                }
                                (Value::Cons(_), Value::Integer(b)) => {
                                    let b_val = *b;
                                    self.stack[dst_idx] = Value::Cons(ConsString::new(
                                        left,
                                        Value::from_string(b_val.to_string()),
                                    ));
                                }
                                (Value::Cons(_), Value::Float(b)) => {
                                    let b_val = *b;
                                    let right =
                                        if b_val.is_finite() && b_val == (b_val as i64) as f64 {
                                            Value::from_string((b_val as i64).to_string().into())
                                        } else {
                                            Value::from_string(b_val.to_string())
                                        };
                                    self.stack[dst_idx] = Value::Cons(ConsString::new(left, right));
                                }
                                _ => {
                                    self.stack[dst_idx] = self.add(left, right)?;
                                }
                            }
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
