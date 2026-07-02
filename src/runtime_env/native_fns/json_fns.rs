use crate::errors::{Error, Result};
use crate::objects::Value;
use crate::vm::interpreter::Interpreter;

use super::helpers::{from_json_value, to_json_value, to_string_value};

pub(super) fn native_json_parse(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let s = match args.first() {
        Some(Value::String(s)) => s.as_str(),
        Some(other) => {
            return Err(Error::TypeError(format!(
                "Cannot parse {}",
                to_string_value(interp, other)
            )))
        }
        None => return Err(Error::TypeError("JSON.parse requires 1 argument".into())),
    };
    let mut s_mut = s.to_owned();
    let parsed: serde_json::Value = unsafe { simd_json::from_str(&mut s_mut) }
        .map_err(|e| Error::SyntaxError(format!("JSON parse error: {}", e)))?;
    Ok(from_json_value(interp, parsed))
}

pub(super) fn native_json_stringify(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let val = args.first().cloned().unwrap_or(Value::Undefined);
    Ok(Value::String(to_json_value(interp, &val)))
}
