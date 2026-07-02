use crate::errors::{Error, Result};
use crate::objects::js_collections::{JsMap, JsSet, JsWeakMap, JsWeakSet};
use crate::objects::Value;
use crate::vm::interpreter::Interpreter;

// Map functions
pub(super) fn native_map_constructor(
    interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let map = JsMap::new();
    let heap_idx = interp.heap.len();
    interp
        .heap
        .push(crate::vm::interpreter::HeapValue::Map(map));
    Ok(Value::Map(heap_idx))
}

pub(super) fn native_map_get(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let idx = match this {
        Value::Map(idx) => *idx,
        _ => return Err(Error::TypeError("Not a Map".into())),
    };

    let key = args.first().cloned().unwrap_or(Value::Undefined);

    if let crate::vm::interpreter::HeapValue::Map(map) = &interp.heap[idx] {
        Ok(map.get(&key).cloned().unwrap_or(Value::Undefined))
    } else {
        Err(Error::TypeError("Not a Map".into()))
    }
}

pub(super) fn native_map_set(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let idx = match this {
        Value::Map(idx) => *idx,
        _ => return Err(Error::TypeError("Not a Map".into())),
    };

    let key = args.first().cloned().unwrap_or(Value::Undefined);
    let value = args.get(1).cloned().unwrap_or(Value::Undefined);

    if let crate::vm::interpreter::HeapValue::Map(map) = &mut interp.heap[idx] {
        map.set(key, value);
        Ok(this.clone())
    } else {
        Err(Error::TypeError("Not a Map".into()))
    }
}

pub(super) fn native_map_has(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let idx = match this {
        Value::Map(idx) => *idx,
        _ => return Err(Error::TypeError("Not a Map".into())),
    };

    let key = args.first().cloned().unwrap_or(Value::Undefined);

    if let crate::vm::interpreter::HeapValue::Map(map) = &interp.heap[idx] {
        Ok(Value::Boolean(map.has(&key)))
    } else {
        Err(Error::TypeError("Not a Map".into()))
    }
}

pub(super) fn native_map_delete(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let idx = match this {
        Value::Map(idx) => *idx,
        _ => return Err(Error::TypeError("Not a Map".into())),
    };

    let key = args.first().cloned().unwrap_or(Value::Undefined);

    if let crate::vm::interpreter::HeapValue::Map(map) = &mut interp.heap[idx] {
        Ok(Value::Boolean(map.delete(&key)))
    } else {
        Err(Error::TypeError("Not a Map".into()))
    }
}

pub(super) fn native_map_clear(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let idx = match this {
        Value::Map(idx) => *idx,
        _ => return Err(Error::TypeError("Not a Map".into())),
    };

    if let crate::vm::interpreter::HeapValue::Map(map) = &mut interp.heap[idx] {
        map.clear();
    }
    Ok(Value::Undefined)
}

pub(super) fn native_map_size(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let idx = match this {
        Value::Map(idx) => *idx,
        _ => return Err(Error::TypeError("Not a Map".into())),
    };

    if let crate::vm::interpreter::HeapValue::Map(map) = &interp.heap[idx] {
        Ok(Value::Float(map.size() as f64))
    } else {
        Err(Error::TypeError("Not a Map".into()))
    }
}

pub(super) fn native_map_for_each(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let idx = match this {
        Value::Map(idx) => *idx,
        _ => return Err(Error::TypeError("Not a Map".into())),
    };

    let callback = args.first().cloned().unwrap_or(Value::Undefined);
    let this_arg = args.get(1).cloned().unwrap_or(Value::Undefined);

    // Clone entries to avoid borrow issues
    let entries: Vec<(Value, Value)> =
        if let crate::vm::interpreter::HeapValue::Map(map) = &interp.heap[idx] {
            map.keys
                .iter()
                .zip(map.values.iter())
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        } else {
            return Err(Error::TypeError("Not a Map".into()));
        };

    for (k, v) in entries {
        interp.call_value(&callback, &this_arg, &[v.clone(), k.clone(), this.clone()])?;
    }
    Ok(Value::Undefined)
}

