use crate::errors::{Error, Result};
use crate::objects::js_collections::{JsMap, JsSet, JsWeakMap, JsWeakSet};
use crate::objects::Value;
use crate::vm::interpreter::Interpreter;

macro_rules! with_map {
    ($interp:expr, $this:expr, |$idx:ident, $map:ident| $body:block) => {{
        let $idx = match $this {
            Value::Map(idx) => *idx,
            _ => return Err(Error::TypeError("Not a Map".into())),
        };
        match &$interp.heap[$idx] {
            crate::vm::interpreter::HeapValue::Map($map) => $body,
            _ => Err(Error::TypeError("Not a Map".into())),
        }
    }};
}

macro_rules! with_map_mut {
    ($interp:expr, $this:expr, |$idx:ident, $map:ident| $body:block) => {{
        let $idx = match $this {
            Value::Map(idx) => *idx,
            _ => return Err(Error::TypeError("Not a Map".into())),
        };
        match &mut $interp.heap[$idx] {
            crate::vm::interpreter::HeapValue::Map($map) => $body,
            _ => Err(Error::TypeError("Not a Map".into())),
        }
    }};
}

macro_rules! with_set {
    ($interp:expr, $this:expr, |$idx:ident, $set:ident| $body:block) => {{
        let $idx = match $this {
            Value::Set(idx) => *idx,
            _ => return Err(Error::TypeError("Not a Set".into())),
        };
        match &$interp.heap[$idx] {
            crate::vm::interpreter::HeapValue::Set($set) => $body,
            _ => Err(Error::TypeError("Not a Set".into())),
        }
    }};
}

macro_rules! with_set_mut {
    ($interp:expr, $this:expr, |$idx:ident, $set:ident| $body:block) => {{
        let $idx = match $this {
            Value::Set(idx) => *idx,
            _ => return Err(Error::TypeError("Not a Set".into())),
        };
        match &mut $interp.heap[$idx] {
            crate::vm::interpreter::HeapValue::Set($set) => $body,
            _ => Err(Error::TypeError("Not a Set".into())),
        }
    }};
}

macro_rules! with_weakmap {
    ($interp:expr, $this:expr, |$idx:ident, $map:ident| $body:block) => {{
        let $idx = match $this {
            Value::WeakMap(idx) => *idx,
            _ => return Err(Error::TypeError("Not a WeakMap".into())),
        };
        match &$interp.heap[$idx] {
            crate::vm::interpreter::HeapValue::WeakMap($map) => $body,
            _ => Err(Error::TypeError("Not a WeakMap".into())),
        }
    }};
}

macro_rules! with_weakmap_mut {
    ($interp:expr, $this:expr, |$idx:ident, $map:ident| $body:block) => {{
        let $idx = match $this {
            Value::WeakMap(idx) => *idx,
            _ => return Err(Error::TypeError("Not a WeakMap".into())),
        };
        match &mut $interp.heap[$idx] {
            crate::vm::interpreter::HeapValue::WeakMap($map) => $body,
            _ => Err(Error::TypeError("Not a WeakMap".into())),
        }
    }};
}

macro_rules! with_weakset {
    ($interp:expr, $this:expr, |$idx:ident, $set:ident| $body:block) => {{
        let $idx = match $this {
            Value::WeakSet(idx) => *idx,
            _ => return Err(Error::TypeError("Not a WeakSet".into())),
        };
        match &$interp.heap[$idx] {
            crate::vm::interpreter::HeapValue::WeakSet($set) => $body,
            _ => Err(Error::TypeError("Not a WeakSet".into())),
        }
    }};
}

macro_rules! with_weakset_mut {
    ($interp:expr, $this:expr, |$idx:ident, $set:ident| $body:block) => {{
        let $idx = match $this {
            Value::WeakSet(idx) => *idx,
            _ => return Err(Error::TypeError("Not a WeakSet".into())),
        };
        match &mut $interp.heap[$idx] {
            crate::vm::interpreter::HeapValue::WeakSet($set) => $body,
            _ => Err(Error::TypeError("Not a WeakSet".into())),
        }
    }};
}

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
    with_map!(interp, this, |idx, map| {
        let key = args.first().cloned().unwrap_or(Value::Undefined);
        Ok(map.get(&key).cloned().unwrap_or(Value::Undefined))
    })
}

pub(super) fn native_map_set(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    with_map_mut!(interp, this, |idx, map| {
        let key = args.first().cloned().unwrap_or(Value::Undefined);
        let value = args.get(1).cloned().unwrap_or(Value::Undefined);
        map.set(key, value);
        Ok(this.clone())
    })
}

pub(super) fn native_map_has(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    with_map!(interp, this, |idx, map| {
        let key = args.first().cloned().unwrap_or(Value::Undefined);
        Ok(Value::Boolean(map.has(&key)))
    })
}

pub(super) fn native_map_delete(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    with_map_mut!(interp, this, |idx, map| {
        let key = args.first().cloned().unwrap_or(Value::Undefined);
        Ok(Value::Boolean(map.delete(&key)))
    })
}

pub(super) fn native_map_clear(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    with_map_mut!(interp, this, |_idx, map| {
        map.clear();
        Ok(Value::Undefined)
    })
}

pub(super) fn native_map_size(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    with_map!(interp, this, |_idx, map| {
        Ok(Value::Float(map.size() as f64))
    })
}

