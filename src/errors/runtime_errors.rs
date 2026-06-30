use crate::errors::Error;

pub fn runtime_error(message: &str) -> Error {
    Error::RuntimeError(message.to_string())
}

pub fn runtime_error_cannot_read_property(obj: &str, prop: &str) -> Error {
    Error::RuntimeError(format!(
        "Cannot read properties of {} (reading '{}')",
        obj, prop
    ))
}

pub fn runtime_error_cannot_set_property(obj: &str, prop: &str) -> Error {
    Error::RuntimeError(format!(
        "Cannot set properties of {} (setting '{}')",
        obj, prop
    ))
}

pub fn runtime_error_cannot_convert_to(value: &str, target: &str) -> Error {
    Error::RuntimeError(format!("Cannot convert {} to {}", value, target))
}

pub fn runtime_error_stack_overflow() -> Error {
    Error::RuntimeError("Maximum call stack size exceeded".into())
}

pub fn runtime_error_out_of_memory() -> Error {
    Error::RuntimeError("JavaScript heap out of memory".into())
}

pub fn runtime_error_invalid_conversion(from: &str, to: &str) -> Error {
    Error::RuntimeError(format!("Cannot convert {} to {}", from, to))
}

pub fn runtime_error_not_callable(value: &str) -> Error {
    Error::RuntimeError(format!("{} is not a function", value))
}

pub fn runtime_error_not_object(value: &str) -> Error {
    Error::RuntimeError(format!("Cannot convert {} to object", value))
}

pub fn runtime_error_cannot_define_property(obj: &str, prop: &str) -> Error {
    Error::RuntimeError(format!("Cannot define property '{}' of {}", prop, obj))
}

pub fn runtime_error_cannot_delete_property(obj: &str, prop: &str) -> Error {
    Error::RuntimeError(format!("Cannot delete property '{}' of {}", prop, obj))
}

pub fn runtime_error_cannot_freeze(obj: &str) -> Error {
    Error::RuntimeError(format!("Cannot freeze {}", obj))
}

pub fn runtime_error_cannot_seal(obj: &str) -> Error {
    Error::RuntimeError(format!("Cannot seal {}", obj))
}

pub fn runtime_error_cannot_prevent_extensions(obj: &str) -> Error {
    Error::RuntimeError(format!("Cannot prevent extensions of {}", obj))
}

pub fn runtime_error_cannot_access_before_initialization(name: &str) -> Error {
    Error::RuntimeError(format!("Cannot access '{}' before initialization", name))
}

pub fn runtime_error_duplicate_variable(name: &str) -> Error {
    Error::RuntimeError(format!("Duplicate variable declaration '{}'", name))
}

pub fn runtime_error_missing_return() -> Error {
    Error::RuntimeError("Function does not return a value".into())
}

pub fn runtime_error_too_many_arguments(max: usize) -> Error {
    Error::RuntimeError(format!("Maximum arguments limit ({}) exceeded", max))
}

pub fn runtime_error_invalid_regular_expression(pattern: &str) -> Error {
    Error::RuntimeError(format!("Invalid regular expression: {}", pattern))
}

pub fn runtime_error_division_by_zero() -> Error {
    Error::RuntimeError("Division by zero".into())
}

pub fn runtime_error_index_out_of_bounds(index: usize, length: usize) -> Error {
    Error::RuntimeError(format!(
        "Index {} out of bounds for length {}",
        index, length
    ))
}
