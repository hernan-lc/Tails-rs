use crate::errors::{Error, Result};
use crate::objects::js_promise::JsPromise;
use crate::objects::Value;
use crate::props;
use crate::runtime_env::native_fns::constants as c;
use crate::vm::interpreter::{HeapValue, Interpreter, JsArray, JsObject, PropertyStorage};
use rustc_hash::FxHashMap;

use super::helpers::to_string_value;

// ============================================================
// Headers (index 370-379)
// ============================================================

pub(super) fn native_headers_constructor(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let mut props = FxHashMap::default();
    // Store headers as a flat string "key1\0value1\nkey2\0value2"
    // This preserves multiple values for the same key
    props.insert("__headers".into(), Value::String(String::new()));

    if let Some(init) = args.first() {
        match init {
            Value::Object(obj_idx) => {
                if let HeapValue::Object(obj) = &interp.heap[*obj_idx] {
                    // Check if it's another Headers instance
                    if let Some(Value::String(h)) = obj.properties.get("__headers") {
                        props.insert("__headers".into(), Value::String(h.clone()));
                    } else {
                        // Plain object: key-value pairs
                        let mut header_strs = Vec::new();
                        for (k, v) in &obj.properties {
                            if !k.starts_with('_') && !k.starts_with('[') {
                                let val = to_string_value(interp, v);
                                header_strs.push(format!("{}\0{}", k.to_lowercase(), val));
                            }
                        }
                        props.insert("__headers".into(), Value::String(header_strs.join("\n")));
                    }
                }
            }
            Value::Array(arr_idx) => {
                if let HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                    let mut header_strs = Vec::new();
                    for elem in &arr.elements {
                        if let Value::Array(pair_idx) = elem {
                            if let HeapValue::Array(pair) = &interp.heap[*pair_idx] {
                                if pair.elements.len() >= 2 {
                                    let k =
                                        to_string_value(interp, &pair.elements[0]).to_lowercase();
                                    let v = to_string_value(interp, &pair.elements[1]);
                                    header_strs.push(format!("{}\0{}", k, v));
                                }
                            }
                        }
                    }
                    props.insert("__headers".into(), Value::String(header_strs.join("\n")));
                }
            }
            _ => {}
        }
    }

    let static_props = props! {
        "append" => Value::NativeFunction(c::HEADERS_APPEND),
        "get" => Value::NativeFunction(c::HEADERS_GET),
        "set" => Value::NativeFunction(c::HEADERS_SET),
        "has" => Value::NativeFunction(c::HEADERS_HAS),
        "delete" => Value::NativeFunction(c::HEADERS_DELETE),
        "forEach" => Value::NativeFunction(c::HEADERS_FOR_EACH),
        "keys" => Value::NativeFunction(c::HEADERS_KEYS),
        "values" => Value::NativeFunction(c::HEADERS_VALUES),
        "entries" => Value::NativeFunction(c::HEADERS_ENTRIES),
    };
    for (k, v) in &static_props {
        props.insert(k.to_string(), v.clone());
    }

    let idx = interp.heap.len();
    interp.heap.push(HeapValue::Object(JsObject {
        properties: PropertyStorage::Map(props),
        prototype: None,
        extensible: true,
    }));
    Ok(Value::Object(idx))
}

fn parse_headers(raw: &str) -> Vec<(String, String)> {
    raw.split('\n')
        .filter(|s| !s.is_empty())
        .filter_map(|line| {
            let mut parts = line.splitn(2, '\0');
            let k = parts.next()?.to_string();
            let v = parts.next().unwrap_or("").to_string();
            Some((k, v))
        })
        .collect()
}

fn get_headers_string(interp: &Interpreter, this: &Value) -> String {
    if let Value::Object(obj_idx) = this {
        if let HeapValue::Object(obj) = &interp.heap[*obj_idx] {
            if let Some(Value::String(h)) = obj.properties.get("__headers") {
                return h.clone();
            }
        }
    }
    String::new()
}

