use crate::errors::{Error, Result};
use crate::objects::Value;
use crate::vm::interpreter::{HeapValue, Interpreter, JsObject};

use super::helpers::to_string_value;

pub(super) fn native_url_constructor(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let url_str = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();

    let parsed = url::Url::parse(&url_str)
        .map_err(|e| Error::TypeError(format!("Invalid URL: {}", e)))?;

    let query_str = parsed.query().unwrap_or("").to_string();

    // Create searchParams first (heap allocation)
    let search_params_idx = create_search_params(interp, &query_str);

    // Build all properties
    let mut props = std::collections::HashMap::new();
    props.insert("href".to_string(), Value::String(parsed.as_str().to_string()));
    props.insert("origin".to_string(), Value::String(parsed.origin().ascii_serialization()));
    props.insert("protocol".to_string(), Value::String(parsed.scheme().to_string() + ":"));
    props.insert("host".to_string(), Value::String(parsed.host_str().unwrap_or("").to_string()));
    props.insert("hostname".to_string(), Value::String(parsed.host_str().unwrap_or("").to_string()));
    props.insert("port".to_string(), Value::String(parsed.port().map(|p| p.to_string()).unwrap_or_default()));
    props.insert("pathname".to_string(), Value::String(parsed.path().to_string()));
    props.insert("search".to_string(), Value::String(if !query_str.is_empty() {
        format!("?{}", query_str)
    } else {
        String::new()
    }));
    props.insert("hash".to_string(), Value::String(if parsed.fragment().is_some() {
        format!("#{}", parsed.fragment().unwrap_or(""))
    } else {
        String::new()
    }));
    props.insert("searchParams".to_string(), Value::Object(search_params_idx));

    // Always create a new object
    let obj_idx = interp.heap.len();
    interp.heap.push(HeapValue::Object(JsObject {
        properties: props,
        prototype: None,
        extensible: true,
    }));
    Ok(Value::Object(obj_idx))
}

pub(super) fn native_url_to_string(
    interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    if let Value::Object(obj_idx) = _this {
        if let HeapValue::Object(obj) = &interp.heap[*obj_idx] {
            if let Some(Value::String(href)) = obj.properties.get("href") {
                return Ok(Value::String(href.clone()));
            }
        }
    }
    Ok(Value::String(String::new()))
}

fn create_search_params(interp: &mut Interpreter, query: &str) -> usize {
    let mut props = std::collections::HashMap::new();
    props.insert("__entries".into(), Value::String(query.to_string()));
    props.insert("size".into(), Value::Integer(
        query.split('&').filter(|s| !s.is_empty()).count() as i64
    ));
    props.insert("get".into(), Value::NativeFunction(275));
    props.insert("getAll".into(), Value::NativeFunction(276));
    props.insert("has".into(), Value::NativeFunction(277));
    props.insert("set".into(), Value::NativeFunction(278));
    props.insert("append".into(), Value::NativeFunction(279));
    props.insert("delete".into(), Value::NativeFunction(280));
    props.insert("toString".into(), Value::NativeFunction(281));
    props.insert("entries".into(), Value::NativeFunction(282));
    props.insert("keys".into(), Value::NativeFunction(283));
    props.insert("values".into(), Value::NativeFunction(284));
    props.insert("forEach".into(), Value::NativeFunction(285));

    let idx = interp.heap.len();
    interp.heap.push(HeapValue::Object(JsObject {
        properties: props,
        prototype: None,
        extensible: true,
    }));
    idx
}

pub(super) fn native_search_params_get(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let key = args.first().map(|v| to_string_value(interp, v)).unwrap_or_default();
    if let Value::Object(obj_idx) = _this {
        if let HeapValue::Object(obj) = &interp.heap[*obj_idx] {
            if let Some(Value::String(entries_str)) = obj.properties.get("__entries") {
                for pair in entries_str.split('&') {
                    let mut parts = pair.splitn(2, '=');
                    if let Some(k) = parts.next() {
                        if k == key {
                            let value = parts.next().unwrap_or("").to_string();
                            let decoded = urlencoding::decode(&value)
                                .unwrap_or(std::borrow::Cow::Owned(value.clone()))
                                .to_string();
                            return Ok(Value::String(decoded));
                        }
                    }
                }
            }
        }
    }
    Ok(Value::Null)
}

pub(super) fn native_search_params_get_all(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let key = args.first().map(|v| to_string_value(interp, v)).unwrap_or_default();
    let mut values = Vec::new();
    if let Value::Object(obj_idx) = _this {
        if let HeapValue::Object(obj) = &interp.heap[*obj_idx] {
            if let Some(Value::String(entries_str)) = obj.properties.get("__entries") {
                for pair in entries_str.split('&') {
                    let mut parts = pair.splitn(2, '=');
                    if let Some(k) = parts.next() {
                        if k == key {
                            let value = parts.next().unwrap_or("").to_string();
                            let decoded = urlencoding::decode(&value)
                                .unwrap_or(std::borrow::Cow::Owned(value.clone()))
                                .to_string();
                            values.push(Value::String(decoded));
                        }
                    }
                }
            }
        }
    }
    let arr_idx = interp.heap.len();
    interp.heap.push(HeapValue::Array(crate::vm::interpreter::JsArray { elements: values }));
    Ok(Value::Array(arr_idx))
}

