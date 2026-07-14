use super::*;
use crate::compiler::Instruction;
use crate::errors::runtime_errors::runtime_error_stack_overflow;
use crate::errors::{Error, Result};
use crate::objects::Value;
use crate::props;
use crate::well_known as wk;

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
                    .ok_or_else(|| Error::RuntimeError(super::ERR_STACK_UNDERFLOW.into()))?;
                self.pending_exception = Some(value.clone());
                // Extract error name and message so dispatch_exception can
                // build a correct stack trace and surface the right error
                // kind if no handler is found.
                let (error_name, message) = if let Value::Object(obj_idx) = &value {
                    if let HeapValue::Object(obj) = &self.heap[*obj_idx] {
                        let name = obj
                            .properties
                            .get(wk::NAME)
                            .and_then(|v| match v {
                                Value::String(s) => Some(s.as_ref().to_string()),
                                Value::Cons(c) => Some(c.flatten().into()),
                                _ => None,
                            })
                            .unwrap_or_else(|| wk::ERROR.to_string());
                        let message = obj
                            .properties
                            .get(wk::MESSAGE)
                            .and_then(|v| match v {
                                Value::String(s) => Some(s.as_ref().to_string()),
                                Value::Cons(c) => Some(c.flatten().into()),
                                _ => None,
                            })
                            .unwrap_or_default();
                        (name, message)
                    } else {
                        (wk::ERROR.to_string(), String::new())
                    }
                } else {
                    (wk::ERROR.to_string(), self.value_to_string(&value))
                };
                return self.dispatch_exception(pc, &error_name, &message);
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
                        self.format_rejection_reason(&exc)
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
                        self.format_rejection_reason(&exc)
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
    /// Create a JS Error object of the given type, set it as the pending
    /// exception, and transfer control to the nearest catch/finally handler.
    ///
    /// Returns `Ok(true)` when a handler was found (caller should `continue`
    /// the bytecode loop with the updated `pc`). Returns `Err(...)` when no
    /// handler exists so the error propagates to the host.
    pub(crate) fn throw_js_error(
        &mut self,
        pc: &mut usize,
        error_name: &str,
        message: &str,
    ) -> Result<bool> {
        let obj_idx = self.heap.len();
        let stack = self.build_stack_trace(error_name, message);
        let props = props! {
            wk::MESSAGE => Value::from_string(message.to_string()),
            wk::NAME => Value::string(error_name),
            wk::STACK => Value::from_string(stack),
        };
        let proto_idx = self.find_error_prototype(error_name);
        self.heap.push(HeapValue::Object(JsObject {
            properties: props,
            prototype: proto_idx,
            extensible: true,
        }));
        let error_value = Value::Object(obj_idx);
        self.pending_exception = Some(error_value);

        // Dispatch to the nearest handler, unwinding the call stack as needed.
        // If no handler exists in the current exception_handlers set, walk the
        // call stack to restore saved exception handler snapshots (handlers
        // are cleared on function entry in exec_calls.rs and saved in each
        // CallFrame's exception_handlers_snapshot).
        self.dispatch_exception(pc, error_name, message)
    }

    /// Find a catch/finally handler and transfer control to it.
    fn dispatch_exception(&mut self, pc: &mut usize, error_name: &str, message: &str) -> Result<bool> {
        while let Some(handler) = self.exception_handlers.last().cloned() {
            self.exception_handlers.pop();
            if handler.catch_pc != 0 {
                self.stack.truncate(handler.stack_depth);
                // Unwind all frames pushed at or above the handler's recorded
                // stack depth. Frames with base_pointer == stack_depth were
                // created by function calls inside the try block (their
                // base_pointer equals the stack depth at the Call, which is
                // the same as the TryJump's recorded stack_depth), so they
                // must also be popped.
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
        // No handler in the current exception_handlers set. Walk the call
        // stack to find a frame whose snapshot contains a handler, unwind to
        // that frame, and restore its handlers so the error can be caught.
        while let Some(frame) = self.call_stack.pop() {
            if !frame.exception_handlers_snapshot.is_empty() {
                self.exception_handlers = frame.exception_handlers_snapshot;
                self.stack.truncate(frame.base_pointer);
                // Re-attempt dispatch with the restored handlers so the
                // catch/finally block executes in the correct scope.
                return self.dispatch_exception(pc, error_name, message);
            }
        }
        // No JS handler — surface as a host-level error of the matching kind.
        let host_err = match error_name {
            n if n == wk::REFERENCE_ERROR => Error::ReferenceError(message.to_string()),
            n if n == wk::TYPE_ERROR => Error::TypeError(message.to_string()),
            n if n == wk::SYNTAX_ERROR => Error::SyntaxError(message.to_string()),
            n if n == wk::RANGE_ERROR => runtime_error_stack_overflow(),
            _ => Error::RuntimeError(message.to_string()),
        };
        Err(self.err_at_location(host_err))
    }

    pub(crate) fn throw_stack_overflow(&mut self, pc: &mut usize) -> Result<bool> {
        self.throw_js_error(pc, wk::RANGE_ERROR, "Maximum call stack size exceeded")
    }

    /// Throw a catchable `ReferenceError` for an undeclared free variable.
    pub(crate) fn throw_reference_error(&mut self, pc: &mut usize, name: &str) -> Result<bool> {
        let message = format!("{} is not defined", name);
        self.throw_js_error(pc, wk::REFERENCE_ERROR, &message)
    }

    /// Convert a host-level `Error` (e.g. from `require()` or a native
    /// function) into a catchable JS exception when try/catch handlers are
    /// active. Without handlers, re-returns the original error.
    pub(crate) fn throw_from_host_error(&mut self, pc: &mut usize, err: Error) -> Result<bool> {
        if self.exception_handlers.is_empty() {
            return Err(err);
        }
        let (js_name, message) = match &err.kind {
            crate::errors::ErrorKind::ReferenceError(m) => (wk::REFERENCE_ERROR, m.clone()),
            crate::errors::ErrorKind::TypeError(m) => (wk::TYPE_ERROR, m.clone()),
            crate::errors::ErrorKind::SyntaxError(m) => (wk::SYNTAX_ERROR, m.clone()),
            crate::errors::ErrorKind::RuntimeError(m) => ("Error", m.clone()),
            crate::errors::ErrorKind::ParseError(m) => (wk::SYNTAX_ERROR, m.clone()),
            crate::errors::ErrorKind::InternalError(m) => ("Error", m.clone()),
        };
        self.throw_js_error(pc, js_name, &message)
    }

    fn find_error_prototype(&self, type_name: &str) -> Option<usize> {
        for (i, hv) in self.heap.iter().enumerate() {
            if let HeapValue::Object(obj) = hv {
                if let Some(Value::String(name)) = obj.properties.get(wk::NAME) {
                    if **name == *type_name {
                        return Some(i);
                    }
                }
            }
        }
        None
    }
}
