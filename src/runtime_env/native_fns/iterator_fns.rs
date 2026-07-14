use crate::errors::{Error, Result};
use crate::objects::Value;
use crate::props;
use crate::runtime_env::native_fns::constants as c;
use crate::vm::interpreter::{HeapValue, Interpreter, JsArray, JsIterator, JsObject};

// Array[Symbol.iterator]() - creates an iterator for an array
pub(super) fn native_array_iterator(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    // this should be the array
    let arr_data = match this {
        Value::Array(arr_idx) => {
            if let HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                arr.elements.clone()
            } else {
                Vec::new()
            }
        }
        _ => Vec::new(),
    };

    make_array_style_iterator(interp, arr_data)
}

/// String.prototype[Symbol.iterator]() — yields each code unit / char as a string.
pub(super) fn native_string_iterator(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let s = match this {
        Value::String(s) => s.to_string(),
        Value::Cons(c) => c.flatten(),
        _ => String::new(),
    };
    let chars: Vec<Value> = s
        .chars()
        .map(|ch| Value::from_string(ch.to_string()))
        .collect();
    make_array_style_iterator(interp, chars)
}

fn make_array_style_iterator(interp: &mut Interpreter, arr_data: Vec<Value>) -> Result<Value> {
    let data_idx = interp.gc.allocate(
        &mut interp.heap,
        HeapValue::Array(JsArray { elements: arr_data }),
    );
    let props = props! {
        "__type" => Value::from_string("array".to_string()),
        "__index" => Value::Integer(0),
        "__data" => Value::Array(data_idx),
        // ES iterator protocol — required by `for (x of y[Symbol.iterator]())`
        // and by libraries that call `.next()` directly.
        "next" => Value::NativeFunction(c::ITERATOR_NEXT),
        "map" => Value::NativeFunction(c::ITERATOR_MAP),
        "filter" => Value::NativeFunction(c::ITERATOR_FILTER),
        "take" => Value::NativeFunction(c::ITERATOR_TAKE),
        "drop" => Value::NativeFunction(c::ITERATOR_DROP),
        "forEach" => Value::NativeFunction(c::ITERATOR_FOR_EACH),
        "toArray" => Value::NativeFunction(c::ITERATOR_TO_ARRAY),
    };

    let iter_idx = interp.gc.allocate(
        &mut interp.heap,
        HeapValue::Object(JsObject {
            properties: props,
            prototype: None,
            extensible: true,
        }),
    );
    Ok(Value::Object(iter_idx))
}

/// Iterator.prototype.next() — returns `{ value, done }` per the ES protocol.
pub(super) fn native_iterator_next(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let next_value = advance_iterator(interp, this)?;
    let (value, done) = match next_value {
        Some(v) => (v, false),
        None => (Value::Undefined, true),
    };
    let props = props! {
        "value" => value,
        "done" => Value::Boolean(done),
    };
    let idx = interp.gc.allocate(
        &mut interp.heap,
        HeapValue::Object(JsObject {
            properties: props,
            prototype: interp.object_proto_idx,
            extensible: true,
        }),
    );
    Ok(Value::Object(idx))
}

// Iterator.prototype.map(callback)
pub(super) fn native_iterator_map(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let callback = args.first().cloned().unwrap_or(Value::Undefined);
    if !matches!(callback, Value::Function(_) | Value::NativeFunction(_)) {
        return Err(Error::TypeError(
            "Iterator.map requires a callback function".into(),
        ));
    }

    // Create a wrapper iterator object
    let props = props! {
        "__type" => Value::from_string("mapped".to_string()),
        "__source" => this.clone(),
        "__callback" => callback,
        "__done" => Value::Boolean(false),
        "next" => Value::NativeFunction(c::ITERATOR_NEXT),
        "map" => Value::NativeFunction(c::ITERATOR_MAP),
        "filter" => Value::NativeFunction(c::ITERATOR_FILTER),
        "take" => Value::NativeFunction(c::ITERATOR_TAKE),
        "drop" => Value::NativeFunction(c::ITERATOR_DROP),
        "forEach" => Value::NativeFunction(c::ITERATOR_FOR_EACH),
        "toArray" => Value::NativeFunction(c::ITERATOR_TO_ARRAY),
    };

    let iter_idx = interp.gc.allocate(
        &mut interp.heap,
        HeapValue::Object(JsObject {
            properties: props,
            prototype: None,
            extensible: true,
        }),
    );
    Ok(Value::Object(iter_idx))
}