pub(super) fn native_map_keys(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let idx = match this {
        Value::Map(idx) => *idx,
        _ => return Err(Error::TypeError("Not a Map".into())),
    };

    if let crate::vm::interpreter::HeapValue::Map(map) = &interp.heap[idx] {
        let keys = map.keys();
        let heap_idx = interp.heap.len();
        interp.heap.push(crate::vm::interpreter::HeapValue::Array(
            crate::vm::interpreter::JsArray { elements: keys },
        ));
        Ok(Value::Array(heap_idx))
    } else {
        Err(Error::TypeError("Not a Map".into()))
    }
}

pub(super) fn native_map_values(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let idx = match this {
        Value::Map(idx) => *idx,
        _ => return Err(Error::TypeError("Not a Map".into())),
    };

    if let crate::vm::interpreter::HeapValue::Map(map) = &interp.heap[idx] {
        let values = map.values();
        let heap_idx = interp.heap.len();
        interp.heap.push(crate::vm::interpreter::HeapValue::Array(
            crate::vm::interpreter::JsArray { elements: values },
        ));
        Ok(Value::Array(heap_idx))
    } else {
        Err(Error::TypeError("Not a Map".into()))
    }
}

pub(super) fn native_map_entries(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let idx = match this {
        Value::Map(idx) => *idx,
        _ => return Err(Error::TypeError("Not a Map".into())),
    };

    if let crate::vm::interpreter::HeapValue::Map(map) = &interp.heap[idx] {
        let entries = map.entries();
        let arr_elements: Vec<Value> = entries
            .into_iter()
            .map(|(k, v)| {
                let heap_idx = interp.heap.len();
                interp.heap.push(crate::vm::interpreter::HeapValue::Array(
                    crate::vm::interpreter::JsArray {
                        elements: vec![k, v],
                    },
                ));
                Value::Array(heap_idx)
            })
            .collect();
        let heap_idx = interp.heap.len();
        interp.heap.push(crate::vm::interpreter::HeapValue::Array(
            crate::vm::interpreter::JsArray {
                elements: arr_elements,
            },
        ));
        Ok(Value::Array(heap_idx))
    } else {
        Err(Error::TypeError("Not a Map".into()))
    }
}

// Set functions
pub(super) fn native_set_constructor(
    interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let set = JsSet::new();
    let heap_idx = interp.heap.len();
    interp
        .heap
        .push(crate::vm::interpreter::HeapValue::Set(set));
    Ok(Value::Set(heap_idx))
}

pub(super) fn native_set_add(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let idx = match this {
        Value::Set(idx) => *idx,
        _ => return Err(Error::TypeError("Not a Set".into())),
    };

    let value = args.first().cloned().unwrap_or(Value::Undefined);

    if let crate::vm::interpreter::HeapValue::Set(set) = &mut interp.heap[idx] {
        set.add(value);
        Ok(this.clone())
    } else {
        Err(Error::TypeError("Not a Set".into()))
    }
}

pub(super) fn native_set_has(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let idx = match this {
        Value::Set(idx) => *idx,
        _ => return Err(Error::TypeError("Not a Set".into())),
    };

    let value = args.first().cloned().unwrap_or(Value::Undefined);

    if let crate::vm::interpreter::HeapValue::Set(set) = &interp.heap[idx] {
        Ok(Value::Boolean(set.has(&value)))
    } else {
        Err(Error::TypeError("Not a Set".into()))
    }
}

pub(super) fn native_set_delete(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let idx = match this {
        Value::Set(idx) => *idx,
        _ => return Err(Error::TypeError("Not a Set".into())),
    };

    let value = args.first().cloned().unwrap_or(Value::Undefined);

    if let crate::vm::interpreter::HeapValue::Set(set) = &mut interp.heap[idx] {
        Ok(Value::Boolean(set.delete(&value)))
    } else {
        Err(Error::TypeError("Not a Set".into()))
    }
}

