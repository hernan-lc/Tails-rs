use crate::errors::{Error, Result};
use crate::objects::Value;
use crate::vm::interpreter::Interpreter;

/// Symbol() constructor - creates a new unique symbol
/// Cannot be called with `new`
pub(super) fn native_symbol_constructor(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let _description = args.first().cloned().unwrap_or(Value::Undefined);
    let id = interp.next_symbol_id;
    interp.next_symbol_id += 1;
    Ok(Value::Symbol(id))
}

/// Symbol.for(key) - retrieves or creates a symbol in the global registry
pub(super) fn native_symbol_for(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let key = match args.first().cloned().unwrap_or(Value::Undefined) {
        Value::String(s) => s,
        other => format!("{}", other),
    };

    if let Some(&id) = interp.symbol_registry.get(&key) {
        return Ok(Value::Symbol(id));
    }

    let id = interp.next_symbol_id;
    interp.next_symbol_id += 1;
    interp.symbol_registry.insert(key, id);
    Ok(Value::Symbol(id))
}

/// Symbol.keyFor(sym) - retrieves the key for a symbol from the global registry
pub(super) fn native_symbol_key_for(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let sym = args.first().cloned().unwrap_or(Value::Undefined);
    match sym {
        Value::Symbol(id) => {
            for (key, &reg_id) in &interp.symbol_registry {
                if reg_id == id {
                    return Ok(Value::String(key.clone()));
                }
            }
            Ok(Value::Undefined)
        }
        _ => Err(Error::TypeError(
            "Symbol.keyFor requires a Symbol argument".into(),
        )),
    }
}
