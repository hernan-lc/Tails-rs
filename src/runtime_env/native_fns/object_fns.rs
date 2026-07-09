use super::helpers::{
    collect_own_enumerable_keys, getter_key, is_user_visible_key, setter_key, ACCESSOR_GETTER,
    ACCESSOR_SETTER,
};
use crate::errors::{Error, Result};
use crate::objects::Value;
use crate::vm::interpreter::{Interpreter, PropertyStorage};

use super::reflect_fns::native_reflect_get_own_property_descriptor;

use crate::well_known as wk;

pub(super) fn native_object_keys(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let obj_val = args.first().cloned().unwrap_or(Value::Undefined);
    let keys = match &obj_val {
        Value::Object(obj_idx) => {
            if let crate::vm::interpreter::HeapValue::Object(obj) = &interp.heap[*obj_idx] {
                collect_own_enumerable_keys(&obj.properties)
                    .into_iter()
                    .map(Value::from_string)
                    .collect()
            } else {
                Vec::new()
            }
        }
        Value::Array(arr_idx) => {
            if let crate::vm::interpreter::HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                let mut keys = Vec::with_capacity(arr.elements.len());
                for i in 0..arr.elements.len() {
                    keys.push(Value::from_string(i.to_string()));
                }
                keys
            } else {
                Vec::new()
            }
        }
        _ => Vec::new(),
    };
    let heap_idx = interp.heap.len();
    interp.heap.push(crate::vm::interpreter::HeapValue::Array(
        crate::vm::interpreter::JsArray { elements: keys },
    ));
    Ok(Value::Array(heap_idx))
}

pub(super) fn native_object_from_entries(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let list = args.first().cloned().unwrap_or(Value::Undefined);

    // First gather (key, value) pairs while only borrowing the heap immutably.
    let mut collected: Vec<(String, Value)> = Vec::new();
    if let Value::Array(list_idx) = &list {
        if let crate::vm::interpreter::HeapValue::Array(list_arr) = &interp.heap[*list_idx] {
            for entry in list_arr.elements.iter() {
                let pair_idx = match entry {
                    Value::Array(i) => *i,
                    _ => continue,
                };
                if let crate::vm::interpreter::HeapValue::Array(pair) = &interp.heap[pair_idx] {
                    if pair.elements.len() < 2 {
                        continue;
                    }
                    let key = match &pair.elements[0] {
                        Value::String(s) => s.to_string(),
                        Value::Cons(c) => c.flatten(),
                        Value::Integer(i) => i.to_string(),
                        Value::Float(f) => f.to_string(),
                        Value::Boolean(b) => b.to_string(),
                        other => format!("{:?}", other),
                    };
                    collected.push((key, pair.elements[1].clone()));
                }
            }
        }
    }

    let obj_idx = interp.heap.len();
    interp.heap.push(crate::vm::interpreter::HeapValue::Object(
        crate::vm::interpreter::JsObject {
            properties: PropertyStorage::new(),
            prototype: None,
            extensible: true,
        },
    ));
    if let crate::vm::interpreter::HeapValue::Object(obj) = &mut interp.heap[obj_idx] {
        for (key, value) in collected {
            obj.properties.insert(key, value);
        }
    }

    Ok(Value::Object(obj_idx))
}

pub(super) fn native_object_values(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let obj_val = args.first().cloned().unwrap_or(Value::Undefined);
    let vals = match &obj_val {
        Value::Object(obj_idx) => {
            let keys =
                if let crate::vm::interpreter::HeapValue::Object(obj) = &interp.heap[*obj_idx] {
                    collect_own_enumerable_keys(&obj.properties)
                } else {
                    Vec::new()
                };
            let mut vals = Vec::with_capacity(keys.len());
            for k in keys {
                let v = interp
                    .get_property_str(&obj_val, &k)
                    .unwrap_or(Value::Undefined);
                vals.push(v);
            }
            vals
        }
        Value::Array(arr_idx) => {
            if let crate::vm::interpreter::HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                let mut vals = Vec::with_capacity(arr.elements.len());
                vals.extend(arr.elements.iter().cloned());
                vals
            } else {
                Vec::new()
            }
        }
        _ => Vec::new(),
    };
    let heap_idx = interp.heap.len();
    interp.heap.push(crate::vm::interpreter::HeapValue::Array(
        crate::vm::interpreter::JsArray { elements: vals },
    ));
    Ok(Value::Array(heap_idx))
}