// Iterator.prototype.filter(callback)
pub(super) fn native_iterator_filter(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let callback = args.first().cloned().unwrap_or(Value::Undefined);
    if !matches!(callback, Value::Function(_) | Value::NativeFunction(_)) {
        return Err(Error::TypeError(
            "Iterator.filter requires a callback function".into(),
        ));
    }

    let props = props! {
        "__type" => Value::from_string("filtered".to_string()),
        "__source" => this.clone(),
        "__callback" => callback,
        "__done" => Value::Boolean(false),
        "next" => Value::NativeFunction(c::ITERATOR_NEXT),
        "map" => Value::NativeFunction(c::ITERATOR_MAP),
        "filter" => Value::NativeFunction(c::ITERATOR_FILTER),
        "take" => Value::NativeFunction(c::ITERATOR_TAKE),
        "drop" => Value::NativeFunction(c::ITERATOR_DROP),
        "forEach" => Value::NativeFunction(c::ITERATOR_FOR_EACH),
        "toArray" => Value::NativeFunction(c::ITERATOR_TO_ARRAY),
    };

    let iter_idx = interp.gc.allocate(
        &mut interp.heap,
        HeapValue::Object(JsObject {
            properties: props,
            prototype: None,
            extensible: true,
        }),
    );
    Ok(Value::Object(iter_idx))
}

// Iterator.prototype.take(count)
pub(super) fn native_iterator_take(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let count = match args.first() {
        Some(Value::Integer(n)) => *n,
        Some(Value::Float(n)) => *n as i64,
        _ => 0,
    };

    let props = props! {
        "__type" => Value::from_string("taking".to_string()),
        "__source" => this.clone(),
        "__remaining" => Value::Integer(count),
        "__done" => Value::Boolean(false),
        "next" => Value::NativeFunction(c::ITERATOR_NEXT),
        "map" => Value::NativeFunction(c::ITERATOR_MAP),
        "filter" => Value::NativeFunction(c::ITERATOR_FILTER),
        "take" => Value::NativeFunction(c::ITERATOR_TAKE),
        "drop" => Value::NativeFunction(c::ITERATOR_DROP),
        "forEach" => Value::NativeFunction(c::ITERATOR_FOR_EACH),
        "toArray" => Value::NativeFunction(c::ITERATOR_TO_ARRAY),
    };

    let iter_idx = interp.gc.allocate(
        &mut interp.heap,
        HeapValue::Object(JsObject {
            properties: props,
            prototype: None,
            extensible: true,
        }),
    );
    Ok(Value::Object(iter_idx))
}

// Iterator.prototype.drop(count)
pub(super) fn native_iterator_drop(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let count = match args.first() {
        Some(Value::Integer(n)) => *n,
        Some(Value::Float(n)) => *n as i64,
        _ => 0,
    };

    let props = props! {
        "__type" => Value::from_string("dropping".to_string()),
        "__source" => this.clone(),
        "__remaining" => Value::Integer(count),
        "__done" => Value::Boolean(false),
        "next" => Value::NativeFunction(c::ITERATOR_NEXT),
        "map" => Value::NativeFunction(c::ITERATOR_MAP),
        "filter" => Value::NativeFunction(c::ITERATOR_FILTER),
        "take" => Value::NativeFunction(c::ITERATOR_TAKE),
        "drop" => Value::NativeFunction(c::ITERATOR_DROP),
        "forEach" => Value::NativeFunction(c::ITERATOR_FOR_EACH),
        "toArray" => Value::NativeFunction(c::ITERATOR_TO_ARRAY),
    };

    let iter_idx = interp.gc.allocate(
        &mut interp.heap,
        HeapValue::Object(JsObject {
            properties: props,
            prototype: None,
            extensible: true,
        }),
    );
    Ok(Value::Object(iter_idx))
}

// Iterator.prototype.forEach(callback)
pub(super) fn native_iterator_for_each(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let callback = args.first().cloned().unwrap_or(Value::Undefined);
    if !matches!(callback, Value::Function(_) | Value::NativeFunction(_)) {
        return Err(Error::TypeError(
            "Iterator.forEach requires a callback function".into(),
        ));
    }

    // Eagerly consume the iterator
    let mut index = 0i64;
    loop {
        let next_value = advance_iterator(interp, this)?;
        match next_value {
            Some(value) => {
                interp.call_value(
                    &callback,
                    &Value::Undefined,
                    &[value, Value::Integer(index)],
                )?;
                index += 1;
            }
            None => break,
        }
    }

    Ok(Value::Undefined)
}

// Iterator.prototype.toArray()
pub(super) fn native_iterator_to_array(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let mut elements = Vec::new();

    loop {
        let next_value = advance_iterator(interp, this)?;
        match next_value {
            Some(value) => elements.push(value),
            None => break,
        }
    }

    let arr_idx = interp.heap.len();
    interp.heap.push(HeapValue::Array(JsArray { elements }));
    Ok(Value::Array(arr_idx))
}

