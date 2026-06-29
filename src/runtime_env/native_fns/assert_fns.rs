use crate::errors::{Error, Result};
use crate::objects::Value;
use crate::vm::interpreter::{HeapValue, Interpreter, JsObject};

use super::helpers::to_string_value;

pub(super) fn native_assert(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let condition = args.first().cloned().unwrap_or(Value::Undefined);

    if !matches!(
        condition,
        Value::Boolean(true) | Value::Integer(1) | Value::Float(1.0)
    ) {
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
    // Debug: print args count and values
    eprintln!("[DEBUG assert.strictEqual] args.len()={}", args.len());
    for (i, arg) in args.iter().enumerate() {
        eprintln!("[DEBUG assert.strictEqual] args[{}] = {:?}", i, arg);
    }

    let actual = args.first().cloned().unwrap_or(Value::Undefined);
    let expected = args.get(1).cloned().unwrap_or(Value::Undefined);

    // Compare values using PartialEq
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

pub(super) fn native_assert_object(
    interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    // Create assert object with methods
    let mut props = std::collections::HashMap::new();
    props.insert("default".into(), Value::NativeFunction(389));
    props.insert("strictEqual".into(), Value::NativeFunction(390));
    props.insert("ok".into(), Value::NativeFunction(389));
    props.insert("equal".into(), Value::NativeFunction(390));
    props.insert("deepEqual".into(), Value::NativeFunction(390));

    let obj_idx = interp.heap.len();
    interp.heap.push(HeapValue::Object(JsObject {
        properties: props,
        prototype: None,
        extensible: true,
    }));

    Ok(Value::Object(obj_idx))
}