pub(super) fn native_object_entries(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let obj_val = args.first().cloned().unwrap_or(Value::Undefined);
    let pairs: Vec<(String, Value)> = match &obj_val {
        Value::Object(obj_idx) => {
            let keys =
                if let crate::vm::interpreter::HeapValue::Object(obj) = &interp.heap[*obj_idx] {
                    collect_own_enumerable_keys(&obj.properties)
                } else {
                    Vec::new()
                };
            let mut pairs = Vec::with_capacity(keys.len());
            for k in keys {
                let v = interp
                    .get_property_str(&obj_val, &k)
                    .unwrap_or(Value::Undefined);
                pairs.push((k, v));
            }
            pairs
        }
        Value::Array(arr_idx) => {
            if let crate::vm::interpreter::HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                let mut pairs = Vec::with_capacity(arr.elements.len());
                for (i, v) in arr.elements.iter().enumerate() {
                    pairs.push((i.to_string(), v.clone()));
                }
                pairs
            } else {
                Vec::new()
            }
        }
        _ => Vec::new(),
    };
    let mut entries = Vec::with_capacity(pairs.len());
    for (k, v) in pairs {
        let heap_idx = interp.heap.len();
        interp.heap.push(crate::vm::interpreter::HeapValue::Array(
            crate::vm::interpreter::JsArray {
                elements: vec![Value::from_string(k.into()), v],
            },
        ));
        entries.push(Value::Array(heap_idx));
    }
    let heap_idx = interp.heap.len();
    interp.heap.push(crate::vm::interpreter::HeapValue::Array(
        crate::vm::interpreter::JsArray { elements: entries },
    ));
    Ok(Value::Array(heap_idx))
}

pub(super) fn native_object_assign(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    if args.is_empty() {
        return Ok(Value::Undefined);
    }
    let target = args[0].clone();
    if let Value::Object(target_idx) = &target {
        for src in &args[1..] {
            if let Value::Object(src_idx) = src {
                let cloned: Vec<(String, Value)> =
                    if let crate::vm::interpreter::HeapValue::Object(src_obj) =
                        &interp.heap[*src_idx]
                    {
                        let mut cloned = Vec::with_capacity(src_obj.properties.len());
                        for (k, v) in src_obj.properties.iter() {
                            cloned.push((k.to_string(), v.clone()));
                        }
                        cloned
                    } else {
                        Vec::new()
                    };
                if let crate::vm::interpreter::HeapValue::Object(tgt_obj) =
                    &mut interp.heap[*target_idx]
                {
                    for (k, v) in cloned {
                        tgt_obj.properties.insert(k, v);
                    }
                }
            }
        }
    }
    Ok(target)
}

