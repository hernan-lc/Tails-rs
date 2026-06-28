use crate::errors::{Error, Result};
use crate::objects::Value;
use crate::vm::interpreter::{HeapValue, Interpreter, JsObject};

use super::helpers::to_string_value;

pub(super) fn native_fetch(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let url = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();

    // Parse options if provided
    let (method, headers, body) = if let Some(Value::Object(opts_idx)) = args.get(1) {
        if let HeapValue::Object(obj) = &interp.heap[*opts_idx] {
            let method = obj
                .properties
                .get("method")
                .and_then(|v| if let Value::String(s) = v { Some(s.clone()) } else { None })
                .unwrap_or_else(|| "GET".to_string());

            let mut headers = std::collections::HashMap::new();
            if let Some(Value::Object(hdr_idx)) = obj.properties.get("headers") {
                if let HeapValue::Object(hdr_obj) = &interp.heap[*hdr_idx] {
                    for (k, v) in &hdr_obj.properties {
                        headers.insert(k.clone(), to_string_value(interp, v));
                    }
                }
            }

            let body = obj
                .properties
                .get("body")
                .map(|v| to_string_value(interp, v));

            (method, headers, body)
        } else {
            ("GET".to_string(), std::collections::HashMap::new(), None)
        }
    } else {
        ("GET".to_string(), std::collections::HashMap::new(), None)
    };

    // Build the request using reqwest blocking client
    let client = reqwest::blocking::Client::new();
    let mut req = match method.to_uppercase().as_str() {
        "POST" => client.post(&url),
        "PUT" => client.put(&url),
        "DELETE" => client.delete(&url),
        "PATCH" => client.patch(&url),
        "HEAD" => client.head(&url),
        _ => client.get(&url),
    };

    // Add headers
    for (key, value) in &headers {
        req = req.header(key.as_str(), value.as_str());
    }

    // Add body
    if let Some(body) = body {
        req = req.body(body);
    }

    // Execute request
    let response = req.send().map_err(|e| {
        Error::RuntimeError(format!("fetch failed: {}", e))
    })?;

    // Build Response object
    let status = response.status().as_u16();
    let status_text = response.status().canonical_reason().unwrap_or("Unknown").to_string();

    let mut resp_props = std::collections::HashMap::new();
    resp_props.insert("status".into(), Value::Integer(status as i64));
    resp_props.insert("statusText".into(), Value::String(status_text));
    resp_props.insert("ok".into(), Value::Boolean(status >= 200 && status < 300));
    resp_props.insert("url".into(), Value::String(url));

    // Read response body
    let body_text = response.text().unwrap_or_default();

    // Add methods
    resp_props.insert("text".into(), Value::NativeFunction(298));
    resp_props.insert("json".into(), Value::NativeFunction(299));
    resp_props.insert("arrayBuffer".into(), Value::NativeFunction(300));

    // Store body for text()/json() methods
    resp_props.insert("__body".into(), Value::String(body_text));

    let resp_idx = interp.heap.len();
    interp.heap.push(HeapValue::Object(JsObject {
        properties: resp_props,
        prototype: None,
        extensible: true,
    }));

    Ok(Value::Object(resp_idx))
}

pub(super) fn native_response_text(
    interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    if let Value::Object(obj_idx) = _this {
        if let HeapValue::Object(obj) = &interp.heap[*obj_idx] {
            if let Some(Value::String(body)) = obj.properties.get("__body") {
                return Ok(Value::String(body.clone()));
            }
        }
    }
    Ok(Value::String(String::new()))
}

pub(super) fn native_response_json(
    interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    if let Value::Object(obj_idx) = _this {
        if let HeapValue::Object(obj) = &interp.heap[*obj_idx] {
            if let Some(Value::String(body)) = obj.properties.get("__body") {
                let json_val: serde_json::Value = serde_json::from_str(body)
                    .map_err(|e| Error::RuntimeError(format!("JSON parse error: {}", e)))?;
                return Ok(super::helpers::from_json_value(interp, json_val));
            }
        }
    }
    Ok(Value::Null)
}

pub(super) fn native_response_array_buffer(
    interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    if let Value::Object(obj_idx) = _this {
        if let HeapValue::Object(obj) = &interp.heap[*obj_idx] {
            if let Some(Value::String(body)) = obj.properties.get("__body") {
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
