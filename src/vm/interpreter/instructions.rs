use super::*;
use crate::compiler::CompiledModule;
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
            Instruction::IncLocal(slot, delta) => {
                let base = self.call_stack.last().map(|f| f.base_pointer).unwrap_or(0);
                let idx = base + *slot as usize;
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
                self.block_scope_stack.push(self.stack.len());
            }
            Instruction::BlockExit => {
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
            _ => return Ok(false),
        }
        Ok(true)
    }
}