pub(super) fn native_search_params_has(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let key = args.first().map(|v| to_string_value(interp, v)).unwrap_or_default();
    if let Value::Object(obj_idx) = _this {
        if let HeapValue::Object(obj) = &interp.heap[*obj_idx] {
            if let Some(Value::String(entries_str)) = obj.properties.get("__entries") {
                for pair in entries_str.split('&') {
                    if let Some(k) = pair.splitn(2, '=').next() {
                        if k == key {
                            return Ok(Value::Boolean(true));
                        }
                    }
                }
            }
        }
    }
    Ok(Value::Boolean(false))
}

pub(super) fn native_search_params_set(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let key = args.first().map(|v| to_string_value(interp, v)).unwrap_or_default();
    let value = args.get(1).map(|v| to_string_value(interp, v)).unwrap_or_default();
    update_entries(interp, _this, |entries| {
        entries.retain(|(k, _)| k != &key);
        entries.push((key.clone(), value.clone()));
    });
    Ok(Value::Undefined)
}

pub(super) fn native_search_params_append(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let key = args.first().map(|v| to_string_value(interp, v)).unwrap_or_default();
    let value = args.get(1).map(|v| to_string_value(interp, v)).unwrap_or_default();
    update_entries(interp, _this, |entries| {
        entries.push((key.clone(), value.clone()));
    });
    Ok(Value::Undefined)
}

pub(super) fn native_search_params_delete(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let key = args.first().map(|v| to_string_value(interp, v)).unwrap_or_default();
    update_entries(interp, _this, |entries| {
        entries.retain(|(k, _)| k != &key);
    });
    Ok(Value::Undefined)
}

pub(super) fn native_search_params_to_string(
    interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    if let Value::Object(obj_idx) = _this {
        if let HeapValue::Object(obj) = &interp.heap[*obj_idx] {
            if let Some(Value::String(entries_str)) = obj.properties.get("__entries") {
                return Ok(Value::String(entries_str.clone()));
            }
        }
    }
    Ok(Value::String(String::new()))
}

pub(super) fn native_search_params_entries(
    interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let entries_data: Vec<(String, String)> = parse_entries(interp, _this);
    let mut entries = Vec::new();
    for (key, value) in entries_data {
        let entry_idx = interp.heap.len();
        interp.heap.push(HeapValue::Array(crate::vm::interpreter::JsArray {
            elements: vec![Value::String(key), Value::String(value)],
        }));
        entries.push(Value::Array(entry_idx));
    }
    let arr_idx = interp.heap.len();
    interp.heap.push(HeapValue::Array(crate::vm::interpreter::JsArray { elements: entries }));
    Ok(Value::Array(arr_idx))
}

pub(super) fn native_search_params_keys(
    interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let entries_data: Vec<(String, String)> = parse_entries(interp, _this);
    let keys: Vec<Value> = entries_data.into_iter().map(|(k, _)| Value::String(k)).collect();
    let arr_idx = interp.heap.len();
    interp.heap.push(HeapValue::Array(crate::vm::interpreter::JsArray { elements: keys }));
    Ok(Value::Array(arr_idx))
}

pub(super) fn native_search_params_values(
    interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let entries_data: Vec<(String, String)> = parse_entries(interp, _this);
    let vals: Vec<Value> = entries_data.into_iter().map(|(_, v)| Value::String(v)).collect();
    let arr_idx = interp.heap.len();
    interp.heap.push(HeapValue::Array(crate::vm::interpreter::JsArray { elements: vals }));
    Ok(Value::Array(arr_idx))
}

pub(super) fn native_search_params_for_each(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let callback = args.first().cloned().unwrap_or(Value::Undefined);
    let entries_data: Vec<(String, String)> = parse_entries(interp, _this);
    for (key, value) in &entries_data {
        let _ = interp.call_value(
            &callback,
            &Value::Undefined,
            &[Value::String(value.clone()), Value::String(key.clone()), _this.clone()],
        );
    }
    Ok(Value::Undefined)
}

fn parse_entries(interp: &Interpreter, this: &Value) -> Vec<(String, String)> {
    if let Value::Object(obj_idx) = this {
        if let HeapValue::Object(obj) = &interp.heap[*obj_idx] {
            if let Some(Value::String(entries_str)) = obj.properties.get("__entries") {
                return entries_str
                    .split('&')
                    .filter(|s| !s.is_empty())
                    .filter_map(|pair| {
                        let mut parts = pair.splitn(2, '=');
                        let k = parts.next()?.to_string();
                        let v = parts.next().unwrap_or("").to_string();
                        Some((k, v))
                    })
                    .collect();
            }
        }
    }
    Vec::new()
}

fn update_entries<F>(interp: &mut Interpreter, this: &Value, mut f: F)
where
    F: FnMut(&mut Vec<(String, String)>),
{
    if let Value::Object(obj_idx) = this {
        if let HeapValue::Object(obj) = &mut interp.heap[*obj_idx] {
            if let Some(Value::String(entries_str)) = obj.properties.get("__entries") {
                let mut entries: Vec<(String, String)> = entries_str
                    .split('&')
                    .filter(|s| !s.is_empty())
                    .filter_map(|pair| {
                        let mut parts = pair.splitn(2, '=');
                        let k = parts.next()?.to_string();
                        let v = parts.next().unwrap_or("").to_string();
                        Some((k, v))
                    })
                    .collect();
                f(&mut entries);
                let new_str: Vec<String> = entries.iter().map(|(k, v)| format!("{}={}", k, v)).collect();
                let _ = obj.properties.insert("__entries".into(), Value::String(new_str.join("&")));
                let _ = obj.properties.insert("size".into(), Value::Integer(entries.len() as i64));
            }
        }
    }
}
