use crate::objects::Value;
use crate::vm::interpreter::Interpreter;
use crate::well_known as wk;
use rustc_hash::FxHashMap;

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
        Value::Cons(c) => c.flatten().parse::<f64>().unwrap_or(f64::NAN),
        _ => f64::NAN,
    }
}

#[allow(dead_code)]
pub(super) fn arg_f64(args: &[Value], idx: usize, default: f64) -> f64 {
    args.get(idx).map(to_f64).unwrap_or(default)
}

#[allow(dead_code)]
pub(super) fn first_arg(args: &[Value]) -> Value {
    args.first().cloned().unwrap_or(Value::Undefined)
}

pub(super) fn normalize_index(index: i64, len: i64) -> i64 {
    if index < 0 {
        (len + index).max(0)
    } else {
        index.min(len)
    }
}

pub(super) fn is_user_visible_key(k: &str) -> bool {
    !k.starts_with("__getter_") && !k.starts_with("__setter_") && !k.starts_with("__method_")
}

const GETTER_PREFIX: &str = "__getter_";
const SETTER_PREFIX: &str = "__setter_";
const METHOD_PREFIX: &str = "__method_";

/// Collect own enumerable string property names, including accessor properties.
/// Internal storage keys (`__getter_X`, `__setter_X`, `__method_X`, `__sym_*`)
/// are never returned as-is; accessors contribute their logical name once.
pub(super) fn collect_own_enumerable_keys(
    properties: &crate::vm::interpreter::PropertyStorage,
) -> Vec<String> {
    let mut keys = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for k in properties.keys() {
        if let Some(real) = k.strip_prefix(GETTER_PREFIX) {
            if !real.is_empty() && seen.insert(real.to_string()) {
                keys.push(real.to_string());
            }
        } else if let Some(real) = k.strip_prefix(SETTER_PREFIX) {
            if !real.is_empty() && seen.insert(real.to_string()) {
                keys.push(real.to_string());
            }
        } else if k.starts_with(METHOD_PREFIX) || k.starts_with("__sym_") {
            continue;
        } else if is_user_visible_key(k) && seen.insert(k.to_string()) {
            keys.push(k.to_string());
        }
    }
    keys
}

pub(super) fn getter_key(name: &str) -> String {
    format!("{}{}", GETTER_PREFIX, name)
}

pub(super) fn setter_key(name: &str) -> String {
    format!("{}{}", SETTER_PREFIX, name)
}

pub(super) const ACCESSOR_GETTER: &str = GETTER_PREFIX;
pub(super) const ACCESSOR_SETTER: &str = SETTER_PREFIX;

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
        Value::Undefined => wk::UNDEFINED.to_string(),
        Value::Null => wk::NULL.to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Integer(n) => n.to_string(),
        Value::Float(n) => {
            if *n == (*n as i64) as f64 {
                (*n as i64).to_string()
            } else {
                n.to_string()
            }
        }
        Value::String(s) => s.to_string(),
        Value::Cons(c) => c.flatten(),
        Value::BigInt(n) => format!("{}n", n),
        Value::Symbol(id) => format!("Symbol({})", id),
        Value::Function(idx) => {
            if let crate::vm::interpreter::HeapValue::Function(f) = &interp.heap[*idx] {
                let name = f.name.as_deref().unwrap_or("");
                if f.prototype.is_some() && f.super_class.is_some() {
                    format!("[class {}]", name)
                } else if !name.is_empty() {
                    format!("[Function: {}]", name)
                } else {
                    "[Function]".to_string()
                }
            } else {
                "[Function]".to_string()
            }
        }
        Value::NativeFunction(_idx) => "[NativeFunction]".to_string(),
        Value::Array(arr_idx) => {
            if let crate::vm::interpreter::HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                let parts: Vec<String> = arr
                    .elements
                    .iter()
                    .map(|e| to_string_value(interp, e))
                    .collect();
                format!("[{}]", parts.join(","))
            } else {
                "[Array]".to_string()
            }
        }
        Value::Object(obj_idx) => {
            if let crate::vm::interpreter::HeapValue::Object(_obj) = &interp.heap[*obj_idx] {
                let mut all_props: Vec<(String, &Value)> = Vec::new();
                collect_all_properties(
                    interp,
                    *obj_idx,
                    &mut all_props,
                    &mut std::collections::HashSet::new(),
                );
                all_props.sort_by(|a, b| a.0.cmp(&b.0));
                let parts: Vec<String> = all_props
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, format_property_value(interp, v)))
                    .collect();
                format!("{{{}}}", parts.join(", "))
            } else {
                "[Object]".to_string()
            }
        }
        Value::Map(idx) => {
            if let crate::vm::interpreter::HeapValue::Map(map) = &interp.heap[*idx] {
                let entries: Vec<String> = map
                    .keys
                    .iter()
                    .zip(map.values.iter())
                    .map(|(k, v)| {
                        format!(
                            "{} => {}",
                            to_string_value(interp, k),
                            to_string_value(interp, v)
                        )
                    })
                    .collect();
                format!("Map({}) {{{}}}", map.keys.len(), entries.join(", "))
            } else {
                "[Map]".to_string()
            }
        }
        Value::Set(idx) => {
            if let crate::vm::interpreter::HeapValue::Set(set) = &interp.heap[*idx] {
                let entries: Vec<String> = set
                    .values
                    .iter()
                    .map(|v| to_string_value(interp, v))
                    .collect();
                format!("Set({}) {{{}}}", set.values.len(), entries.join(", "))
            } else {
                "[Set]".to_string()
            }
        }
        Value::Date(idx) => {
            if let crate::vm::interpreter::HeapValue::Date(d) = &interp.heap[*idx] {
                format!("Date({})", d.to_utc_string())
            } else {
                "[Date]".to_string()
            }
        }
        Value::RegExp(idx) => {
            if let crate::vm::interpreter::HeapValue::RegExp(r) = &interp.heap[*idx] {
                format!("/{}/{}", r.source, r.flags)
            } else {
                "[RegExp]".to_string()
            }
        }
        Value::Proxy(_) => "[Proxy]".to_string(),
        Value::Buffer(_) => "[Buffer]".to_string(),
        Value::Promise(_) => "[Promise]".to_string(),
        Value::Generator(_) => "[Generator]".to_string(),
        Value::TypedArray(_) => "[TypedArray]".to_string(),
        Value::WeakMap(_) => "[WeakMap]".to_string(),
        Value::WeakSet(_) => "[WeakSet]".to_string(),
        Value::NativeObject(_) => "[NativeObject]".to_string(),
    }
}

