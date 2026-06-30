use crate::errors::Error;

pub fn runtime_error_stack_overflow() -> Error {
    Error::RuntimeError("Maximum call stack size exceeded".into())
}