fn define_property_on(
    interp: &mut Interpreter,
    target: &Value,
    property: &str,
    descriptor: &Value,
) -> Result<()> {
    let Value::Object(desc_idx) = descriptor else {
        return Ok(());
    };
    let (getter, setter, value, has_get, has_set, has_value) =
        if let crate::vm::interpreter::HeapValue::Object(desc) = &interp.heap[*desc_idx] {
            let has_get = desc.properties.contains_key("get");
            let has_set = desc.properties.contains_key("set");
            let has_value = desc.properties.contains_key("value");
            (
                desc.properties.get("get").cloned(),
                desc.properties.get("set").cloned(),
                desc.properties.get("value").cloned(),
                has_get,
                has_set,
                has_value,
            )
        } else {
            return Ok(());
        };

    let is_accessor = has_get || has_set;
    let g_key = getter_key(property);
    let s_key = setter_key(property);

    match target {
        Value::Object(tgt_idx) => {
            if let crate::vm::interpreter::HeapValue::Object(tgt) = &mut interp.heap[*tgt_idx] {
                if is_accessor {
                    // Accessor descriptor replaces any data property.
                    tgt.properties.remove(property);
                    if let Some(getter_fn) = getter {
                        if !matches!(getter_fn, Value::Undefined) {
                            tgt.properties.insert(g_key, getter_fn);
                        } else {
                            tgt.properties.remove(&g_key);
                        }
                    }
                    if let Some(setter_fn) = setter {
                        if !matches!(setter_fn, Value::Undefined) {
                            tgt.properties.insert(s_key, setter_fn);
                        } else {
                            tgt.properties.remove(&s_key);
                        }
                    }
                } else if has_value {
                    // Data descriptor replaces any existing accessor.
                    tgt.properties.remove(&g_key);
                    tgt.properties.remove(&s_key);
                    if let Some(val) = value {
                        tgt.properties.insert(property.to_string(), val);
                    }
                }
            }
        }
        Value::Function(func_idx) => {
            if let crate::vm::interpreter::HeapValue::Function(f) = &mut interp.heap[*func_idx] {
                if is_accessor {
                    f.properties.remove(property);
                    if let Some(getter_fn) = getter {
                        if !matches!(getter_fn, Value::Undefined) {
                            f.properties.insert(g_key, getter_fn);
                        } else {
                            f.properties.remove(&g_key);
                        }
                    }
                    if let Some(setter_fn) = setter {
                        if !matches!(setter_fn, Value::Undefined) {
                            f.properties.insert(s_key, setter_fn);
                        } else {
                            f.properties.remove(&s_key);
                        }
                    }
                } else if has_value {
                    f.properties.remove(&g_key);
                    f.properties.remove(&s_key);
                    if let Some(val) = value {
                        f.properties.insert(property.to_string(), val);
                    }
                }
            }
        }
        Value::Array(arr_idx) => {
            if let crate::vm::interpreter::HeapValue::Array(arr) = &mut interp.heap[*arr_idx] {
                if property == wk::LENGTH {
                    if let Some(val) = value {
                        let new_len = match &val {
                            Value::Integer(n) => (*n).max(0) as usize,
                            Value::Float(n) => (*n as i64).max(0) as usize,
                            _ => 0,
                        };
                        if new_len < arr.elements.len() {
                            arr.elements.truncate(new_len);
                        } else if new_len > arr.elements.len() {
                            arr.elements.resize(new_len, Value::Undefined);
                        }
                    }
                } else if let Some(index) =
                    crate::vm::interpreter::property_access::parse_array_index(property)
                {
                    if let Some(val) = value {
                        if index >= arr.elements.len() {
                            arr.elements.resize(index + 1, Value::Undefined);
                        }
                        arr.elements[index] = val;
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}

pub(super) fn native_object_define_property(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let target = args.first().cloned().unwrap_or(Value::Undefined);
    let property = match args.get(1) {
        Some(Value::String(s)) => s.to_string(),
        Some(Value::Cons(c)) => c.flatten(),
        Some(Value::Symbol(id)) => format!("__sym_{}", id),
        Some(Value::Integer(n)) => n.to_string(),
        Some(Value::Float(n)) => {
            if *n == (*n as i64) as f64 {
                (*n as i64).to_string()
            } else {
                n.to_string()
            }
        }
        _ => return Ok(target),
    };
    let descriptor = args.get(2).cloned().unwrap_or(Value::Undefined);
    define_property_on(interp, &target, &property, &descriptor)?;
    Ok(target)
}

pub(super) fn native_object_define_properties(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let target = args.first().cloned().unwrap_or(Value::Undefined);
    let descriptors = args.get(1).cloned().unwrap_or(Value::Undefined);
    if let Value::Object(d_idx) = &descriptors {
        if let crate::vm::interpreter::HeapValue::Object(d_obj) = &interp.heap[*d_idx] {
            let items: Vec<(String, Value)> = d_obj
                .properties
                .iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect();
            for (key, desc) in items {
                define_property_on(interp, &target, &key, &desc)?;
            }
        }
    }
    Ok(target)
}

pub(super) fn native_object_get_own_property_descriptor(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    native_reflect_get_own_property_descriptor(interp, _this, args)
}

pub(super) fn native_object_get_own_property_descriptors(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let obj_val = args.first().cloned().unwrap_or(Value::Undefined);

    // Collect (key, value, getter, setter) for every own property.
    // Accessor-only properties (stored as __getter_/__setter_ without a data
    // slot) must still appear — Object.getOwnPropertyDescriptors is how Zod
    // merges schema defs.
    let mut entries: Vec<(String, Value, Option<Value>, Option<Value>)> = Vec::new();
    match &obj_val {
        Value::Object(obj_idx) => {
            if let crate::vm::interpreter::HeapValue::Object(obj) = &interp.heap[*obj_idx] {
                let mut getters: std::collections::HashMap<String, Value> =
                    std::collections::HashMap::new();
                let mut setters: std::collections::HashMap<String, Value> =
                    std::collections::HashMap::new();
                let mut data_keys: Vec<(String, Value)> = Vec::new();
                let mut accessor_names: std::collections::HashSet<String> =
                    std::collections::HashSet::new();

                for (k, v) in obj.properties.iter() {
                    if let Some(real) = k.strip_prefix(ACCESSOR_GETTER) {
                        getters.insert(real.to_string(), v.clone());
                        accessor_names.insert(real.to_string());
                    } else if let Some(real) = k.strip_prefix(ACCESSOR_SETTER) {
                        setters.insert(real.to_string(), v.clone());
                        accessor_names.insert(real.to_string());
                    } else if is_user_visible_key(k) && !k.starts_with("__sym_") {
                        data_keys.push((k.to_string(), v.clone()));
                    }
                }

                // Data properties: if a key still has a getter/setter alongside
                // a data value, prefer the data descriptor (defineProperty with
                // value should have removed accessors; keep data if both exist).
                let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
                for (k, v) in data_keys {
                    seen.insert(k.clone());
                    // If this key is only present as data (accessors already
                    // removed), emit a data descriptor. If somehow both remain,
                    // prefer data so assignProp self-caching works.
                    entries.push((k, v, None, None));
                }
                // Accessor-only properties (no data slot)
                for name in accessor_names {
                    if seen.contains(&name) {
                        continue;
                    }
                    let getter = getters.get(&name).cloned();
                    let setter = setters.get(&name).cloned();
                    entries.push((name, Value::Undefined, getter, setter));
                }
            }
        }
        Value::Array(arr_idx) => {
            if let crate::vm::interpreter::HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                for (i, v) in arr.elements.iter().enumerate() {
                    entries.push((i.to_string(), v.clone(), None, None));
                }
                entries.push((
                    wk::LENGTH.to_string(),
                    Value::Float(arr.elements.len() as f64),
                    None,
                    None,
                ));
            }
        }
        _ => {}
    }

    let mut result_props = PropertyStorage::new();
    for (key, value, getter, setter) in entries {
        let mut desc = PropertyStorage::new();
        if getter.is_some() || setter.is_some() {
            desc.insert("get".into(), getter.unwrap_or(Value::Undefined));
            desc.insert("set".into(), setter.unwrap_or(Value::Undefined));
            desc.insert("enumerable".into(), Value::Boolean(true));
            desc.insert("configurable".into(), Value::Boolean(true));
        } else {
            desc.insert("value".into(), value);
            desc.insert("writable".into(), Value::Boolean(true));
            desc.insert("enumerable".into(), Value::Boolean(true));
            desc.insert("configurable".into(), Value::Boolean(true));
        }
        let desc_idx = interp.heap.len();
        interp.heap.push(crate::vm::interpreter::HeapValue::Object(
            crate::vm::interpreter::JsObject {
                properties: desc,
                prototype: None,
                extensible: true,
            },
        ));
        result_props.insert(key, Value::Object(desc_idx));
    }

    let result_idx = interp.heap.len();
    interp.heap.push(crate::vm::interpreter::HeapValue::Object(
        crate::vm::interpreter::JsObject {
            properties: result_props,
            prototype: None,
            extensible: true,
        },
    ));
    Ok(Value::Object(result_idx))
}

pub(super) fn native_object_freeze(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let target = args.first().cloned().unwrap_or(Value::Undefined);
    if let Value::Object(obj_idx) = &target {
        if let crate::vm::interpreter::HeapValue::Object(obj) = &mut interp.heap[*obj_idx] {
            obj.extensible = false;
        }
    }
    Ok(target)
}

pub(super) fn native_object_is(
    _interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let val1 = args.first().cloned().unwrap_or(Value::Undefined);
    let val2 = args.get(1).cloned().unwrap_or(Value::Undefined);
    Ok(Value::Boolean(value_strict_equal(&val1, &val2)))
}

fn value_strict_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Undefined, Value::Undefined) => true,
        (Value::Null, Value::Null) => true,
        (Value::Boolean(a), Value::Boolean(b)) => a == b,
        (Value::String(a), Value::String(b)) => a == b,
        (Value::Cons(a), Value::String(b)) => a.flatten() == **b,
        (Value::String(a), Value::Cons(b)) => *a == b.flatten().into(),
        (Value::Cons(a), Value::Cons(b)) => a.flatten() == b.flatten(),
        (Value::Integer(a), Value::Integer(b)) => a == b,
        (Value::Float(a), Value::Float(b)) => {
            if a.is_nan() && b.is_nan() {
                true
            } else {
                a == b
            }
        }
        (Value::Integer(a), Value::Float(b)) => {
            let a_f = *a as f64;
            if a_f.is_nan() && b.is_nan() {
                true
            } else {
                a_f == *b
            }
        }
        (Value::Float(a), Value::Integer(b)) => {
            let b_f = *b as f64;
            if a.is_nan() && b_f.is_nan() {
                true
            } else {
                *a == b_f
            }
        }
        (Value::BigInt(a), Value::BigInt(b)) => a == b,
        _ => {
            // For heap types, compare by index
            std::mem::discriminant(a) == std::mem::discriminant(b)
                && match (a, b) {
                    (Value::Object(a), Value::Object(b)) => a == b,
                    (Value::Array(a), Value::Array(b)) => a == b,
                    (Value::Function(a), Value::Function(b)) => a == b,
                    _ => false,
                }
        }
    }
}

pub(super) fn native_object_prevent_extensions(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let target = args.first().cloned().unwrap_or(Value::Undefined);
    if let Value::Object(obj_idx) = &target {
        if let crate::vm::interpreter::HeapValue::Object(obj) = &mut interp.heap[*obj_idx] {
            obj.extensible = false;
        }
    }
    Ok(target)
}

pub(super) fn native_object_is_extensible(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let target = args.first().cloned().unwrap_or(Value::Undefined);
    match &target {
        Value::Object(obj_idx) => {
            if let crate::vm::interpreter::HeapValue::Object(obj) = &interp.heap[*obj_idx] {
                Ok(Value::Boolean(obj.extensible))
            } else {
                Ok(Value::Boolean(false))
            }
        }
        _ => Ok(Value::Boolean(false)),
    }
}

pub(super) fn native_object_is_sealed(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let target = args.first().cloned().unwrap_or(Value::Undefined);
    match &target {
        Value::Object(obj_idx) => {
            if let crate::vm::interpreter::HeapValue::Object(obj) = &interp.heap[*obj_idx] {
                // An object is sealed if it's not extensible (simplified: no property descriptor tracking yet)
                Ok(Value::Boolean(!obj.extensible))
            } else {
                Ok(Value::Boolean(false))
            }
        }
        _ => Ok(Value::Boolean(false)),
    }
}

pub(super) fn native_object_is_frozen(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let target = args.first().cloned().unwrap_or(Value::Undefined);
    match &target {
        Value::Object(obj_idx) => {
            if let crate::vm::interpreter::HeapValue::Object(obj) = &interp.heap[*obj_idx] {
                // An object is frozen if it's not extensible (simplified: no property descriptor tracking yet)
                Ok(Value::Boolean(!obj.extensible))
            } else {
                Ok(Value::Boolean(false))
            }
        }
        _ => Ok(Value::Boolean(false)),
    }
}

pub(super) fn native_object_seal(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let target = args.first().cloned().unwrap_or(Value::Undefined);
    if let Value::Object(obj_idx) = &target {
        if let crate::vm::interpreter::HeapValue::Object(obj) = &mut interp.heap[*obj_idx] {
            obj.extensible = false;
        }
    }
    Ok(target)
}

pub(super) fn native_object_has_own_property(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let prop = match args.first() {
        Some(Value::String(s)) => s.to_string(),
        Some(Value::Cons(c)) => c.flatten(),
        Some(v) => format!("{:?}", v),
        None => return Ok(Value::Boolean(false)),
    };
    match this {
        Value::Object(obj_idx) => {
            if let crate::vm::interpreter::HeapValue::Object(obj) = &interp.heap[*obj_idx] {
                let has = obj.properties.contains_key(&prop)
                    || obj.properties.contains_key(&getter_key(&prop))
                    || obj.properties.contains_key(&setter_key(&prop));
                Ok(Value::Boolean(has))
            } else {
                Ok(Value::Boolean(false))
            }
        }
        Value::Array(arr_idx) => {
            if let crate::vm::interpreter::HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                if prop == wk::LENGTH {
                    return Ok(Value::Boolean(true));
                }
                if let Some(index) =
                    crate::vm::interpreter::property_access::parse_array_index(&prop)
                {
                    return Ok(Value::Boolean(index < arr.elements.len()));
                }
            }
            Ok(Value::Boolean(false))
        }
        _ => Ok(Value::Boolean(false)),
    }
}