fn format_property_value(interp: &Interpreter, v: &Value) -> String {
    match v {
        Value::Function(idx) => {
            if let crate::vm::interpreter::HeapValue::Function(f) = &interp.heap[*idx] {
                let name = f.name.as_deref().unwrap_or("");
                if f.prototype.is_some() && f.super_class.is_some() {
                    format!("[class {}]", name)
                } else if !name.is_empty() {
                    format!("[Function: {}]", name)
                } else {
                    "[Function]".to_string()
                }
            } else {
                "[Function]".to_string()
            }
        }
        Value::NativeFunction(_) => "[NativeFunction]".to_string(),
        Value::Object(idx) => {
            if let crate::vm::interpreter::HeapValue::Object(obj) = &interp.heap[*idx] {
                let mut parts: Vec<String> = Vec::new();
                for (k, val) in &obj.properties {
                    let formatted = format_property_value(interp, val);
                    parts.push(format!("{}: {}", k, formatted));
                }
                parts.sort();
                format!("{{{}}}", parts.join(", "))
            } else {
                "[Object]".to_string()
            }
        }
        Value::Array(arr_idx) => {
            if let crate::vm::interpreter::HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                let parts: Vec<String> = arr
                    .elements
                    .iter()
                    .map(|e| format_property_value(interp, e))
                    .collect();
                format!("[{}]", parts.join(","))
            } else {
                "[Array]".to_string()
            }
        }
        Value::String(s) => format!("\"{}\"", s),
        Value::Map(idx) => {
            if let crate::vm::interpreter::HeapValue::Map(map) = &interp.heap[*idx] {
                let entries: Vec<String> = map
                    .keys
                    .iter()
                    .zip(map.values.iter())
                    .map(|(k, v)| {
                        format!(
                            "{} => {}",
                            to_string_value(interp, k),
                            to_string_value(interp, v)
                        )
                    })
                    .collect();
                format!("Map({}) {{{}}}", map.keys.len(), entries.join(", "))
            } else {
                "[Map]".to_string()
            }
        }
        Value::Set(idx) => {
            if let crate::vm::interpreter::HeapValue::Set(set) = &interp.heap[*idx] {
                let entries: Vec<String> = set
                    .values
                    .iter()
                    .map(|v| to_string_value(interp, v))
                    .collect();
                format!("Set({}) {{{}}}", set.values.len(), entries.join(", "))
            } else {
                "[Set]".to_string()
            }
        }
        _ => to_string_value(interp, v),
    }
}

pub(super) fn to_display_string(interp: &Interpreter, v: &Value) -> String {
    match v {
        Value::String(s) => s.to_string(),
        other => to_string_value(interp, other),
    }
}

pub(super) fn to_json_value(interp: &Interpreter, v: &Value) -> String {
    to_json_value_inner(interp, v, &mut std::collections::HashSet::new(), 0)
}

