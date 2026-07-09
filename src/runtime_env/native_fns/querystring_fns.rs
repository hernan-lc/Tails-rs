use crate::errors::Result;
use crate::objects::Value;
use crate::vm::interpreter::{HeapValue, Interpreter, JsObject};

use super::helpers::to_string_value;

pub(super) fn native_querystring_parse(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let input = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    let sep = args
        .get(1)
        .map(|v| to_string_value(interp, v))
        .unwrap_or_else(|| "&".to_string());
    let eq = args
        .get(2)
        .map(|v| to_string_value(interp, v))
        .unwrap_or_else(|| "=".to_string());

    let obj_idx = interp
        .gc
        .allocate(&mut interp.heap, HeapValue::Object(JsObject::new()));

    if input.is_empty() {
        return Ok(Value::Object(obj_idx));
    }

    if let HeapValue::Object(obj) = &mut interp.heap[obj_idx] {
        for pair in input.split(&sep) {
            if pair.is_empty() {
                continue;
            }
            let (key, value) = if let Some(pos) = pair.find(&eq) {
                (&pair[..pos], &pair[pos + eq.len()..])
            } else {
                (pair, "")
            };
            let decoded_key = urlencoding::decode(key)
                .map(|s| s.into_owned())
                .unwrap_or_else(|_| key.to_string());
            let decoded_value = urlencoding::decode(value)
                .map(|s| s.into_owned())
                .unwrap_or_else(|_| value.to_string());
            obj.properties
                .insert(decoded_key, Value::from_string(decoded_value));
        }
    }

    Ok(Value::Object(obj_idx))
}

pub(super) fn native_querystring_stringify(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let obj = args.first().cloned().unwrap_or(Value::Undefined);
    let sep = args
        .get(1)
        .map(|v| to_string_value(interp, v))
        .unwrap_or_else(|| "&".to_string());
    let eq = args
        .get(2)
        .map(|v| to_string_value(interp, v))
        .unwrap_or_else(|| "=".to_string());

    let idx = match &obj {
        Value::Object(idx) => *idx,
        _ => return Ok(Value::string("")),
    };

    let mut pairs;
    if let HeapValue::Object(obj) = &interp.heap[idx] {
        pairs = Vec::with_capacity(obj.properties.len());
        for (key, value) in &obj.properties {
            let val_str = to_string_value(interp, value);
            let encoded_key = urlencoding::encode(key);
            let encoded_val = urlencoding::encode(&val_str);
            pairs.push(format!("{}{}{}", encoded_key, eq, encoded_val));
        }
    } else {
        pairs = Vec::new();
    }

    Ok(Value::from_string(pairs.join(&sep)))
}

pub(super) fn native_querystring_encode(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let input = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    Ok(Value::from_string(
        urlencoding::encode(&input).into_owned(),
    ))
}

pub(super) fn native_querystring_decode(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let input = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    Ok(Value::from_string(
        urlencoding::decode(&input)
            .map(|s| s.into_owned())
            .unwrap_or_default(),
    ))
}