/// Advance a raw `HeapValue::Iterator` by one step.
/// These iterators are created by `exec_get_iterator` for built‑in
/// collection types (Array, String, Cons, Map, Set) when the collection
/// itself is not an Object with an iterator protocol.  They carry their
/// iteration state inline (`kind`, `index`, `target`, `data`) so we can
/// drive them without going through property access.
///
/// Takes the iterator state by value (cloned), returns the updated state
/// alongside the next value so the caller can write it back without
/// fighting the borrow checker (interp.heap is borrowed immutably for
/// the target/data lookups).
fn advance_raw_iterator(
    interp: &mut Interpreter,
    mut js_iter: JsIterator,
) -> Result<(Option<Value>, JsIterator)> {
    match js_iter.kind.as_str() {
        "array" | "string" => {
            let data_idx = match js_iter.data {
                Some(Value::Array(idx)) => idx,
                _ => return Ok((None, js_iter)),
            };
            let index = js_iter.index;
            let done = if let HeapValue::Array(ref arr) = interp.heap[data_idx] {
                index >= arr.elements.len()
            } else {
                true
            };
            if done {
                return Ok((None, js_iter));
            }
            let value = if let HeapValue::Array(ref arr) = interp.heap[data_idx] {
                arr.elements[index].clone()
            } else {
                return Ok((None, js_iter));
            };
            js_iter.index = index + 1;
            Ok((Some(value), js_iter))
        }
        "set" => {
            let set_idx = match js_iter.target {
                Some(Value::Set(idx)) => idx,
                _ => return Ok((None, js_iter)),
            };
            let index = js_iter.index;
            let values: Vec<Value> = if let HeapValue::Set(ref set) = interp.heap[set_idx] {
                set.values()
            } else {
                return Ok((None, js_iter));
            };
            if index >= values.len() {
                return Ok((None, js_iter));
            }
            js_iter.index = index + 1;
            Ok((Some(values[index].clone()), js_iter))
        }
        "map" => {
            let map_idx = match js_iter.target {
                Some(Value::Map(idx)) => idx,
                _ => return Ok((None, js_iter)),
            };
            let index = js_iter.index;
            let entries: Vec<(Value, Value)> = if let HeapValue::Map(ref map) = interp.heap[map_idx]
            {
                map.entries()
            } else {
                return Ok((None, js_iter));
            };
            if index >= entries.len() {
                return Ok((None, js_iter));
            }
            js_iter.index = index + 1;
            let (k, v) = &entries[index];
            let pair_idx = interp.gc.allocate(
                &mut interp.heap,
                HeapValue::Array(JsArray {
                    elements: vec![k.clone(), v.clone()],
                }),
            );
            Ok((Some(Value::Array(pair_idx)), js_iter))
        }
        _ => Ok((None, js_iter)),
    }
}