fn to_json_value_inner(
    interp: &Interpreter,
    v: &Value,
    visited: &mut std::collections::HashSet<usize>,
    depth: usize,
) -> String {
    if depth > 64 {
        return wk::NULL.to_string();
    }
    match v {
        Value::Null => wk::NULL.to_string(),
        Value::Undefined => wk::UNDEFINED.to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Integer(n) => n.to_string(),
        Value::Float(n) => {
            if n.is_nan() {
                wk::NULL.to_string()
            } else if *n == (*n as i64) as f64 {
                (*n as i64).to_string()
            } else {
                n.to_string()
            }
        }
        Value::String(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
        Value::Array(arr_idx) => {
            if let crate::vm::interpreter::HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                if !visited.insert(*arr_idx) {
                    return wk::NULL.to_string();
                }
                let parts: Vec<String> = arr
                    .elements
                    .iter()
                    .map(|e| to_json_value_inner(interp, e, visited, depth + 1))
                    .collect();
                format!("[{}]", parts.join(","))
            } else {
                "[]".to_string()
            }
        }
        Value::Object(obj_idx) => {
            if let crate::vm::interpreter::HeapValue::Object(obj) = &interp.heap[*obj_idx] {
                if !visited.insert(*obj_idx) {
                    return wk::NULL.to_string();
                }
                let parts: Vec<String> = obj
                    .properties
                    .iter()
                    .filter(|(k, _)| is_user_visible_key(k))
                    .map(|(k, v)| {
                        format!(
                            "\"{}\":{}",
                            k,
                            to_json_value_inner(interp, v, visited, depth + 1)
                        )
                    })
                    .collect();
                format!("{{{}}}", parts.join(","))
            } else {
                "{}".to_string()
            }
        }
        Value::Proxy(_) => wk::NULL.to_string(),
        _ => wk::NULL.to_string(),
    }
}

pub(super) fn from_json_value(interp: &mut Interpreter, val: serde_json::Value) -> Value {
    match val {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Boolean(b),
        serde_json::Value::Number(n) => {
            // Preserve integer precision: if the number fits in an i64 and
            // doesn't have a fractional part, store it as Value::Integer.
            // Otherwise fall back to f64. This matches JavaScript's number
            // model for safe integers (Number.MAX_SAFE_INTEGER = 2^53 - 1).
            if let Some(i) = n.as_i64() {
                Value::Integer(i)
            } else {
                Value::Float(n.as_f64().unwrap_or(f64::NAN))
            }
        }
        serde_json::Value::String(s) => Value::from_string(s.into()),
        serde_json::Value::Array(arr) => {
            let len = arr.len();
            let elems: Vec<Value> = Vec::with_capacity(len);
            let mut elems = elems;
            elems.extend(arr.into_iter().map(|v| from_json_value(interp, v)));
            let heap_idx = interp.heap.len();
            interp.heap.push(crate::vm::interpreter::HeapValue::Array(
                crate::vm::interpreter::JsArray { elements: elems },
            ));
            Value::Array(heap_idx)
        }
        serde_json::Value::Object(map) => {
            let len = map.len();
            let mut props = FxHashMap::with_capacity_and_hasher(len, Default::default());
            for (k, v) in map {
                props.insert(k, from_json_value(interp, v));
            }
            let heap_idx = interp.heap.len();
            interp.heap.push(crate::vm::interpreter::HeapValue::Object(
                crate::vm::interpreter::JsObject {
                    properties: props.into(),
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
        Value::String(s) => Some(s.to_string()),
        Value::Cons(c) => Some(c.flatten()),
        _ => None,
    }
}

pub(super) fn find_error_ctor_proto(interp: &Interpreter) -> Option<usize> {
    for hv in &interp.heap {
        if let crate::vm::interpreter::HeapValue::Object(obj) = hv {
            if obj.properties.contains_key(wk::PROTOTYPE) && !obj.properties.contains_key(wk::NAME)
            {
                if let Some(Value::Object(proto_idx)) = obj.properties.get(wk::PROTOTYPE) {
                    return Some(*proto_idx);
                }
            }
        }
    }
    None
}

pub(crate) fn find_error_proto(interp: &Interpreter, type_name: &str) -> Option<usize> {
    for (i, hv) in interp.heap.iter().enumerate() {
        if let crate::vm::interpreter::HeapValue::Object(obj) = hv {
            if let Some(Value::String(name)) = obj.properties.get(wk::NAME) {
                if **name == *type_name {
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

fn collect_all_properties<'a>(
    interp: &'a Interpreter,
    obj_idx: usize,
    out: &mut Vec<(String, &'a Value)>,
    visited: &mut std::collections::HashSet<usize>,
) {
    if !visited.insert(obj_idx) {
        return;
    }
    if let crate::vm::interpreter::HeapValue::Object(obj) = &interp.heap[obj_idx] {
        for (k, v) in &obj.properties {
            if k == wk::CONSTRUCTOR {
                continue;
            }
            out.push((k.to_string(), v));
        }
        if let Some(proto_idx) = obj.prototype {
            collect_all_properties(interp, proto_idx, out, visited);
        }
    }
}