pub(super) fn native_object_create(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let proto = args.first().cloned().unwrap_or(Value::Null);
    let properties = args.get(1).cloned();

    // Object.create(proto, [propertiesObject])
    // proto must be an object or null
    let proto_idx = match &proto {
        Value::Null => None,
        Value::Object(idx) => Some(*idx),
        _ => {
            return Err(Error::TypeError(
                "Object prototype may only be an Object or null".into(),
            ))
        }
    };

    // Create new object with the specified prototype
    let new_obj_idx = interp.gc.allocate(
        &mut interp.heap,
        crate::vm::interpreter::HeapValue::Object(crate::vm::interpreter::JsObject {
            properties: PropertyStorage::new(),
            prototype: proto_idx,
            extensible: true,
        }),
    );

    // If properties object is provided, define properties
    if let Some(Value::Object(props_idx)) = properties {
        // Collect property definitions first to avoid borrow issues
        let mut prop_defs = Vec::new();
        if let crate::vm::interpreter::HeapValue::Object(props_obj) = &interp.heap[props_idx] {
            for (key, desc) in &props_obj.properties {
                if let Value::Object(desc_idx) = desc {
                    if let crate::vm::interpreter::HeapValue::Object(desc_obj) =
                        &interp.heap[*desc_idx]
                    {
                        if let Some(value) = desc_obj.properties.get("value") {
                            prop_defs.push((key.to_string(), value.clone()));
                        }
                    }
                }
            }
        }
        // Now apply the collected properties
        for (key, value) in prop_defs {
            if let crate::vm::interpreter::HeapValue::Object(new_obj) =
                &mut interp.heap[new_obj_idx]
            {
                new_obj.properties.insert(key, value);
            }
        }
    }

    Ok(Value::Object(new_obj_idx))
}