pub(super) fn native_set_clear(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let idx = match this {
        Value::Set(idx) => *idx,
        _ => return Err(Error::TypeError("Not a Set".into())),
    };

    if let crate::vm::interpreter::HeapValue::Set(set) = &mut interp.heap[idx] {
        set.clear();
    }
    Ok(Value::Undefined)
}

pub(super) fn native_set_size(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let idx = match this {
        Value::Set(idx) => *idx,
        _ => return Err(Error::TypeError("Not a Set".into())),
    };

    if let crate::vm::interpreter::HeapValue::Set(set) = &interp.heap[idx] {
        Ok(Value::Float(set.size() as f64))
    } else {
        Err(Error::TypeError("Not a Set".into()))
    }
}

pub(super) fn native_set_for_each(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let idx = match this {
        Value::Set(idx) => *idx,
        _ => return Err(Error::TypeError("Not a Set".into())),
    };

    let callback = args.first().cloned().unwrap_or(Value::Undefined);
    let this_arg = args.get(1).cloned().unwrap_or(Value::Undefined);

    // Clone values to avoid borrow issues
    let values: Vec<Value> = if let crate::vm::interpreter::HeapValue::Set(set) = &interp.heap[idx]
    {
        set.values.clone()
    } else {
        return Err(Error::TypeError("Not a Set".into()));
    };

    for v in values {
        interp.call_value(&callback, &this_arg, &[v.clone(), v.clone(), this.clone()])?;
    }
    Ok(Value::Undefined)
}

pub(super) fn native_set_values(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let idx = match this {
        Value::Set(idx) => *idx,
        _ => return Err(Error::TypeError("Not a Set".into())),
    };

    if let crate::vm::interpreter::HeapValue::Set(set) = &interp.heap[idx] {
        let values = set.values();
        let heap_idx = interp.heap.len();
        interp.heap.push(crate::vm::interpreter::HeapValue::Array(
            crate::vm::interpreter::JsArray { elements: values },
        ));
        Ok(Value::Array(heap_idx))
    } else {
        Err(Error::TypeError("Not a Set".into()))
    }
}

pub(super) fn native_set_keys(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    // In Set, keys() is the same as values()
    native_set_values(interp, this, _args)
}

pub(super) fn native_set_entries(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let idx = match this {
        Value::Set(idx) => *idx,
        _ => return Err(Error::TypeError("Not a Set".into())),
    };

    // Clone values to avoid borrow issues
    let values: Vec<Value> = if let crate::vm::interpreter::HeapValue::Set(set) = &interp.heap[idx]
    {
        set.values.clone()
    } else {
        return Err(Error::TypeError("Not a Set".into()));
    };

    let entries: Vec<Value> = values
        .into_iter()
        .map(|v| {
            let heap_idx = interp.heap.len();
            interp.heap.push(crate::vm::interpreter::HeapValue::Array(
                crate::vm::interpreter::JsArray {
                    elements: vec![v.clone(), v.clone()],
                },
            ));
            Value::Array(heap_idx)
        })
        .collect();
    let heap_idx = interp.heap.len();
    interp.heap.push(crate::vm::interpreter::HeapValue::Array(
        crate::vm::interpreter::JsArray { elements: entries },
    ));
    Ok(Value::Array(heap_idx))
}

// WeakMap functions
pub(super) fn native_weakmap_constructor(
    interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let weakmap = JsWeakMap::new();
    let heap_idx = interp.heap.len();
    interp
        .heap
        .push(crate::vm::interpreter::HeapValue::WeakMap(weakmap));
    Ok(Value::WeakMap(heap_idx))
}

