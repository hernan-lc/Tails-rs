use crate::objects::Value;
use crate::props;
use crate::runtime_env::native_fns::constants as c;
use crate::runtime_env::native_fns::helpers::to_string_value;
use crate::vm::interpreter::{HeapValue, Interpreter, JsObject, PropertyStorage};
use rustc_hash::FxHashMap;

pub(crate) fn create_headers_props(headers_raw: &str) -> PropertyStorage {
    let mut props = FxHashMap::default();
    props.insert(
        "__headers".into(),
        Value::from_string(headers_raw.to_string()),
    );
    props.insert("append".into(), Value::NativeFunction(c::HEADERS_APPEND));
    props.insert("get".into(), Value::NativeFunction(c::HEADERS_GET));
    props.insert("set".into(), Value::NativeFunction(c::HEADERS_SET));
    props.insert("has".into(), Value::NativeFunction(c::HEADERS_HAS));
    props.insert("delete".into(), Value::NativeFunction(c::HEADERS_DELETE));
    props.insert("forEach".into(), Value::NativeFunction(c::HEADERS_FOR_EACH));
    props.insert("keys".into(), Value::NativeFunction(c::HEADERS_KEYS));
    props.insert("values".into(), Value::NativeFunction(c::HEADERS_VALUES));
    props.insert("entries".into(), Value::NativeFunction(c::HEADERS_ENTRIES));
    PropertyStorage::Map(props)
}

pub(crate) fn get_string_prop(obj: &crate::vm::interpreter::JsObject, key: &str) -> Option<String> {
    obj.properties.get(key).and_then(|v| {
        if let Value::String(s) = v {
            Some(s.to_string())
        } else {
            None
        }
    })
}

pub(crate) fn parse_headers(raw: &str) -> Vec<(String, String)> {
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

pub(crate) fn get_headers_string(interp: &Interpreter, this: &Value) -> String {
    if let Value::Object(obj_idx) = this {
        if let HeapValue::Object(obj) = &interp.heap[*obj_idx] {
            if let Some(h) = get_string_prop(obj, "__headers") {
                return h.clone();
            }
        }
    }
    String::new()
}

pub(crate) fn modify_headers<F>(interp: &mut Interpreter, this: &Value, f: F)
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
                        Some(s.to_string())
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
            obj.properties.insert(
                "__headers".into(),
                Value::from_string(new_raw.join("\n")),
            );
        }
    }
}

pub(crate) fn native_headers_constructor(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> crate::errors::Result<Value> {
    let mut props = FxHashMap::default();
    props.insert("__headers".into(), Value::string(""));

    if let Some(init) = args.first() {
        match init {
            Value::Object(obj_idx) => {
                if let HeapValue::Object(obj) = &interp.heap[*obj_idx] {
                    if let Some(h) = get_string_prop(obj, "__headers") {
                        props.insert("__headers".into(), Value::from_string(h.clone()));
                    } else {
                        let mut header_strs = Vec::new();
                        for (k, v) in &obj.properties {
                            if !k.starts_with('_') && !k.starts_with('[') {
                                let val = to_string_value(interp, v);
                                header_strs.push(format!("{}\0{}", k.to_lowercase(), val));
                            }
                        }
                        props.insert(
                            "__headers".into(),
                            Value::from_string(header_strs.join("\n")),
                        );
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
                    props.insert(
                        "__headers".into(),
                        Value::from_string(header_strs.join("\n")),
                    );
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

pub(crate) fn native_headers_append(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> crate::errors::Result<Value> {
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

pub(crate) fn native_headers_get(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> crate::errors::Result<Value> {
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
        Ok(Value::from_string(values.join(", ")))
    }
}

pub(crate) fn native_headers_set(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> crate::errors::Result<Value> {
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

pub(crate) fn native_headers_has(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> crate::errors::Result<Value> {
    let key = args
        .first()
        .map(|v| to_string_value(interp, v).to_lowercase())
        .unwrap_or_default();
    let raw = get_headers_string(interp, _this);
    let entries = parse_headers(&raw);
    Ok(Value::Boolean(entries.iter().any(|(k, _)| k == &key)))
}

pub(crate) fn native_headers_delete(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> crate::errors::Result<Value> {
    let key = args
        .first()
        .map(|v| to_string_value(interp, v).to_lowercase())
        .unwrap_or_default();
    modify_headers(interp, _this, |entries| {
        entries.retain(|(k, _)| k != &key);
    });
    Ok(Value::Undefined)
}

pub(crate) fn native_headers_for_each(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> crate::errors::Result<Value> {
    let callback = args.first().cloned().unwrap_or(Value::Undefined);
    let raw = get_headers_string(interp, _this);
    let entries = parse_headers(&raw);
    for (key, value) in &entries {
        let _ = interp.call_value(
            &callback,
            &Value::Undefined,
            &[
                Value::from_string(value.clone()),
                Value::from_string(key.clone()),
                _this.clone(),
            ],
        );
    }
    Ok(Value::Undefined)
}

pub(crate) fn native_headers_keys(
    interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> crate::errors::Result<Value> {
    let raw = get_headers_string(interp, _this);
    let entries = parse_headers(&raw);
    let keys: Vec<Value> = entries
        .into_iter()
        .map(|(k, _)| Value::from_string(k))
        .collect();
    let arr_idx = interp.heap.len();
    interp
        .heap
        .push(HeapValue::Array(crate::vm::interpreter::JsArray {
            elements: keys,
        }));
    Ok(Value::Array(arr_idx))
}

pub(crate) fn native_headers_values(
    interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> crate::errors::Result<Value> {
    let raw = get_headers_string(interp, _this);
    let entries = parse_headers(&raw);
    let vals: Vec<Value> = entries
        .into_iter()
        .map(|(_, v)| Value::from_string(v))
        .collect();
    let arr_idx = interp.heap.len();
    interp
        .heap
        .push(HeapValue::Array(crate::vm::interpreter::JsArray {
            elements: vals,
        }));
    Ok(Value::Array(arr_idx))
}

pub(crate) fn native_headers_entries(
    interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> crate::errors::Result<Value> {
    let raw = get_headers_string(interp, _this);
    let entries = parse_headers(&raw);
    let mut result = Vec::with_capacity(entries.len());
    for (k, v) in entries {
        let pair_idx = interp.heap.len();
        interp
            .heap
            .push(HeapValue::Array(crate::vm::interpreter::JsArray {
                elements: vec![Value::from_string(k), Value::from_string(v)],
            }));
        result.push(Value::Array(pair_idx));
    }
    let arr_idx = interp.heap.len();
    interp
        .heap
        .push(HeapValue::Array(crate::vm::interpreter::JsArray {
            elements: result,
        }));
    Ok(Value::Array(arr_idx))
}
