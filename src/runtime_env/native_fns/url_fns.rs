use crate::errors::{Error, Result};
use crate::objects::Value;
use crate::runtime_env::native_fns::constants as c;
use crate::vm::interpreter::{HeapValue, Interpreter, JsObject};
use crate::props;

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

    let parsed =
        url::Url::parse(&url_str).map_err(|e| Error::TypeError(format!("Invalid URL: {}", e)))?;

    let query_str = parsed.query().unwrap_or("").to_string();

    // Create searchParams first (heap allocation)
    let search_params_idx = create_search_params(interp, &query_str);

    // Build all properties
    let search = if !query_str.is_empty() {
        format!("?{}", query_str)
    } else {
        String::new()
    };
    let hash = if parsed.fragment().is_some() {
        format!("#{}", parsed.fragment().unwrap_or(""))
    } else {
        String::new()
    };

    let props = props! {
        "href" => Value::String(parsed.as_str().to_string()),
        "origin" => Value::String(parsed.origin().ascii_serialization()),
        "protocol" => Value::String(parsed.scheme().to_string() + ":"),
        "host" => Value::String(parsed.host_str().unwrap_or("").to_string()),
        "hostname" => Value::String(parsed.host_str().unwrap_or("").to_string()),
        "port" => Value::String(parsed.port().map(|p| p.to_string()).unwrap_or_default()),
        "pathname" => Value::String(parsed.path().to_string()),
        "search" => Value::String(search),
        "hash" => Value::String(hash),
        "searchParams" => Value::Object(search_params_idx),
        "toString" => Value::NativeFunction(c::URL_TO_STRING),
        "toJSON" => Value::NativeFunction(c::URL_TO_JSON),
    };

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
    let size = query.split('&').filter(|s| !s.is_empty()).count() as i64;
    let props = props! {
        "__entries" => Value::String(query.to_string()),
        "size" => Value::Integer(size),
        "get" => Value::NativeFunction(c::SEARCH_PARAMS_GET),
        "getAll" => Value::NativeFunction(c::SEARCH_PARAMS_GET_ALL),
        "has" => Value::NativeFunction(c::SEARCH_PARAMS_HAS),
        "set" => Value::NativeFunction(c::SEARCH_PARAMS_SET),
        "append" => Value::NativeFunction(c::SEARCH_PARAMS_APPEND),
        "delete" => Value::NativeFunction(c::SEARCH_PARAMS_DELETE),
        "toString" => Value::NativeFunction(c::SEARCH_PARAMS_TO_STRING),
        "entries" => Value::NativeFunction(c::SEARCH_PARAMS_ENTRIES),
        "keys" => Value::NativeFunction(c::SEARCH_PARAMS_KEYS),
        "values" => Value::NativeFunction(c::SEARCH_PARAMS_VALUES),
        "forEach" => Value::NativeFunction(c::SEARCH_PARAMS_FOR_EACH),
    };

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
    let key = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
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
    let key = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
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
    interp
        .heap
        .push(HeapValue::Array(crate::vm::interpreter::JsArray {
            elements: values,
        }));
    Ok(Value::Array(arr_idx))
}

