use super::*;
use crate::compiler::Instruction;
use crate::errors::runtime_errors::runtime_error_stack_overflow;
use crate::errors::{Error, Result};
use crate::objects::Value;

impl Interpreter {
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
                    }
                    if handler.finally_pc != 0 {
                        self.exception_handlers.pop();
                        self.stack.truncate(handler.stack_depth);
                        *pc = handler.finally_pc as usize;
                        return Ok(true);
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

    pub(crate) fn handle_pending_exception(&mut self, pc: &mut usize) -> Result<bool> {
        if self.pending_exception.is_some() {
            while let Some(handler) = self.exception_handlers.last().cloned() {
                self.exception_handlers.pop();
                self.stack.truncate(handler.stack_depth);
                if handler.catch_pc != 0 {
                    *pc = handler.catch_pc as usize;
                    return Ok(true);
                }
                if handler.finally_pc != 0 {
                    *pc = handler.finally_pc as usize;
                    return Ok(true);
                }
            }
            let exc = self.pending_exception.take().unwrap_or(Value::Undefined);
            let formatted = self.format_rejection_reason(&exc);
            return Err(Error::RuntimeError(format!(
                "Unhandled promise rejection:\n{}",
                formatted
            )));
        }
        Ok(false)
    }
}

impl Interpreter {
    pub(crate) fn throw_stack_overflow(&mut self, pc: &mut usize) -> Result<bool> {
        let message = "Maximum call stack size exceeded";
        let obj_idx = self.heap.len();
        let stack = self.build_stack_trace("RangeError", message);
        let mut props = std::collections::HashMap::new();
        props.insert("message".into(), Value::String(message.into()));
        props.insert("name".into(), Value::String("RangeError".into()));
        props.insert("stack".into(), Value::String(stack));
        let proto_idx = self.find_error_prototype("RangeError");
        self.heap.push(HeapValue::Object(JsObject {
            properties: props,
            prototype: proto_idx,
            extensible: true,
        }));
        let range_error = Value::Object(obj_idx);
        self.pending_exception = Some(range_error.clone());
        while let Some(handler) = self.exception_handlers.last().cloned() {
            self.exception_handlers.pop();
            if handler.catch_pc != 0 {
                self.stack.truncate(handler.stack_depth);
                while self
                    .call_stack
                    .last()
                    .is_some_and(|f| f.base_pointer >= handler.stack_depth)
                {
                    self.call_stack.pop();
                }
                *pc = handler.catch_pc as usize;
                return Ok(true);
            }
            if handler.finally_pc != 0 {
                self.stack.truncate(handler.stack_depth);
                while self
                    .call_stack
                    .last()
                    .is_some_and(|f| f.base_pointer >= handler.stack_depth)
                {
                    self.call_stack.pop();
                }
                *pc = handler.finally_pc as usize;
                return Ok(true);
            }
        }
        Err(runtime_error_stack_overflow())
    }

    fn find_error_prototype(&self, type_name: &str) -> Option<usize> {
        for (i, hv) in self.heap.iter().enumerate() {
            if let HeapValue::Object(obj) = hv {
                if let Some(Value::String(name)) = obj.properties.get("name") {
                    if name == type_name {
                        return Some(i);
                    }
                }
            }
        }
        None
    }
}
