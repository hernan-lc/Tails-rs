use crate::errors::Result;
use crate::objects::Value;
use crate::vm::interpreter::Interpreter;

use super::helpers::to_string_value;

pub(super) fn native_path_join(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let parts: Vec<String> = args.iter().map(|v| to_string_value(interp, v)).collect();
    Ok(Value::from_string(tails_path::join(&parts)))
}

pub(super) fn native_path_resolve(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let parts: Vec<String> = args.iter().map(|v| to_string_value(interp, v)).collect();
    Ok(Value::from_string(tails_path::resolve(&parts)))
}

pub(super) fn native_path_basename(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let path = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    let ext = args.get(1).map(|v| to_string_value(interp, v));
    let ext_ref = ext.as_deref();
    Ok(Value::from_string(tails_path::basename(&path, ext_ref)))
}

pub(super) fn native_path_dirname(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let path = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    Ok(Value::from_string(tails_path::dirname(&path)))
}

pub(super) fn native_path_extname(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let path = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    Ok(Value::from_string(tails_path::extname(&path)))
}

pub(super) fn native_path_relative(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let from = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_else(|| ".".to_string());
    let to = args
        .get(1)
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    Ok(Value::from_string(tails_path::relative(&from, &to)))
}

pub(super) fn native_path_is_absolute(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let path = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    Ok(Value::Boolean(tails_path::is_absolute(&path)))
}

pub(super) fn native_path_normalize(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let path = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    Ok(Value::from_string(tails_path::normalize(&path)))
}