fn modify_headers<F>(interp: &mut Interpreter, this: &Value, f: F)
where
    F: FnOnce(&mut Vec<(String, String)>),
{
    if let Value::Object(obj_idx) = this {
        if let HeapValue::Object(obj) = &mut interp.heap[*obj_idx] {
            let raw = obj
                .properties
                .get("__headers")
                .and_then(|v| {
                    if let Value::String(s) = v {
                        Some(s.clone())
                    } else {
                        None
                    }
                })
                .unwrap_or_default();
            let mut entries = parse_headers(&raw);
            f(&mut entries);
            let new_raw: Vec<String> = entries
                .iter()
                .map(|(k, v)| format!("{}\0{}", k, v))
                .collect();
            obj.properties
                .insert("__headers".into(), Value::String(new_raw.join("\n")));
        }
    }
}

pub(super) fn native_headers_append(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let key = args
        .first()
        .map(|v| to_string_value(interp, v).to_lowercase())
        .unwrap_or_default();
    let value = args
        .get(1)
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    modify_headers(interp, _this, |entries| {
        entries.push((key, value));
    });
    Ok(Value::Undefined)
}

pub(super) fn native_headers_get(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let key = args
        .first()
        .map(|v| to_string_value(interp, v).to_lowercase())
        .unwrap_or_default();
    let raw = get_headers_string(interp, _this);
    let entries = parse_headers(&raw);
    let values: Vec<&str> = entries
        .iter()
        .filter(|(k, _)| k == &key)
        .map(|(_, v)| v.as_str())
        .collect();
    if values.is_empty() {
        Ok(Value::Null)
    } else {
        Ok(Value::String(values.join(", ")))
    }
}

pub(super) fn native_headers_set(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let key = args
        .first()
        .map(|v| to_string_value(interp, v).to_lowercase())
        .unwrap_or_default();
    let value = args
        .get(1)
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    modify_headers(interp, _this, |entries| {
        entries.retain(|(k, _)| k != &key);
        entries.push((key, value));
    });
    Ok(Value::Undefined)
}

pub(super) fn native_headers_has(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let key = args
        .first()
        .map(|v| to_string_value(interp, v).to_lowercase())
        .unwrap_or_default();
    let raw = get_headers_string(interp, _this);
    let entries = parse_headers(&raw);
    Ok(Value::Boolean(entries.iter().any(|(k, _)| k == &key)))
}

pub(super) fn native_headers_delete(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let key = args
        .first()
        .map(|v| to_string_value(interp, v).to_lowercase())
        .unwrap_or_default();
    modify_headers(interp, _this, |entries| {
        entries.retain(|(k, _)| k != &key);
    });
    Ok(Value::Undefined)
}

pub(super) fn native_headers_for_each(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let callback = args.first().cloned().unwrap_or(Value::Undefined);
    let raw = get_headers_string(interp, _this);
    let entries = parse_headers(&raw);
    for (key, value) in &entries {
        let _ = interp.call_value(
            &callback,
            &Value::Undefined,
            &[
                Value::String(value.clone()),
                Value::String(key.clone()),
                _this.clone(),
            ],
        );
    }
    Ok(Value::Undefined)
}

pub(super) fn native_headers_keys(
    interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let raw = get_headers_string(interp, _this);
    let entries = parse_headers(&raw);
    let keys: Vec<Value> = entries.into_iter().map(|(k, _)| Value::String(k)).collect();
    let arr_idx = interp.heap.len();
    interp
        .heap
        .push(HeapValue::Array(JsArray { elements: keys }));
    Ok(Value::Array(arr_idx))
}

pub(super) fn native_headers_values(
    interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let raw = get_headers_string(interp, _this);
    let entries = parse_headers(&raw);
    let vals: Vec<Value> = entries.into_iter().map(|(_, v)| Value::String(v)).collect();
    let arr_idx = interp.heap.len();
    interp
        .heap
        .push(HeapValue::Array(JsArray { elements: vals }));
    Ok(Value::Array(arr_idx))
}

pub(super) fn native_headers_entries(
    interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let raw = get_headers_string(interp, _this);
    let entries = parse_headers(&raw);
    let mut result = Vec::with_capacity(entries.len());
    for (k, v) in entries {
        let pair_idx = interp.heap.len();
        interp.heap.push(HeapValue::Array(JsArray {
            elements: vec![Value::String(k), Value::String(v)],
        }));
        result.push(Value::Array(pair_idx));
    }
    let arr_idx = interp.heap.len();
    interp
        .heap
        .push(HeapValue::Array(JsArray { elements: result }));
    Ok(Value::Array(arr_idx))
}

// ============================================================
// Request (index 380)
// ============================================================

