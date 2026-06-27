use crate::objects::Value;
use crate::vm::interpreter::Interpreter;

pub(super) fn to_f64(v: &Value) -> f64 {
    match v {
        Value::Integer(n) => *n as f64,
        Value::Float(n) => *n,
        Value::Boolean(b) => {
            if *b {
                1.0
            } else {
                0.0
            }
        }
        Value::Null => 0.0,
        Value::Undefined => f64::NAN,
        Value::String(s) => s.parse::<f64>().unwrap_or(f64::NAN),
        _ => f64::NAN,
    }
}

pub(super) fn to_i64(v: &Value) -> i64 {
    match v {
        Value::Integer(n) => *n,
        Value::Float(n) => *n as i64,
        Value::Boolean(b) if *b => 1,
        Value::Boolean(_) => 0,
        _ => 0,
    }
}

pub(super) fn to_string_value(interp: &Interpreter, v: &Value) -> String {
    match v {
        Value::Undefined => "undefined".to_string(),
        Value::Null => "null".to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Integer(n) => n.to_string(),
        Value::Float(n) => {
            if *n == (*n as i64) as f64 {
                (*n as i64).to_string()
            } else {
                n.to_string()
            }
        }
        Value::String(s) => s.clone(),
        Value::Array(arr_idx) => {
            if let crate::vm::interpreter::HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                let parts: Vec<String> = arr
                    .elements
                    .iter()
                    .map(|e| to_string_value(interp, e))
                    .collect();
                parts.join(",")
            } else {
                "[Array]".to_string()
            }
        }
        Value::Object(obj_idx) => {
            if let crate::vm::interpreter::HeapValue::Object(obj) = &interp.heap[*obj_idx] {
                let parts: Vec<String> = obj
                    .properties
                    .iter()
                    .map(|(k, v)| format!("\"{}\":{}", k, to_json_value(interp, v)))
                    .collect();
                format!("{{{}}}", parts.join(","))
            } else {
                "[Object]".to_string()
            }
        }
        Value::Proxy(_) => "[Proxy]".to_string(),
        _ => "[Function]".to_string(),
    }
}

pub(super) fn to_display_string(interp: &Interpreter, v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        other => to_string_value(interp, other),
    }
}

pub(super) fn to_json_value(interp: &Interpreter, v: &Value) -> String {
    match v {
        Value::Null => "null".to_string(),
        Value::Undefined => "undefined".to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Integer(n) => n.to_string(),
        Value::Float(n) => {
            if n.is_nan() {
                "null".to_string()
            } else if *n == (*n as i64) as f64 {
                (*n as i64).to_string()
            } else {
                n.to_string()
            }
        }
        Value::String(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
        Value::Array(arr_idx) => {
            if let crate::vm::interpreter::HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                let parts: Vec<String> = arr
                    .elements
                    .iter()
                    .map(|e| to_json_value(interp, e))
                    .collect();
                format!("[{}]", parts.join(","))
            } else {
                "[]".to_string()
            }
        }
        Value::Object(obj_idx) => {
            if let crate::vm::interpreter::HeapValue::Object(obj) = &interp.heap[*obj_idx] {
                let parts: Vec<String> = obj
                    .properties
                    .iter()
                    .map(|(k, v)| format!("\"{}\":{}", k, to_json_value(interp, v)))
                    .collect();
                format!("{{{}}}", parts.join(","))
            } else {
                "{}".to_string()
            }
        }
        Value::Proxy(_) => "null".to_string(),
        _ => "null".to_string(),
    }
}

pub(super) fn from_json_value(interp: &mut Interpreter, val: serde_json::Value) -> Value {
    match val {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Boolean(b),
        serde_json::Value::Number(n) => Value::Float(n.as_f64().unwrap_or(f64::NAN)),
        serde_json::Value::String(s) => Value::String(s),
        serde_json::Value::Array(arr) => {
            let elems: Vec<Value> = arr
                .into_iter()
                .map(|v| from_json_value(interp, v))
                .collect();
            let heap_idx = interp.heap.len();
            interp.heap.push(crate::vm::interpreter::HeapValue::Array(
                crate::vm::interpreter::JsArray { elements: elems },
            ));
            Value::Array(heap_idx)
        }
        serde_json::Value::Object(map) => {
            let heap_idx = interp.heap.len();
            let mut props = std::collections::HashMap::new();
            for (k, v) in map {
                props.insert(k, from_json_value(interp, v));
            }
            interp.heap.push(crate::vm::interpreter::HeapValue::Object(
                crate::vm::interpreter::JsObject {
                    properties: props,
                    prototype: None,
                    extensible: true,
                },
            ));
            Value::Object(heap_idx)
        }
    }
}

pub(super) fn get_array_elements(
    interp: &Interpreter,
    v: &Value,
) -> std::result::Result<Vec<Value>, crate::errors::Error> {
    match v {
        Value::Array(arr_idx) => {
            if let crate::vm::interpreter::HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                Ok(arr.elements.clone())
            } else {
                Ok(Vec::new())
            }
        }
        _ => Ok(Vec::new()),
    }
}

pub(super) fn get_string(this: &Value) -> Option<String> {
    match this {
        Value::String(s) => Some(s.clone()),
        _ => None,
    }
}

pub(super) fn find_error_ctor_proto(interp: &Interpreter) -> Option<usize> {
    for hv in &interp.heap {
        if let crate::vm::interpreter::HeapValue::Object(obj) = hv {
            if obj.properties.contains_key("prototype") && !obj.properties.contains_key("name") {
                if let Some(Value::Object(proto_idx)) = obj.properties.get("prototype") {
                    return Some(*proto_idx);
                }
            }
        }
    }
    None
}

pub(super) fn find_error_proto(interp: &Interpreter, type_name: &str) -> Option<usize> {
    for (i, hv) in interp.heap.iter().enumerate() {
        if let crate::vm::interpreter::HeapValue::Object(obj) = hv {
            if let Some(Value::String(name)) = obj.properties.get("name") {
                if name == type_name {
                    return Some(i);
                }
            }
        }
    }
    None
}

pub(super) fn is_truthy(v: &Value) -> bool {
    match v {
        Value::Undefined | Value::Null => false,
        Value::Boolean(b) => *b,
        Value::Integer(n) => *n != 0,
        Value::Float(n) => !n.is_nan() && *n != 0.0,
        Value::String(s) => !s.is_empty(),
        Value::BigInt(n) => *n != 0,
        _ => true,
    }
}