pub(super) fn native_map_for_each(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    with_map!(interp, this, |_idx, map| {
        let callback = args.first().cloned().unwrap_or(Value::Undefined);
        let this_arg = args.get(1).cloned().unwrap_or(Value::Undefined);
        let entries: Vec<(Value, Value)> = map
            .keys
            .iter()
            .zip(map.values.iter())
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        for (k, v) in entries {
            interp.call_value(&callback, &this_arg, &[v.clone(), k.clone(), this.clone()])?;
        }
        Ok(Value::Undefined)
    })
}

pub(super) fn native_map_keys(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    with_map!(interp, this, |_idx, map| {
        let keys = map.keys();
        let heap_idx = interp.heap.len();
        interp.heap.push(crate::vm::interpreter::HeapValue::Array(
            crate::vm::interpreter::JsArray { elements: keys },
        ));
        Ok(Value::Array(heap_idx))
    })
}

pub(super) fn native_map_values(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    with_map!(interp, this, |_idx, map| {
        let values = map.values();
        let heap_idx = interp.heap.len();
        interp.heap.push(crate::vm::interpreter::HeapValue::Array(
            crate::vm::interpreter::JsArray { elements: values },
        ));
        Ok(Value::Array(heap_idx))
    })
}

pub(super) fn native_map_entries(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    with_map!(interp, this, |_idx, map| {
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
    })
}

// Set functions
pub(super) fn native_set_constructor(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let mut set = JsSet::new();
    // `new Set(iterable)` — support Array and Set sources (common cases).
    if let Some(iterable) = args.first() {
        match iterable {
            Value::Array(arr_idx) => {
                if let crate::vm::interpreter::HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                    let elements = arr.elements.clone();
                    for v in elements {
                        set.add(v);
                    }
                }
            }
            Value::Set(set_idx) => {
                if let crate::vm::interpreter::HeapValue::Set(src) = &interp.heap[*set_idx] {
                    for v in src.values() {
                        set.add(v);
                    }
                }
            }
            Value::Undefined | Value::Null => {}
            _ => {}
        }
    }
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
    with_set_mut!(interp, this, |_idx, set| {
        let value = args.first().cloned().unwrap_or(Value::Undefined);
        set.add(value);
        Ok(this.clone())
    })
}

pub(super) fn native_set_has(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    with_set!(interp, this, |_idx, set| {
        let value = args.first().cloned().unwrap_or(Value::Undefined);
        Ok(Value::Boolean(set.has(&value)))
    })
}

pub(super) fn native_set_delete(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    with_set_mut!(interp, this, |_idx, set| {
        let value = args.first().cloned().unwrap_or(Value::Undefined);
        Ok(Value::Boolean(set.delete(&value)))
    })
}

pub(super) fn native_set_clear(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    with_set_mut!(interp, this, |_idx, set| {
        set.clear();
        Ok(Value::Undefined)
    })
}

pub(super) fn native_set_size(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    with_set!(interp, this, |_idx, set| {
        Ok(Value::Float(set.size() as f64))
    })
}

pub(super) fn native_set_for_each(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    with_set!(interp, this, |_idx, set| {
        let callback = args.first().cloned().unwrap_or(Value::Undefined);
        let this_arg = args.get(1).cloned().unwrap_or(Value::Undefined);
        let values: Vec<Value> = set.values.clone();
        for v in values {
            interp.call_value(&callback, &this_arg, &[v.clone(), v.clone(), this.clone()])?;
        }
        Ok(Value::Undefined)
    })
}

pub(super) fn native_set_values(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    with_set!(interp, this, |_idx, set| {
        let values = set.values();
        let heap_idx = interp.heap.len();
        interp.heap.push(crate::vm::interpreter::HeapValue::Array(
            crate::vm::interpreter::JsArray { elements: values },
        ));
        Ok(Value::Array(heap_idx))
    })
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
    with_set!(interp, this, |_idx, set| {
        let values: Vec<Value> = set.values.clone();
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
    })
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
    with_weakmap!(interp, this, |_idx, map| {
        let key = args.first().cloned().unwrap_or(Value::Undefined);
        Ok(map.get(&key).cloned().unwrap_or(Value::Undefined))
    })
}

pub(super) fn native_weakmap_set(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    with_weakmap_mut!(interp, this, |_idx, map| {
        let key = args.first().cloned().unwrap_or(Value::Undefined);
        let value = args.get(1).cloned().unwrap_or(Value::Undefined);
        map.set(key, value);
        Ok(this.clone())
    })
}

pub(super) fn native_weakmap_has(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    with_weakmap!(interp, this, |_idx, map| {
        let key = args.first().cloned().unwrap_or(Value::Undefined);
        Ok(Value::Boolean(map.has(&key)))
    })
}

pub(super) fn native_weakmap_delete(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    with_weakmap_mut!(interp, this, |_idx, map| {
        let key = args.first().cloned().unwrap_or(Value::Undefined);
        Ok(Value::Boolean(map.delete(&key)))
    })
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
    with_weakset_mut!(interp, this, |_idx, set| {
        let value = args.first().cloned().unwrap_or(Value::Undefined);
        set.add(value);
        Ok(this.clone())
    })
}

pub(super) fn native_weakset_has(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    with_weakset!(interp, this, |_idx, set| {
        let value = args.first().cloned().unwrap_or(Value::Undefined);
        Ok(Value::Boolean(set.has(&value)))
    })
}

pub(super) fn native_weakset_delete(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    with_weakset_mut!(interp, this, |_idx, set| {
        let value = args.first().cloned().unwrap_or(Value::Undefined);
        Ok(Value::Boolean(set.delete(&value)))
    })
}