pub(super) fn native_request_constructor(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let url = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();

    let mut method = "GET".to_string();
    let mut headers_raw = String::new();
    let mut body: Option<String> = None;

    // If first arg is a Request object, clone its properties
    if let Some(Value::Object(obj_idx)) = args.first() {
        if let HeapValue::Object(obj) = &interp.heap[*obj_idx] {
            if obj.properties.contains_key("__is_request") {
                let cloned_url = obj
                    .properties
                    .get("url")
                    .map(|v| to_string_value(interp, v))
                    .unwrap_or_default();
                let cloned_method = obj
                    .properties
                    .get("method")
                    .map(|v| to_string_value(interp, v))
                    .unwrap_or_default();
                headers_raw = obj
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
                body = obj.properties.get("__body").and_then(|v| {
                    if let Value::String(s) = v {
                        Some(s.clone())
                    } else {
                        None
                    }
                });
                method = cloned_method;
                // Use the cloned URL but allow second arg to override
                let init_url = url;
                let _ = init_url; // url from first arg was already parsed

                // Process init options from second arg if present
                if let Some(Value::Object(init_idx)) = args.get(1) {
                    if let HeapValue::Object(init_obj) = &interp.heap[*init_idx] {
                        if let Some(m) = init_obj.properties.get("method") {
                            method = to_string_value(interp, m);
                        }
                        // ... other overrides handled below
                    }
                }

                let mut props = props! {
                    "url" => Value::String(cloned_url),
                    "method" => Value::String(method.to_uppercase()),
                    "__headers" => Value::String(headers_raw.clone()),
                    "__body" => body.clone().map(Value::String).unwrap_or(Value::Null),
                    "bodyUsed" => Value::Boolean(false),
                    "__is_request" => Value::Boolean(true),
                    "__method" => Value::String(method.to_uppercase()),
                };

                // Create headers object
                let h_idx = interp.heap.len();
                let h_props = props! {
                    "__headers" => Value::String(headers_raw),
                    "append" => Value::NativeFunction(c::HEADERS_APPEND),
                    "get" => Value::NativeFunction(c::HEADERS_GET),
                    "set" => Value::NativeFunction(c::HEADERS_SET),
                    "has" => Value::NativeFunction(c::HEADERS_HAS),
                    "delete" => Value::NativeFunction(c::HEADERS_DELETE),
                    "forEach" => Value::NativeFunction(c::HEADERS_FOR_EACH),
                    "keys" => Value::NativeFunction(c::HEADERS_KEYS),
                    "values" => Value::NativeFunction(c::HEADERS_VALUES),
                    "entries" => Value::NativeFunction(c::HEADERS_ENTRIES),
                };
                interp.heap.push(HeapValue::Object(JsObject {
                    properties: h_props,
                    prototype: None,
                    extensible: true,
                }));
                props.insert("headers".into(), Value::Object(h_idx));

    let idx = interp.heap.len();
    interp.heap.push(HeapValue::Object(JsObject {
        properties: props.into(),
        prototype: None,
        extensible: true,
    }));
    return Ok(Value::Object(idx));
}
        }
    }

    // Parse init options from second arg
    if let Some(Value::Object(init_idx)) = args.get(1) {
        if let HeapValue::Object(init_obj) = &interp.heap[*init_idx] {
            if let Some(m) = init_obj.properties.get("method") {
                method = to_string_value(interp, m);
            }
            if let Some(Value::Object(hdr_idx)) = init_obj.properties.get("headers") {
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
            if let Some(b) = init_obj.properties.get("body") {
                body = Some(to_string_value(interp, b));
            }
        }
    }

    let mut props = props! {
        "url" => Value::String(url),
        "method" => Value::String(method.to_uppercase()),
        "__headers" => Value::String(headers_raw.clone()),
        "__body" => body.map(Value::String).unwrap_or(Value::Null),
        "bodyUsed" => Value::Boolean(false),
        "__is_request" => Value::Boolean(true),
    };

    // Create headers object
    let h_idx = interp.heap.len();
    let h_props = props! {
        "__headers" => Value::String(headers_raw),
        "append" => Value::NativeFunction(c::HEADERS_APPEND),
        "get" => Value::NativeFunction(c::HEADERS_GET),
        "set" => Value::NativeFunction(c::HEADERS_SET),
        "has" => Value::NativeFunction(c::HEADERS_HAS),
        "delete" => Value::NativeFunction(c::HEADERS_DELETE),
        "forEach" => Value::NativeFunction(c::HEADERS_FOR_EACH),
        "keys" => Value::NativeFunction(c::HEADERS_KEYS),
        "values" => Value::NativeFunction(c::HEADERS_VALUES),
        "entries" => Value::NativeFunction(c::HEADERS_ENTRIES),
    };
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

// ============================================================
// Response (index 381-385)
// ============================================================

pub(super) fn native_response_constructor(
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

fn build_response(
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

    // Create headers object
    let h_idx = interp.heap.len();
    let h_props = props! {
        "__headers" => Value::String(headers_raw.to_string()),
        "append" => Value::NativeFunction(c::HEADERS_APPEND),
        "get" => Value::NativeFunction(c::HEADERS_GET),
        "set" => Value::NativeFunction(c::HEADERS_SET),
        "has" => Value::NativeFunction(c::HEADERS_HAS),
        "delete" => Value::NativeFunction(c::HEADERS_DELETE),
        "forEach" => Value::NativeFunction(c::HEADERS_FOR_EACH),
        "keys" => Value::NativeFunction(c::HEADERS_KEYS),
        "values" => Value::NativeFunction(c::HEADERS_VALUES),
        "entries" => Value::NativeFunction(c::HEADERS_ENTRIES),
    };
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

// Response.json() static method (index 382)
pub(super) fn native_response_json_static(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let data = args.first().cloned().unwrap_or(Value::Undefined);
    let json_str = super::helpers::to_json_value(interp, &data);

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

// Response.error() static method (index 383)
pub(super) fn native_response_error(
    interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    build_response(interp, String::new(), 0, "", "")
}

// Response.redirect() static method (index 384)
pub(super) fn native_response_redirect(
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

// Response.clone() instance method (index 385)
pub(super) fn native_response_clone(
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

// ============================================================
// Response body methods (existing indices 308-310)
// ============================================================

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

// ============================================================
// fetch (index 307) - now returns a Promise
// ============================================================

pub(super) fn native_fetch(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    // Parse input - can be a URL string or Request object
    let (url, method, headers_map, body) = if let Some(Value::Object(obj_idx)) = args.first() {
        if let HeapValue::Object(obj) = &interp.heap[*obj_idx] {
            if obj.properties.contains_key("__is_request") {
                // It's a Request object
                let url = obj
                    .properties
                    .get("url")
                    .map(|v| to_string_value(interp, v))
                    .unwrap_or_default();
                let method = obj
                    .properties
                    .get("method")
                    .map(|v| to_string_value(interp, v))
                    .unwrap_or_else(|| "GET".to_string());
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
                let headers_map = parse_headers_to_map(&headers_raw);
                let body = obj.properties.get("__body").and_then(|v| {
                    if let Value::String(s) = v {
                        if s.is_empty() {
                            None
                        } else {
                            Some(s.clone())
                        }
                    } else {
                        None
                    }
                });
                (url, method, headers_map, body)
            } else {
                // Treat as options object
                let url = to_string_value(interp, args.first().unwrap());
                let (method, headers_map, body) = parse_fetch_options(interp, args);
                (url, method, headers_map, body)
            }
        } else {
            let url = to_string_value(interp, args.first().unwrap());
            let (method, headers_map, body) = parse_fetch_options(interp, args);
            (url, method, headers_map, body)
        }
    } else {
        let url = args
            .first()
            .map(|v| to_string_value(interp, v))
            .unwrap_or_default();
        let (method, headers_map, body) = parse_fetch_args(interp, args);
        (url, method, headers_map, body)
    };

    // Execute request synchronously, wrap result in a fulfilled Promise
    let result = execute_fetch(interp, &url, &method, &headers_map, body.as_deref());

    match result {
        Ok(response_value) => {
            let promise = JsPromise::fulfilled(response_value);
            let promise_idx = interp.heap.len();
            interp.heap.push(HeapValue::Promise(promise));
            Ok(Value::Promise(promise_idx))
        }
        Err(e) => {
            let err_msg = Value::String(e.to_string());
            let promise = JsPromise::rejected(err_msg);
            let promise_idx = interp.heap.len();
            interp.heap.push(HeapValue::Promise(promise));
            Ok(Value::Promise(promise_idx))
        }
    }
}

fn parse_fetch_args(
    interp: &mut Interpreter,
    args: &[Value],
) -> (
    String,
    std::collections::HashMap<String, String>,
    Option<String>,
) {
    let method = "GET".to_string();
    let headers_map = std::collections::HashMap::new();
    let mut body = None;

    if let Some(Value::Object(opts_idx)) = args.get(1) {
        if let HeapValue::Object(obj) = &interp.heap[*opts_idx] {
            let m = obj
                .properties
                .get("method")
                .map(|v| to_string_value(interp, v))
                .unwrap_or_else(|| "GET".to_string());

            let mut hdrs = std::collections::HashMap::new();
            if let Some(Value::Object(hdr_idx)) = obj.properties.get("headers") {
                if let HeapValue::Object(hdr_obj) = &interp.heap[*hdr_idx] {
                    if let Some(Value::String(h)) = hdr_obj.properties.get("__headers") {
                        for (k, v) in parse_headers(h) {
                            hdrs.insert(k, v);
                        }
                    } else {
                        for (k, v) in &hdr_obj.properties {
                            if !k.starts_with('_') {
                                hdrs.insert(k.to_string(), to_string_value(interp, v));
                            }
                        }
                    }
                }
            }

            body = obj
                .properties
                .get("body")
                .map(|v| to_string_value(interp, v));

            return (m, hdrs, body);
        }
    }

    (method, headers_map, body)
}

fn parse_fetch_options(
    interp: &mut Interpreter,
    args: &[Value],
) -> (
    String,
    std::collections::HashMap<String, String>,
    Option<String>,
) {
    let method = "GET".to_string();
    let headers_map = std::collections::HashMap::new();
    let body = None;

    if let Some(Value::Object(opts_idx)) = args.get(1) {
        if let HeapValue::Object(obj) = &interp.heap[*opts_idx] {
            let m = obj
                .properties
                .get("method")
                .map(|v| to_string_value(interp, v))
                .unwrap_or_else(|| "GET".to_string());

            let mut hdrs = std::collections::HashMap::new();
            if let Some(Value::Object(hdr_idx)) = obj.properties.get("headers") {
                if let HeapValue::Object(hdr_obj) = &interp.heap[*hdr_idx] {
                    for (k, v) in &hdr_obj.properties {
                        if !k.starts_with('_') {
                            hdrs.insert(k.to_string(), to_string_value(interp, v));
                        }
                    }
                }
            }

            let b = obj
                .properties
                .get("body")
                .map(|v| to_string_value(interp, v));

            return (m, hdrs, b);
        }
    }

    (method, headers_map, body)
}

fn parse_headers_to_map(raw: &str) -> std::collections::HashMap<String, String> {
    parse_headers(raw).into_iter().collect()
}

fn execute_fetch(
    interp: &mut Interpreter,
    url: &str,
    method: &str,
    headers: &std::collections::HashMap<String, String>,
    body: Option<&str>,
) -> Result<Value> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()
        .map_err(|e| Error::RuntimeError(format!("Failed to create HTTP client: {}", e)))?;

    let mut req = match method.to_uppercase().as_str() {
        "POST" => client.post(url),
        "PUT" => client.put(url),
        "DELETE" => client.delete(url),
        "PATCH" => client.patch(url),
        "HEAD" => client.head(url),
        "OPTIONS" => client.request(reqwest::Method::OPTIONS, url),
        _ => client.get(url),
    };

    for (key, value) in headers {
        req = req.header(key.as_str(), value.as_str());
    }

    if let Some(body_str) = body {
        req = req.body(body_str.to_string());
    }

    let response = req
        .send()
        .map_err(|e| Error::RuntimeError(format!("fetch failed: {}", e)))?;

    let status = response.status().as_u16();
    let status_text = response
        .status()
        .canonical_reason()
        .unwrap_or("Unknown")
        .to_string();

    // Collect response headers
    let mut resp_headers = Vec::new();
    for (key, value) in response.headers() {
        if let Ok(val) = value.to_str() {
            resp_headers.push(format!("{}\0{}", key.as_str().to_lowercase(), val));
        }
    }
    let headers_raw = resp_headers.join("\n");

    let body_text = response.text().unwrap_or_default();

    build_response(interp, body_text, status, &status_text, &headers_raw)
}
