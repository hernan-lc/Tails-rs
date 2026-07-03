use crate::errors::Result;
use crate::objects::Value;
use crate::vm::interpreter::Interpreter;

use super::reflect_fns::native_reflect_get_own_property_descriptor;

pub(super) fn native_object_keys(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let obj_val = args.first().cloned().unwrap_or(Value::Undefined);
    let mut keys = Vec::new();
    match &obj_val {
        Value::Object(obj_idx) => {
            if let crate::vm::interpreter::HeapValue::Object(obj) = &interp.heap[*obj_idx] {
                for k in obj.properties.keys() {
                    // Skip internal getter/setter/method keys
                    if k.starts_with("__getter_")
                        || k.starts_with("__setter_")
                        || k.starts_with("__method_")
                    {
                        continue;
                    }
                    keys.push(Value::String(k.clone()));
                }
            }
        }
        Value::Array(arr_idx) => {
            if let crate::vm::interpreter::HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                for i in 0..arr.elements.len() {
                    keys.push(Value::String(i.to_string()));
                }
            }
        }
        _ => {}
    }
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
    let mut vals = Vec::new();
    match &obj_val {
        Value::Object(obj_idx) => {
            if let crate::vm::interpreter::HeapValue::Object(obj) = &interp.heap[*obj_idx] {
                for v in obj.properties.values() {
                    vals.push(v.clone());
                }
            }
        }
        Value::Array(arr_idx) => {
            if let crate::vm::interpreter::HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                vals.extend(arr.elements.iter().cloned());
            }
        }
        _ => {}
    }
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
                obj.properties
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect()
            } else {
                Vec::new()
            }
        }
        Value::Array(arr_idx) => {
            if let crate::vm::interpreter::HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                arr.elements
                    .iter()
                    .enumerate()
                    .map(|(i, v)| (i.to_string(), v.clone()))
                    .collect()
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
                        src_obj
                            .properties
                            .iter()
                            .map(|(k, v)| (k.clone(), v.clone()))
                            .collect()
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

    let value = match &descriptor {
        Value::Object(obj_idx) => {
            if let crate::vm::interpreter::HeapValue::Object(obj) = &interp.heap[*obj_idx] {
                obj.properties.get("value").cloned()
            } else {
                None
            }
        }
        _ => None,
    };

    if let Some(val) = value {
        match &target {
            Value::Object(obj_idx) => {
                if let crate::vm::interpreter::HeapValue::Object(obj) = &mut interp.heap[*obj_idx] {
                    obj.properties.insert(property, val);
                }
            }
            Value::Function(func_idx) => {
                if let crate::vm::interpreter::HeapValue::Function(f) = &mut interp.heap[*func_idx]
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
