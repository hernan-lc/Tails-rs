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
        Some(Value::String(s)) => s.as_ref(),
        Some(other) => {
            return Err(Error::TypeError(format!(
                "Cannot parse {}",
                to_string_value(interp, other)
            )))
        }
        None => return Err(Error::TypeError("JSON.parse requires 1 argument".into())),
    };
    // Default: fully safe serde_json. Optional `fast-json` uses simd-json
    // (which requires `unsafe` at the parse call site) for higher throughput.
    #[cfg(feature = "fast-json")]
    let parsed: serde_json::Value = {
        let mut s_mut = s.to_owned();
        // Safety: simd_json::from_str requires a mutable exclusive buffer; we
        // own `s_mut` and do not alias it for the duration of the call.
        match unsafe { simd_json::from_str(&mut s_mut) } {
            Ok(v) => v,
            Err(_) => serde_json::from_str(s)
                .map_err(|e| Error::SyntaxError(format!("JSON parse error: {}", e)))?,
        }
    };
    #[cfg(not(feature = "fast-json"))]
    let parsed: serde_json::Value = serde_json::from_str(s)
        .map_err(|e| Error::SyntaxError(format!("JSON parse error: {}", e)))?;
    Ok(from_json_value(interp, parsed))
}

pub(super) fn native_json_stringify(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let val = args.first().cloned().unwrap_or(Value::Undefined);
    Ok(Value::from_string(to_json_value(interp, &val)))
}
