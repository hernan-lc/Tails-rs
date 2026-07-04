
pub fn type_error(message: &str) -> Error {
    Error::TypeError(message.to_string())
}

pub fn type_error_expected(expected: &str, actual: &str) -> Error {
    Error::TypeError(format!("Expected {}, got {}", expected, actual))
}

pub fn type_error_not_assignable(from: &str, to: &str) -> Error {
    Error::TypeError(format!("Type '{}' is not assignable to type '{}'", from, to))
}

pub fn type_error_missing_initializer(name: &str) -> Error {
    Error::TypeError(format!("Missing initializer in const declaration '{}'", name))
}

pub fn type_error_cannot_invoke(value: &str) -> Error {
    Error::TypeError(format!("'{}' is not a function", value))
}

pub fn type_error_cannot_new(value: &str) -> Error {
    Error::TypeError(format!("'{}' is not a constructor", value))
}

pub fn type_error_cannot_read_property(obj: &str, prop: &str) -> Error {
    Error::TypeError(format!("Cannot read properties of {} (reading '{}')", obj, prop))
}

pub fn type_error_cannot_set_property(obj: &str, prop: &str) -> Error {
    Error::TypeError(format!("Cannot set properties of {} (setting '{}')", obj, prop))
}

pub fn type_error_operator(op: &str, left: &str, right: &str) -> Error {
    Error::TypeError(format!(
        "Operator '{}' cannot be applied to types '{}' and '{}'",
        op, left, right
    ))
}

pub fn type_error_undefined(name: &str) -> Error {
    Error::TypeError(format!("'{}' is undefined", name))
}