// Helper: advance an iterator by one step
fn advance_iterator(interp: &mut Interpreter, iterator: &Value) -> Result<Option<Value>> {
    match iterator {
        Value::Object(iter_idx) => {
            let iter_idx = *iter_idx;
            // If the heap slot is a raw HeapValue::Iterator (created by
            // exec_get_iterator for non‑Array/String types), drive it directly.
            // Clone the iterator state out, advance it, then write back.
            let js_iter_clone = if let HeapValue::Iterator(ref js_iter) = interp.heap[iter_idx] {
                js_iter.clone()
            } else {
                JsIterator {
                    kind: String::new(),
                    index: 0,
                    target: None,
                    data: None,
                }
            };
            if let HeapValue::Iterator(_) = interp.heap[iter_idx] {
                let (value, updated) = advance_raw_iterator(interp, js_iter_clone)?;
                // Write back updated iterator state.
                if let HeapValue::Iterator(ref mut slot) = interp.heap[iter_idx] {
                    *slot = updated;
                }
                return Ok(value);
            }
            // Otherwise it is a built‑in Object‑style iterator.
            let iter_type = if let HeapValue::Object(ref obj) = interp.heap[iter_idx] {
                obj.properties.get("__type").cloned()
            } else {
                None
            };

            match iter_type {
                Some(Value::String(ref ty)) if **ty == *"array" || **ty == *"string" => {
                    // Built-in array/string iterator
                    let (index, data_idx) =
                        if let HeapValue::Object(ref obj) = interp.heap[iter_idx] {
                            let index = match obj.properties.get("__index") {
                                Some(Value::Integer(i)) => *i as usize,
                                _ => 0,
                            };
                            let data_idx = match obj.properties.get("__data") {
                                Some(Value::Array(idx)) => *idx,
                                _ => return Ok(None),
                            };
                            (index, data_idx)
                        } else {
                            return Ok(None);
                        };

                    let done = if let HeapValue::Array(ref arr) = interp.heap[data_idx] {
                        index >= arr.elements.len()
                    } else {
                        true
                    };

                    if done {
                        return Ok(None);
                    }

                    let value = if let HeapValue::Array(ref arr) = interp.heap[data_idx] {
                        arr.elements[index].clone()
                    } else {
                        return Ok(None);
                    };

                    // Update index
                    if let HeapValue::Object(ref mut obj) = interp.heap[iter_idx] {
                        obj.properties
                            .insert("__index".to_string(), Value::Integer((index + 1) as i64));
                    }

                    Ok(Some(value))
                }
                Some(Value::String(ref ty)) if **ty == *"mapped" => {
                    // Mapped iterator: get source value, apply callback
                    let (source, callback) =
                        if let HeapValue::Object(ref obj) = interp.heap[iter_idx] {
                            let source = obj
                                .properties
                                .get("__source")
                                .cloned()
                                .unwrap_or(Value::Undefined);
                            let callback = obj
                                .properties
                                .get("__callback")
                                .cloned()
                                .unwrap_or(Value::Undefined);
                            (source, callback)
                        } else {
                            return Ok(None);
                        };

                    let source_value = advance_iterator(interp, &source)?;
                    match source_value {
                        Some(value) => {
                            let mapped =
                                interp.call_value(&callback, &Value::Undefined, &[value])?;
                            Ok(Some(mapped))
                        }
                        None => Ok(None),
                    }
                }
                Some(Value::String(ref ty)) if **ty == *"filtered" => {
                    // Filtered iterator: get source values until callback returns true
                    let (source, callback) =
                        if let HeapValue::Object(ref obj) = interp.heap[iter_idx] {
                            let source = obj
                                .properties
                                .get("__source")
                                .cloned()
                                .unwrap_or(Value::Undefined);
                            let callback = obj
                                .properties
                                .get("__callback")
                                .cloned()
                                .unwrap_or(Value::Undefined);
                            (source, callback)
                        } else {
                            return Ok(None);
                        };

                    loop {
                        let source_value = advance_iterator(interp, &source)?;
                        match source_value {
                            Some(value) => {
                                let result = interp.call_value(
                                    &callback,
                                    &Value::Undefined,
                                    std::slice::from_ref(&value),
                                )?;
                                if interp.is_truthy(&result) {
                                    return Ok(Some(value));
                                }
                                // Continue to next value
                            }
                            None => return Ok(None),
                        }
                    }
                }
                Some(Value::String(ref ty)) if **ty == *"taking" => {
                    // Taking iterator: return first N values
                    let (source, remaining) =
                        if let HeapValue::Object(ref obj) = interp.heap[iter_idx] {
                            let source = obj
                                .properties
                                .get("__source")
                                .cloned()
                                .unwrap_or(Value::Undefined);
                            let remaining = match obj.properties.get("__remaining") {
                                Some(Value::Integer(n)) => *n,
                                _ => 0,
                            };
                            (source, remaining)
                        } else {
                            return Ok(None);
                        };

                    if remaining <= 0 {
                        return Ok(None);
                    }

                    let source_value = advance_iterator(interp, &source)?;
                    match source_value {
                        Some(value) => {
                            // Decrement remaining
                            if let HeapValue::Object(ref mut obj) = interp.heap[iter_idx] {
                                obj.properties.insert(
                                    "__remaining".to_string(),
                                    Value::Integer(remaining - 1),
                                );
                            }
                            Ok(Some(value))
                        }
                        None => Ok(None),
                    }
                }
                Some(Value::String(ref ty)) if **ty == *"dropping" => {
                    // Dropping iterator: skip first N values
                    let (source, remaining) =
                        if let HeapValue::Object(ref obj) = interp.heap[iter_idx] {
                            let source = obj
                                .properties
                                .get("__source")
                                .cloned()
                                .unwrap_or(Value::Undefined);
                            let remaining = match obj.properties.get("__remaining") {
                                Some(Value::Integer(n)) => *n,
                                _ => 0,
                            };
                            (source, remaining)
                        } else {
                            return Ok(None);
                        };

                    // Skip values if needed
                    let mut remaining = remaining;
                    while remaining > 0 {
                        let skipped = advance_iterator(interp, &source)?;
                        if skipped.is_none() {
                            return Ok(None);
                        }
                        remaining -= 1;
                        if let HeapValue::Object(ref mut obj) = interp.heap[iter_idx] {
                            obj.properties
                                .insert("__remaining".to_string(), Value::Integer(remaining));
                        }
                    }

                    // Now return the next value
                    advance_iterator(interp, &source)
                }
                _ => {
                    // Generic iterator - call .next() via native function dispatch
                    // This case won't work with private methods, so we return None
                    // The for...of loop handles generic iterators directly
                    Ok(None)
                }
            }
        }
        _ => Ok(None),
    }
}
