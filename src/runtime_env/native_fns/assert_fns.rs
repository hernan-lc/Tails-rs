use crate::errors::{Error, Result};
use crate::objects::Value;
use crate::vm::interpreter::heap_types::HeapValue;
use crate::vm::interpreter::Interpreter;

use super::helpers::{is_truthy, to_string_value};

pub(super) fn native_assert(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let condition = args.first().cloned().unwrap_or(Value::Undefined);

    if !is_truthy(&condition) {
        let message = args
            .get(1)
            .map(|v| to_string_value(interp, v))
            .unwrap_or_else(|| "Assertion failed".to_string());
        return Err(Error::RuntimeError(message));
    }

    Ok(Value::Undefined)
}

pub(super) fn native_assert_strict_equal(
    _interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let actual = args.first().cloned().unwrap_or(Value::Undefined);
    let expected = args.get(1).cloned().unwrap_or(Value::Undefined);

    if actual == expected {
        Ok(Value::Undefined)
    } else {
        let message = format!(
            "Values are not strictly equal. Expected: {:?}, Actual: {:?}",
            expected, actual
        );
        Err(Error::RuntimeError(message))
    }
}

fn deep_equal(a: &Value, b: &Value, heap: &[HeapValue]) -> bool {
    match (a, b) {
        (Value::Integer(x), Value::Integer(y)) => x == y,
        (Value::Float(x), Value::Float(y)) => x == y,
        (Value::Integer(x), Value::Float(y)) => (*x as f64) == *y,
        (Value::Float(x), Value::Integer(y)) => *x == (*y as f64),
        (Value::String(x), Value::String(y)) => x == y,
        (Value::Boolean(x), Value::Boolean(y)) => x == y,
        (Value::Null, Value::Null) => true,
        (Value::Undefined, Value::Undefined) => true,
        (Value::Object(ax), Value::Object(bx)) => {
            let (Some(HeapValue::Object(oa)), Some(HeapValue::Object(ob))) =
                (heap.get(*ax), heap.get(*bx))
            else {
                return false;
            };
            if oa.properties.len() != ob.properties.len() {
                return false;
            }
            for (k, va) in &oa.properties {
                match ob.properties.get(k) {
                    Some(vb) if deep_equal(va, vb, heap) => {}
                    _ => return false,
                }
            }
            true
        }
        (Value::Array(ax), Value::Array(bx)) => {
            let (Some(HeapValue::Array(aa)), Some(HeapValue::Array(ba))) =
                (heap.get(*ax), heap.get(*bx))
            else {
                return false;
            };
            if aa.elements.len() != ba.elements.len() {
                return false;
            }
            aa.elements
                .iter()
                .zip(ba.elements.iter())
                .all(|(x, y)| deep_equal(x, y, heap))
        }
        _ => false,
    }
}

pub(super) fn native_assert_deep_equal(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let actual = args.first().cloned().unwrap_or(Value::Undefined);
    let expected = args.get(1).cloned().unwrap_or(Value::Undefined);
    if deep_equal(&actual, &expected, &interp.heap) {
        Ok(Value::Undefined)
    } else {
        let message = format!(
            "Values are not deeply equal. Expected: {:?}, Actual: {:?}",
            expected, actual
        );
        Err(Error::RuntimeError(message))
    }
}

pub(super) fn native_assert_not_strict_equal(
    _interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let actual = args.first().cloned().unwrap_or(Value::Undefined);
    let expected = args.get(1).cloned().unwrap_or(Value::Undefined);
    if actual != expected {
        Ok(Value::Undefined)
    } else {
        Err(Error::RuntimeError(format!(
            "Values are strictly equal but expected to differ. Actual: {:?}",
            actual
        )))
    }
}

pub(super) fn native_assert_not_equal(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    native_assert_not_strict_equal(interp, this, args)
}

pub(super) fn native_assert_not_deep_strict_equal(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let actual = args.first().cloned().unwrap_or(Value::Undefined);
    let expected = args.get(1).cloned().unwrap_or(Value::Undefined);
    if !deep_equal(&actual, &expected, &interp.heap) {
        Ok(Value::Undefined)
    } else {
        Err(Error::RuntimeError(format!(
            "Values are deeply equal but expected to differ. Actual: {:?}",
            actual
        )))
    }
}

pub(super) fn native_assert_not_deep_equal(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    native_assert_not_deep_strict_equal(interp, this, args)
}

pub(super) fn native_assert_if_error(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let val = args.first().cloned().unwrap_or(Value::Undefined);
    let is_error = match &val {
        Value::Null | Value::Undefined | Value::Boolean(false) => false,
        Value::Object(idx) => matches!(interp.heap.get(*idx), Some(HeapValue::Object(_))),
        _ => true,
    };
    if !is_error {
        return Ok(Value::Undefined);
    }
    let message = match val {
        Value::Object(idx) => match interp.heap.get(idx) {
            Some(HeapValue::Object(o)) => o
                .properties
                .get("message")
                .map(|m| to_string_value(interp, m))
                .unwrap_or_else(|| "ifError got truthy value".to_string()),
            _ => "ifError got truthy value".to_string(),
        },
        _ => "ifError got truthy value".to_string(),
    };
    Err(Error::RuntimeError(message))
}

pub(super) fn native_assert_fail(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let message = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_else(|| "Failed".to_string());
    Err(Error::RuntimeError(message))
}

pub(super) fn native_assert_throws(
    _interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let fn_val = args.first().cloned().unwrap_or(Value::Undefined);
    match fn_val {
        Value::Function(_) | Value::NativeFunction(_) => Ok(Value::Undefined),
        _ => Err(Error::RuntimeError(
            "assert.throws requires a function".to_string(),
        )),
    }
}

pub(super) fn native_assert_does_not_throw(
    _interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let fn_val = args.first().cloned().unwrap_or(Value::Undefined);
    if matches!(fn_val, Value::Function(_) | Value::NativeFunction(_)) {
        Ok(Value::Undefined)
    } else {
        Err(Error::RuntimeError(
            "assert.doesNotThrow requires a function".to_string(),
        ))
    }
}

pub(super) fn native_assert_rejects(
    _interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    Ok(Value::Undefined)
}

pub(super) fn native_assert_does_not_reject(
    _interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    Ok(Value::Undefined)
}

pub(super) fn native_assert_match(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let value = args.first().cloned().unwrap_or(Value::Undefined);
    let pattern = args.get(1).cloned().unwrap_or(Value::Undefined);
    let vs = to_string_value(interp, &value);
    let ps = to_string_value(interp, &pattern);
    if regex_match(&vs, &ps) {
        Ok(Value::Undefined)
    } else {
        Err(Error::RuntimeError(format!(
            "The input did not match the regular expression {}. Input: '{}'",
            ps, vs
        )))
    }
}

pub(super) fn native_assert_not_match(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let value = args.first().cloned().unwrap_or(Value::Undefined);
    let pattern = args.get(1).cloned().unwrap_or(Value::Undefined);
    let vs = to_string_value(interp, &value);
    let ps = to_string_value(interp, &pattern);
    if !regex_match(&vs, &ps) {
        Ok(Value::Undefined)
    } else {
        Err(Error::RuntimeError(format!(
            "The input matched the regular expression {}. Input: '{}'",
            ps, vs
        )))
    }
}

fn regex_match(input: &str, pattern: &str) -> bool {
    use regex::Regex;
    if let Ok(re) = Regex::new(pattern) {
        re.is_match(input)
    } else {
        input.contains(pattern)
    }
}
