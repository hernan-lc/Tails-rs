use super::helpers::is_user_visible_key;
use crate::errors::{Error, Result};
use crate::objects::Value;
use crate::vm::interpreter::{Interpreter, PropertyStorage};

use super::reflect_fns::native_reflect_get_own_property_descriptor;

pub(super) fn native_object_keys(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let obj_val = args.first().cloned().unwrap_or(Value::Undefined);
    let keys = match &obj_val {
        Value::Object(obj_idx) => {
            if let crate::vm::interpreter::HeapValue::Object(obj) = &interp.heap[*obj_idx] {
                let mut keys = Vec::with_capacity(obj.properties.len());
                for k in obj.properties.keys() {
                    if !is_user_visible_key(k) {
                        continue;
                    }
                    keys.push(Value::String(k.to_string()));
                }
                keys
            } else {
                Vec::new()
            }
        }
        Value::Array(arr_idx) => {
            if let crate::vm::interpreter::HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                let mut keys = Vec::with_capacity(arr.elements.len());
                for i in 0..arr.elements.len() {
                    keys.push(Value::String(i.to_string()));
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

pub(super) fn native_object_values(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let obj_val = args.first().cloned().unwrap_or(Value::Undefined);
    let vals = match &obj_val {
        Value::Object(obj_idx) => {
            if let crate::vm::interpreter::HeapValue::Object(obj) = &interp.heap[*obj_idx] {
                let mut vals = Vec::with_capacity(obj.properties.len());
                for v in obj.properties.values() {
                    vals.push(v.clone());
                }
                vals
            } else {
                Vec::new()
            }
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
            if let crate::vm::interpreter::HeapValue::Object(obj) = &interp.heap[*obj_idx] {
                let mut pairs = Vec::with_capacity(obj.properties.len());
                for (k, v) in obj.properties.iter() {
                    pairs.push((k.to_string(), v.clone()));
                }
                pairs
            } else {
                Vec::new()
            }
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
                elements: vec![Value::String(k), v],
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

pub(super) fn native_object_define_property(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let target = args.first().cloned().unwrap_or(Value::Undefined);
    let property = match args.get(1) {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Cons(c)) => c.flatten(),
        _ => return Ok(target),
    };
    let descriptor = args.get(2).cloned().unwrap_or(Value::Undefined);

    if let Value::Object(obj_idx) = &descriptor {
        if let crate::vm::interpreter::HeapValue::Object(desc) = &interp.heap[*obj_idx] {
            let getter = desc.properties.get("get").cloned();
            let setter = desc.properties.get("set").cloned();
            let value = desc.properties.get("value").cloned();

            match &target {
                Value::Object(tgt_idx) => {
                    if let crate::vm::interpreter::HeapValue::Object(tgt) =
                        &mut interp.heap[*tgt_idx]
                    {
                        if let Some(getter_fn) = getter {
                            if !matches!(getter_fn, Value::Undefined) {
                                tgt.properties
                                    .insert(format!("__getter_{}", property), getter_fn);
                            }
                        }
                        if let Some(setter_fn) = setter {
                            if !matches!(setter_fn, Value::Undefined) {
                                tgt.properties
                                    .insert(format!("__setter_{}", property), setter_fn);
                            }
                        }
                        if let Some(val) = value {
                            tgt.properties.insert(property, val);
                        }
                    }
                }
                Value::Function(func_idx) => {
                    if let crate::vm::interpreter::HeapValue::Function(f) =
                        &mut interp.heap[*func_idx]
                    {
                        if let Some(getter_fn) = getter {
                            if !matches!(getter_fn, Value::Undefined) {
                                f.properties
                                    .insert(format!("__getter_{}", property), getter_fn);
                            }
                        }
                        if let Some(setter_fn) = setter {
                            if !matches!(setter_fn, Value::Undefined) {
                                f.properties
                                    .insert(format!("__setter_{}", property), setter_fn);
                            }
                        }
                        if let Some(val) = value {
                            f.properties.insert(property, val);
                        }
                    }
                }
                _ => {}
            }
        }
    } else if let Some(val) = match &descriptor {
        Value::Object(obj_idx) => {
            if let crate::vm::interpreter::HeapValue::Object(obj) = &interp.heap[*obj_idx] {
                obj.properties.get("value").cloned()
            } else {
                None
            }
        }
        _ => None,
    } {
        match &target {
            Value::Object(obj_idx) => {
                if let crate::vm::interpreter::HeapValue::Object(obj) =
                    &mut interp.heap[*obj_idx]
                {
                    obj.properties.insert(property, val);
                }
            }
            Value::Function(func_idx) => {
                if let crate::vm::interpreter::HeapValue::Function(f) =
                    &mut interp.heap[*func_idx]
                {
                    f.properties.insert(property, val);
                }
            }
            _ => {}
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
        (Value::Cons(a), Value::String(b)) => a.flatten() == *b,
        (Value::String(a), Value::Cons(b)) => *a == b.flatten(),
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
        Some(Value::String(s)) => s.clone(),
        Some(Value::Cons(c)) => c.flatten(),
        Some(v) => format!("{:?}", v),
        None => return Ok(Value::Boolean(false)),
    };
    match this {
        Value::Object(obj_idx) => {
            if let crate::vm::interpreter::HeapValue::Object(obj) = &interp.heap[*obj_idx] {
                Ok(Value::Boolean(obj.properties.contains_key(&prop)))
            } else {
                Ok(Value::Boolean(false))
            }
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