pub(super) fn native_weakmap_get(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let idx = match this {
        Value::WeakMap(idx) => *idx,
        _ => return Err(Error::TypeError("Not a WeakMap".into())),
    };

    let key = args.first().cloned().unwrap_or(Value::Undefined);

    if let crate::vm::interpreter::HeapValue::WeakMap(map) = &interp.heap[idx] {
        Ok(map.get(&key).cloned().unwrap_or(Value::Undefined))
    } else {
        Err(Error::TypeError("Not a WeakMap".into()))
    }
}

pub(super) fn native_weakmap_set(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let idx = match this {
        Value::WeakMap(idx) => *idx,
        _ => return Err(Error::TypeError("Not a WeakMap".into())),
    };

    let key = args.first().cloned().unwrap_or(Value::Undefined);
    let value = args.get(1).cloned().unwrap_or(Value::Undefined);

    if let crate::vm::interpreter::HeapValue::WeakMap(map) = &mut interp.heap[idx] {
        map.set(key, value);
        Ok(this.clone())
    } else {
        Err(Error::TypeError("Not a WeakMap".into()))
    }
}

pub(super) fn native_weakmap_has(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let idx = match this {
        Value::WeakMap(idx) => *idx,
        _ => return Err(Error::TypeError("Not a WeakMap".into())),
    };

    let key = args.first().cloned().unwrap_or(Value::Undefined);

    if let crate::vm::interpreter::HeapValue::WeakMap(map) = &interp.heap[idx] {
        Ok(Value::Boolean(map.has(&key)))
    } else {
        Err(Error::TypeError("Not a WeakMap".into()))
    }
}

pub(super) fn native_weakmap_delete(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let idx = match this {
        Value::WeakMap(idx) => *idx,
        _ => return Err(Error::TypeError("Not a WeakMap".into())),
    };

    let key = args.first().cloned().unwrap_or(Value::Undefined);

    if let crate::vm::interpreter::HeapValue::WeakMap(map) = &mut interp.heap[idx] {
        Ok(Value::Boolean(map.delete(&key)))
    } else {
        Err(Error::TypeError("Not a WeakMap".into()))
    }
}

// WeakSet functions
pub(super) fn native_weakset_constructor(
    interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let weakset = JsWeakSet::new();
    let heap_idx = interp.heap.len();
    interp
        .heap
        .push(crate::vm::interpreter::HeapValue::WeakSet(weakset));
    Ok(Value::WeakSet(heap_idx))
}

pub(super) fn native_weakset_add(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let idx = match this {
        Value::WeakSet(idx) => *idx,
        _ => return Err(Error::TypeError("Not a WeakSet".into())),
    };

    let value = args.first().cloned().unwrap_or(Value::Undefined);

    if let crate::vm::interpreter::HeapValue::WeakSet(set) = &mut interp.heap[idx] {
        set.add(value);
        Ok(this.clone())
    } else {
        Err(Error::TypeError("Not a WeakSet".into()))
    }
}

pub(super) fn native_weakset_has(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let idx = match this {
        Value::WeakSet(idx) => *idx,
        _ => return Err(Error::TypeError("Not a WeakSet".into())),
    };

    let value = args.first().cloned().unwrap_or(Value::Undefined);

    if let crate::vm::interpreter::HeapValue::WeakSet(set) = &interp.heap[idx] {
        Ok(Value::Boolean(set.has(&value)))
    } else {
        Err(Error::TypeError("Not a WeakSet".into()))
    }
}

pub(super) fn native_weakset_delete(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let idx = match this {
        Value::WeakSet(idx) => *idx,
        _ => return Err(Error::TypeError("Not a WeakSet".into())),
    };

    let value = args.first().cloned().unwrap_or(Value::Undefined);

    if let crate::vm::interpreter::HeapValue::WeakSet(set) = &mut interp.heap[idx] {
        Ok(Value::Boolean(set.delete(&value)))
    } else {
        Err(Error::TypeError("Not a WeakSet".into()))
    }
}
