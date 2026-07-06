use crate::errors::{Error, Result};
use crate::objects::Value;
use crate::props;
use crate::runtime_env::native_fns::constants as c;
use crate::runtime_env::native_fns::helpers::to_string_value;
use crate::vm::interpreter::{HeapValue, Interpreter, JsObject};

use super::headers::{create_headers_props, get_string_prop};

pub(crate) fn build_response(
    interp: &mut Interpreter,
    body: String,
    status: u16,
    status_text: &str,
    headers_raw: &str,
) -> Result<Value> {
    let mut props = props! {
        "status" => Value::Integer(status as i64),
        "statusText" => Value::String(status_text.to_string()),
        "ok" => Value::Boolean((200..300).contains(&status)),
        "__body" => Value::String(body),
        "__headers" => Value::String(headers_raw.to_string()),
        "text" => Value::NativeFunction(c::RESPONSE_TEXT),
        "json" => Value::NativeFunction(c::RESPONSE_JSON),
        "arrayBuffer" => Value::NativeFunction(c::RESPONSE_ARRAY_BUFFER),
        "clone" => Value::NativeFunction(c::RESPONSE_CLONE),
    };

    let h_idx = interp.heap.len();
    let h_props = create_headers_props(headers_raw);
    interp.heap.push(HeapValue::Object(JsObject {
        properties: h_props,
        prototype: None,
        extensible: true,
    }));
    props.insert("headers".into(), Value::Object(h_idx));

    let idx = interp.heap.len();
    interp.heap.push(HeapValue::Object(JsObject {
        properties: props,
        prototype: None,
        extensible: true,
    }));
    Ok(Value::Object(idx))
}

pub(crate) fn native_response_constructor(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let body = args.first().cloned().unwrap_or(Value::Undefined);
    let body_str = match &body {
        Value::Null | Value::Undefined => None,
        Value::String(s) => Some(s.clone()),
        _ => Some(to_string_value(interp, &body)),
    };

    let mut status = 200;
    let mut status_text = "OK".to_string();
    let mut headers_raw = String::new();

    if let Some(Value::Object(init_idx)) = args.get(1) {
        if let HeapValue::Object(obj) = &interp.heap[*init_idx] {
            if let Some(s) = obj.properties.get("status") {
                status = match s {
                    Value::Integer(n) => *n as u16,
                    Value::Float(n) => *n as u16,
                    _ => 200,
                };
            }
            if let Some(st) = obj.properties.get("statusText") {
                status_text = to_string_value(interp, st);
            }
            if let Some(Value::Object(hdr_idx)) = obj.properties.get("headers") {
                if let HeapValue::Object(hdr_obj) = &interp.heap[*hdr_idx] {
                    if let Some(Value::String(h)) = hdr_obj.properties.get("__headers") {
                        headers_raw = h.clone();
                    } else {
                        let mut header_strs = Vec::new();
                        for (k, v) in &hdr_obj.properties {
                            if !k.starts_with('_') {
                                let val = to_string_value(interp, v);
                                header_strs.push(format!("{}\0{}", k.to_lowercase(), val));
                            }
                        }
                        headers_raw = header_strs.join("\n");
                    }
                }
            }
        }
    }

    build_response(
        interp,
        body_str.unwrap_or_default(),
        status,
        &status_text,
        &headers_raw,
    )
}

pub(crate) fn native_response_json_static(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let data = args.first().cloned().unwrap_or(Value::Undefined);
    let json_str = crate::runtime_env::native_fns::helpers::to_json_value(interp, &data);

    let mut status = 200;
    let mut status_text = "OK".to_string();
    let headers_raw = format!("{}\0{}", "content-type", "application/json");

    if let Some(Value::Object(init_idx)) = args.get(1) {
        if let HeapValue::Object(obj) = &interp.heap[*init_idx] {
            if let Some(s) = obj.properties.get("status") {
                status = match s {
                    Value::Integer(n) => *n as u16,
                    Value::Float(n) => *n as u16,
                    _ => 200,
                };
            }
            if let Some(st) = obj.properties.get("statusText") {
                status_text = to_string_value(interp, st);
            }
        }
    }

    build_response(interp, json_str, status, &status_text, &headers_raw)
}

pub(crate) fn native_response_error(
    interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    build_response(interp, String::new(), 0, "", "")
}

pub(crate) fn native_response_redirect(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let url = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    let status = args
        .get(1)
        .map(|v| match v {
            Value::Integer(n) => *n as u16,
            Value::Float(n) => *n as u16,
            _ => 302,
        })
        .unwrap_or(302);

    let headers_raw = format!("{}\0{}", "location", url);
    build_response(interp, String::new(), status, "Redirect", &headers_raw)
}

pub(crate) fn native_response_clone(
    interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    if let Value::Object(obj_idx) = _this {
        if let HeapValue::Object(obj) = &interp.heap[*obj_idx] {
            let body = obj
                .properties
                .get("__body")
                .map(|v| {
                    if let Value::String(s) = v {
                        s.clone()
                    } else {
                        String::new()
                    }
                })
                .unwrap_or_default();
            let status = obj
                .properties
                .get("status")
                .map(|v| match v {
                    Value::Integer(n) => *n as u16,
                    Value::Float(n) => *n as u16,
                    _ => 200,
                })
                .unwrap_or(200);
            let status_text = obj
                .properties
                .get("statusText")
                .map(|v| to_string_value(interp, v))
                .unwrap_or_else(|| "OK".to_string());
            let headers_raw = obj
                .properties
                .get("__headers")
                .map(|v| {
                    if let Value::String(s) = v {
                        s.clone()
                    } else {
                        String::new()
                    }
                })
                .unwrap_or_default();
            return build_response(interp, body, status, &status_text, &headers_raw);
        }
    }
    build_response(interp, String::new(), 200, "OK", "")
}

pub(crate) fn native_response_text(
    interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    if let Value::Object(obj_idx) = _this {
        if let HeapValue::Object(obj) = &interp.heap[*obj_idx] {
            if let Some(body) = get_string_prop(obj, "__body") {
                return Ok(Value::String(body.clone()));
            }
        }
    }
    Ok(Value::String(String::new()))
}

pub(crate) fn native_response_json(
    interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    if let Value::Object(obj_idx) = _this {
        if let HeapValue::Object(obj) = &interp.heap[*obj_idx] {
            if let Some(body) = get_string_prop(obj, "__body") {
                let json_val: serde_json::Value = serde_json::from_str(&body)
                    .map_err(|e| Error::RuntimeError(format!("JSON parse error: {}", e)))?;
                return Ok(crate::runtime_env::native_fns::helpers::from_json_value(interp, json_val));
            }
        }
    }
    Ok(Value::Null)
}

pub(crate) fn native_response_array_buffer(
    interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    if let Value::Object(obj_idx) = _this {
        if let HeapValue::Object(obj) = &interp.heap[*obj_idx] {
            if let Some(body) = get_string_prop(obj, "__body") {
                let bytes = body.as_bytes().to_vec();
                let buf_idx = interp.heap.len();
                interp.heap.push(HeapValue::Buffer(bytes));
                return Ok(Value::Buffer(buf_idx));
            }
        }
    }
    let buf_idx = interp.heap.len();
    interp.heap.push(HeapValue::Buffer(Vec::new()));
    Ok(Value::Buffer(buf_idx))
}
