use super::*;
use crate::compiler::Instruction;
use crate::errors::{Error, Result};
use crate::objects::Value;

pub(crate) enum ControlFlowOutcome {
    Continue,
    Next,
    Jump(usize),
    Return(Value),
}

impl Interpreter {
    pub(crate) fn exec_control_flow(
        &mut self,
        instruction: &Instruction,
        _module: &CompiledModule,
        pc: &mut usize,
    ) -> Result<ControlFlowOutcome> {
        match instruction {
            Instruction::Jump(target) => {
                *pc = *target as usize;
                Ok(ControlFlowOutcome::Continue)
            }
            Instruction::JumpIf(target) => {
                let value = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                if self.is_truthy(&value) {
                    *pc = *target as usize;
                    Ok(ControlFlowOutcome::Continue)
                } else {
                    Ok(ControlFlowOutcome::Next)
                }
            }
            Instruction::JumpIfNot(target) => {
                let value = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                if !self.is_truthy(&value) {
                    *pc = *target as usize;
                    Ok(ControlFlowOutcome::Continue)
                } else {
                    Ok(ControlFlowOutcome::Next)
                }
            }
            Instruction::JumpIfUndefined(target) => {
                let value = self
                    .stack
                    .pop()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                if matches!(value, Value::Undefined | Value::Null) {
                    *pc = *target as usize;
                    Ok(ControlFlowOutcome::Continue)
                } else {
                    Ok(ControlFlowOutcome::Next)
                }
            }
            Instruction::JumpIfNotUndefined(target) => {
                let value = self
                    .stack
                    .last()
                    .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                if !matches!(value, Value::Undefined | Value::Null) {
                    *pc = *target as usize;
                    Ok(ControlFlowOutcome::Continue)
                } else {
                    Ok(ControlFlowOutcome::Next)
                }
            }
            Instruction::Return => {
                let return_value = self.stack.pop().unwrap_or(Value::Undefined);
                if let Some(frame) = self.call_stack.pop() {
                    if frame.closure_var_count > 0 {
                        if let Some(heap_idx) = frame.func_heap_idx {
                            if let HeapValue::Function(f) = &mut self.heap[heap_idx] {
                                f.closure.clear();
                                for i in 0..frame.closure_var_count {
                                    let val = self
                                        .stack
                                        .get(frame.base_pointer + i)
                                        .cloned()
                                        .unwrap_or(Value::Undefined);
                                    f.closure.push(val);
                                }
                            }
                        }
                    }
                    let final_value = if frame.is_construct {
                        match &return_value {
                            Value::Object(_)
                            | Value::Array(_)
                            | Value::Function(_)
                            | Value::Promise(_)
                            | Value::Proxy(_)
                            | Value::Date(_)
                            | Value::RegExp(_)
                            | Value::Map(_)
                            | Value::Set(_)
                            | Value::TypedArray(_) => return_value,
                            _ => frame.this_value.clone().unwrap_or(return_value),
                        }
                    } else {
                        return_value
                    };
                    self.stack.truncate(frame.base_pointer);
                    self.stack.push(final_value);
                    self.exception_handlers = frame.exception_handlers_snapshot;
                    *pc = frame.return_address;
                    Ok(ControlFlowOutcome::Continue)
                } else {
                    Ok(ControlFlowOutcome::Return(return_value))
                }
            }
            Instruction::Yield => {
                let yield_value = self.stack.pop().unwrap_or(Value::Undefined);
                if let Some(frame) = self.call_stack.last() {
                    let saved_pc = *pc + 1;
                    let saved_stack: Vec<Value> = self.stack[frame.base_pointer..].to_vec();
                    if let Some(gen_heap_idx) = frame.generator_heap_idx {
                        if let HeapValue::Generator(gen) = &mut self.heap[gen_heap_idx] {
                            gen.yield_value = yield_value.clone();
                            gen.resume_pc = saved_pc;
                            gen.saved_stack = saved_stack;
                            gen.saved_block_scope_stack = self.block_scope_stack.clone();
                            gen.func_heap_idx = frame.func_heap_idx;
                            gen.generator_yielded = true;
                        }
                    }
                }
                Ok(ControlFlowOutcome::Return(yield_value))
            }
            _ => Ok(ControlFlowOutcome::Next),
        }
    }
}
