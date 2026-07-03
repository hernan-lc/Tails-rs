use crate::errors::Result;
use crate::objects::Value;
use crate::vm::interpreter::{HeapValue, Interpreter, JsArray, JsObject};

use super::helpers::to_string_value;

pub(super) fn native_dns_resolve(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let domain = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    let rrtype = args
        .get(1)
        .map(|v| to_string_value(interp, v))
        .unwrap_or_else(|| "A".to_string());

    match rrtype.to_uppercase().as_str() {
        "A" => native_dns_resolve4(interp, _this, &[Value::String(domain)]),
        "AAAA" => native_dns_resolve6(interp, _this, &[Value::String(domain)]),
        "MX" => native_dns_resolve_mx(interp, _this, &[Value::String(domain)]),
        _ => {
            let arr_idx = interp.heap.len();
            interp.heap.push(HeapValue::Array(JsArray {
                elements: Vec::new(),
            }));
            Ok(Value::Array(arr_idx))
        }
    }
}

pub(super) fn native_dns_lookup(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let hostname = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    let result_idx = interp.gc.allocate(
        &mut interp.heap,
        HeapValue::Object(JsObject {
            properties: crate::props! {
                "address" => Value::String(hostname.clone()),
                "family" => Value::Integer(4),
            },
            prototype: None,
            extensible: true,
        }),
    );
    Ok(Value::Object(result_idx))
}

pub(super) fn native_dns_resolve4(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let _domain = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    let arr_idx = interp.heap.len();
    interp.heap.push(HeapValue::Array(JsArray {
        elements: Vec::new(),
    }));
    Ok(Value::Array(arr_idx))
}

pub(super) fn native_dns_resolve6(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let _domain = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    let arr_idx = interp.heap.len();
    interp.heap.push(HeapValue::Array(JsArray {
        elements: Vec::new(),
    }));
    Ok(Value::Array(arr_idx))
}

pub(super) fn native_dns_resolve_mx(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let _domain = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    let arr_idx = interp.heap.len();
    interp.heap.push(HeapValue::Array(JsArray {
        elements: Vec::new(),
    }));
    Ok(Value::Array(arr_idx))
}