pub(super) fn native_search_params_has(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let key = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    if let Value::Object(obj_idx) = _this {
        if let HeapValue::Object(obj) = &interp.heap[*obj_idx] {
            if let Some(Value::String(entries_str)) = obj.properties.get("__entries") {
                for pair in entries_str.split('&') {
                    if let Some(k) = pair.split('=').next() {
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
    let key = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    let value = args
        .get(1)
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
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
    let key = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    let value = args
        .get(1)
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
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
    let key = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
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
        interp
            .heap
            .push(HeapValue::Array(crate::vm::interpreter::JsArray {
                elements: vec![Value::String(key), Value::String(value)],
            }));
        entries.push(Value::Array(entry_idx));
    }
    let arr_idx = interp.heap.len();
    interp
        .heap
        .push(HeapValue::Array(crate::vm::interpreter::JsArray {
            elements: entries,
        }));
    Ok(Value::Array(arr_idx))
}

pub(super) fn native_search_params_keys(
    interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let entries_data: Vec<(String, String)> = parse_entries(interp, _this);
    let keys: Vec<Value> = entries_data
        .into_iter()
        .map(|(k, _)| Value::String(k))
        .collect();
    let arr_idx = interp.heap.len();
    interp
        .heap
        .push(HeapValue::Array(crate::vm::interpreter::JsArray {
            elements: keys,
        }));
    Ok(Value::Array(arr_idx))
}

pub(super) fn native_search_params_values(
    interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let entries_data: Vec<(String, String)> = parse_entries(interp, _this);
    let vals: Vec<Value> = entries_data
        .into_iter()
        .map(|(_, v)| Value::String(v))
        .collect();
    let arr_idx = interp.heap.len();
    interp
        .heap
        .push(HeapValue::Array(crate::vm::interpreter::JsArray {
            elements: vals,
        }));
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
            &[
                Value::String(value.clone()),
                Value::String(key.clone()),
                _this.clone(),
            ],
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
                let new_str: Vec<String> = entries
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect();
                let _ = obj
                    .properties
                    .insert("__entries".into(), Value::String(new_str.join("&")));
                let _ = obj
                    .properties
                    .insert("size".into(), Value::Integer(entries.len() as i64));
            }
        }
    }
}

// --- URLSearchParams standalone constructor (index 366) ---

pub(super) fn native_url_search_params_constructor(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let init = args.first().cloned().unwrap_or(Value::Undefined);
    let query_str = match &init {
        Value::String(s) => s.clone(),
        Value::Object(obj_idx) => {
            if let HeapValue::Object(obj) = &interp.heap[*obj_idx] {
                // If it has __entries, it's already a URLSearchParams
                if let Some(Value::String(entries)) = obj.properties.get("__entries") {
                    entries.clone()
                } else {
                    // Object: key=value pairs
                    let pairs: Vec<String> = obj
                        .properties
                        .iter()
                        .filter(|(k, _)| !k.starts_with('_'))
                        .map(|(k, v)| {
                            let val = super::helpers::to_string_value(interp, v);
                            format!("{}={}", k, urlencoding::encode(&val))
                        })
                        .collect();
                    pairs.join("&")
                }
            } else {
                String::new()
            }
        }
        Value::Array(arr_idx) => {
            if let HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                // Array of [key, value] pairs
                let pairs: Vec<String> = arr
                    .elements
                    .iter()
                    .filter_map(|elem| {
                        if let Value::Array(pair_idx) = elem {
                            if let HeapValue::Array(pair) = &interp.heap[*pair_idx] {
                                if pair.elements.len() >= 2 {
                                    let k =
                                        super::helpers::to_string_value(interp, &pair.elements[0]);
                                    let v =
                                        super::helpers::to_string_value(interp, &pair.elements[1]);
                                    return Some(format!("{}={}", k, urlencoding::encode(&v)));
                                }
                            }
                        }
                        None
                    })
                    .collect();
                pairs.join("&")
            } else {
                String::new()
            }
        }
        Value::Null | Value::Undefined => String::new(),
        _ => String::new(),
    };

    let idx = create_search_params(interp, &query_str);
    Ok(Value::Object(idx))
}

// --- URL.canParse (index 367) ---

pub(super) fn native_url_can_parse(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let url_str = args
        .first()
        .map(|v| super::helpers::to_string_value(interp, v))
        .unwrap_or_default();

    let base = args
        .get(1)
        .map(|v| super::helpers::to_string_value(interp, v));

    let result = if let Some(base_str) = base {
        url::Url::parse(&base_str)
            .ok()
            .and_then(|base_url| base_url.join(&url_str).ok())
            .is_some()
    } else {
        url::Url::parse(&url_str).is_ok()
    };

    Ok(Value::Boolean(result))
}

// --- URL.parse (index 368) ---

pub(super) fn native_url_parse(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let url_str = args
        .first()
        .map(|v| super::helpers::to_string_value(interp, v))
        .unwrap_or_default();

    let base = args
        .get(1)
        .map(|v| super::helpers::to_string_value(interp, v));

    let parsed = if let Some(base_str) = base {
        url::Url::parse(&base_str)
            .ok()
            .and_then(|base_url| base_url.join(&url_str).ok())
    } else {
        url::Url::parse(&url_str).ok()
    };

    match parsed {
        Some(url) => {
            // Reuse native_url_constructor logic
            native_url_constructor(interp, &Value::Undefined, &[Value::String(url.to_string())])
        }
        None => Ok(Value::Null),
    }
}

// --- URL.toJSON (index 369) ---

pub(super) fn native_url_to_json(
    interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    native_url_to_string(interp, _this, &[])
}

// --- fileURLToPath (index 389) ---

pub(super) fn native_url_file_url_to_path(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let url_str = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();

    if let Some(path) = url_str.strip_prefix("file://") {
        // On Unix, percent-decode the path
        let decoded = urlencoding::decode(path)
            .unwrap_or(std::borrow::Cow::Borrowed(path))
            .to_string();
        // Normalize the path (remove . and .. segments)
        let normalized = std::path::Path::new(&decoded)
            .components()
            .collect::<std::path::PathBuf>();
        Ok(Value::String(normalized.to_string_lossy().into_owned()))
    } else {
        Err(Error::TypeError(format!("Invalid file URL: {}", url_str)))
    }
}
