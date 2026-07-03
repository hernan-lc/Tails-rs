use crate::errors::Result;
use crate::objects::Value;
use crate::vm::interpreter::{HeapValue, Interpreter, JsArray, JsObject};

use super::helpers::to_string_value;

pub(super) fn native_event_emitter_constructor(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    // The VM already creates an object with the correct prototype and passes it as `this`.
    // We just need to add the _listeners property to it.
    let obj_idx = match this {
        Value::Object(idx) => *idx,
        _ => return Ok(this.clone()),
    };

    let listeners_idx = interp
        .gc
        .allocate(&mut interp.heap, HeapValue::Object(JsObject::new()));

    if let HeapValue::Object(obj) = &mut interp.heap[obj_idx] {
        obj.properties
            .insert("_listeners".into(), Value::Object(listeners_idx));
    }

    Ok(this.clone())
}

pub(super) fn native_event_emitter_on(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let event = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    let callback = args.get(1).cloned().unwrap_or(Value::Undefined);

    let obj_idx = match this {
        Value::Object(idx) => *idx,
        _ => return Ok(this.clone()),
    };

    let listeners_idx = match &interp.heap[obj_idx] {
        HeapValue::Object(obj) => match obj.properties.get("_listeners") {
            Some(Value::Object(idx)) => *idx,
            _ => return Ok(this.clone()),
        },
        _ => return Ok(this.clone()),
    };

    // Get or create the array for this event
    let arr_idx = match &interp.heap[listeners_idx] {
        HeapValue::Object(listeners_obj) => {
            match listeners_obj.properties.get(&event) {
                Some(Value::Array(idx)) => *idx,
                _ => {
                    // Create new array
                    let new_idx = interp.heap.len();
                    interp.heap.push(HeapValue::Array(JsArray {
                        elements: Vec::new(),
                    }));
                    new_idx
                }
            }
        }
        _ => return Ok(this.clone()),
    };

    // Add callback to array
    if let HeapValue::Array(arr_obj) = &mut interp.heap[arr_idx] {
        arr_obj.elements.push(callback);
    }

    // Update listeners map
    if let HeapValue::Object(listeners_obj) = &mut interp.heap[listeners_idx] {
        listeners_obj
            .properties
            .insert(event, Value::Array(arr_idx));
    }

    Ok(this.clone())
}

pub(super) fn native_event_emitter_emit(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let event = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    let emit_args: Vec<Value> = args.get(1..).unwrap_or(&[]).to_vec();

    let obj_idx = match this {
        Value::Object(idx) => *idx,
        _ => return Ok(Value::Boolean(false)),
    };

    let listeners_idx = match &interp.heap[obj_idx] {
        HeapValue::Object(obj) => match obj.properties.get("_listeners") {
            Some(Value::Object(idx)) => *idx,
            _ => return Ok(Value::Boolean(false)),
        },
        _ => return Ok(Value::Boolean(false)),
    };

    // Clone callbacks to avoid borrow issues
    let callbacks: Vec<Value> = match &interp.heap[listeners_idx] {
        HeapValue::Object(listeners_obj) => match listeners_obj.properties.get(&event) {
            Some(Value::Array(arr_idx)) => match &interp.heap[*arr_idx] {
                HeapValue::Array(arr_obj) => arr_obj.elements.clone(),
                _ => Vec::new(),
            },
            _ => Vec::new(),
        },
        _ => Vec::new(),
    };

    let had_listeners = !callbacks.is_empty();
    for callback in &callbacks {
        let _ = interp.call_value(callback, this, &emit_args);
    }

    Ok(Value::Boolean(had_listeners))
}

pub(super) fn native_event_emitter_off(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let event = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    let callback = args.get(1);

    let obj_idx = match this {
        Value::Object(idx) => *idx,
        _ => return Ok(this.clone()),
    };

    let listeners_idx = match &interp.heap[obj_idx] {
        HeapValue::Object(obj) => match obj.properties.get("_listeners") {
            Some(Value::Object(idx)) => *idx,
            _ => return Ok(this.clone()),
        },
        _ => return Ok(this.clone()),
    };

    let arr_idx = match &interp.heap[listeners_idx] {
        HeapValue::Object(listeners_obj) => match listeners_obj.properties.get(&event) {
            Some(Value::Array(idx)) => *idx,
            _ => return Ok(this.clone()),
        },
        _ => return Ok(this.clone()),
    };

    if let Some(cb) = callback {
        // Remove specific callback
        if let HeapValue::Array(arr_obj) = &mut interp.heap[arr_idx] {
            arr_obj.elements.retain(|v| !value_eq(v, cb));
        }
    } else {
        // Remove all listeners for this event
        if let HeapValue::Object(listeners_obj) = &mut interp.heap[listeners_idx] {
            listeners_obj.properties.remove(&event);
        }
    }

    Ok(this.clone())
}

pub(super) fn native_event_emitter_listener_count(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let event = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();

    let obj_idx = match this {
        Value::Object(idx) => *idx,
        _ => return Ok(Value::Integer(0)),
    };

    let listeners_idx = match &interp.heap[obj_idx] {
        HeapValue::Object(obj) => match obj.properties.get("_listeners") {
            Some(Value::Object(idx)) => *idx,
            _ => return Ok(Value::Integer(0)),
        },
        _ => return Ok(Value::Integer(0)),
    };

    match &interp.heap[listeners_idx] {
        HeapValue::Object(listeners_obj) => match listeners_obj.properties.get(&event) {
            Some(Value::Array(arr_idx)) => match &interp.heap[*arr_idx] {
                HeapValue::Array(arr_obj) => Ok(Value::Integer(arr_obj.elements.len() as i64)),
                _ => Ok(Value::Integer(0)),
            },
            _ => Ok(Value::Integer(0)),
        },
        _ => Ok(Value::Integer(0)),
    }
}

pub(super) fn native_event_emitter_prepend_listener(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let event = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    let callback = args.get(1).cloned().unwrap_or(Value::Undefined);

    let obj_idx = match this {
        Value::Object(idx) => *idx,
        _ => return Ok(this.clone()),
    };

    let listeners_idx = match &interp.heap[obj_idx] {
        HeapValue::Object(obj) => match obj.properties.get("_listeners") {
            Some(Value::Object(idx)) => *idx,
            _ => return Ok(this.clone()),
        },
        _ => return Ok(this.clone()),
    };

    let arr_idx = match &interp.heap[listeners_idx] {
        HeapValue::Object(listeners_obj) => match listeners_obj.properties.get(&event) {
            Some(Value::Array(idx)) => *idx,
            _ => {
                let new_idx = interp.heap.len();
                interp.heap.push(HeapValue::Array(JsArray {
                    elements: Vec::new(),
                }));
                new_idx
            }
        },
        _ => return Ok(this.clone()),
    };

    if let HeapValue::Array(arr_obj) = &mut interp.heap[arr_idx] {
        arr_obj.elements.insert(0, callback);
    }

    if let HeapValue::Object(listeners_obj) = &mut interp.heap[listeners_idx] {
        listeners_obj
            .properties
            .insert(event, Value::Array(arr_idx));
    }

    Ok(this.clone())
}

pub(super) fn native_event_emitter_once(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let event = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    let callback = args.get(1).cloned().unwrap_or(Value::Undefined);

    let wrapper_idx = interp.gc.allocate(
        &mut interp.heap,
        HeapValue::Object(JsObject {
            properties: crate::props! {
                "_callback" => callback,
                "_event" => Value::String(event.clone()),
                "_once" => Value::Boolean(true),
            },
            prototype: None,
            extensible: true,
        }),
    );

    let _ = native_event_emitter_on(interp, this, &[Value::String(event), Value::Object(wrapper_idx)]);
    Ok(this.clone())
}

pub(super) fn native_event_emitter_remove_all_listeners(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let event = args.first().map(|v| to_string_value(interp, v));

    let obj_idx = match this {
        Value::Object(idx) => *idx,
        _ => return Ok(this.clone()),
    };

    let listeners_idx = match &interp.heap[obj_idx] {
        HeapValue::Object(obj) => match obj.properties.get("_listeners") {
            Some(Value::Object(idx)) => *idx,
            _ => return Ok(this.clone()),
        },
        _ => return Ok(this.clone()),
    };

    if let Some(evt) = event {
        if let HeapValue::Object(listeners_obj) = &mut interp.heap[listeners_idx] {
            listeners_obj.properties.remove(&evt);
        }
    } else {
        if let HeapValue::Object(listeners_obj) = &mut interp.heap[listeners_idx] {
            listeners_obj.properties.clear();
        }
    }

    Ok(this.clone())
}

pub(super) fn native_event_emitter_event_names(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let obj_idx = match this {
        Value::Object(idx) => *idx,
        _ => {
            let arr_idx = interp.heap.len();
            interp.heap.push(HeapValue::Array(JsArray {
                elements: Vec::new(),
            }));
            return Ok(Value::Array(arr_idx));
        }
    };

    let listeners_idx = match &interp.heap[obj_idx] {
        HeapValue::Object(obj) => match obj.properties.get("_listeners") {
            Some(Value::Object(idx)) => *idx,
            _ => {
                let arr_idx = interp.heap.len();
                interp.heap.push(HeapValue::Array(JsArray {
                    elements: Vec::new(),
                }));
                return Ok(Value::Array(arr_idx));
            }
        },
        _ => {
            let arr_idx = interp.heap.len();
            interp.heap.push(HeapValue::Array(JsArray {
                elements: Vec::new(),
            }));
            return Ok(Value::Array(arr_idx));
        }
    };

    let mut names = Vec::new();
    if let HeapValue::Object(listeners_obj) = &interp.heap[listeners_idx] {
        for (key, val) in &listeners_obj.properties {
            if let Value::Array(arr_idx) = val {
                if let HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                    if !arr.elements.is_empty() {
                        names.push(Value::String(key.clone()));
                    }
                }
            }
        }
    }

    let arr_idx = interp.heap.len();
    interp
        .heap
        .push(HeapValue::Array(JsArray { elements: names }));
    Ok(Value::Array(arr_idx))
}

pub(super) fn native_event_emitter_get_max_listeners(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let obj_idx = match this {
        Value::Object(idx) => *idx,
        _ => return Ok(Value::Integer(10)),
    };

    match &interp.heap[obj_idx] {
        HeapValue::Object(obj) => match obj.properties.get("_maxListeners") {
            Some(val) => Ok(val.clone()),
            _ => Ok(Value::Integer(10)),
        },
        _ => Ok(Value::Integer(10)),
    }
}

pub(super) fn native_event_emitter_set_max_listeners(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let n = args
        .first()
        .map(|v| match v {
            Value::Integer(i) => *i,
            Value::Float(f) => *f as i64,
            _ => 10,
        })
        .unwrap_or(10);

    let obj_idx = match this {
        Value::Object(idx) => *idx,
        _ => return Ok(this.clone()),
    };

    if let HeapValue::Object(obj) = &mut interp.heap[obj_idx] {
        obj.properties
            .insert("_maxListeners".into(), Value::Integer(n));
    }

    Ok(this.clone())
}

pub(super) fn native_event_emitter_prepend_once_listener(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let event = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    let callback = args.get(1).cloned().unwrap_or(Value::Undefined);

    let wrapper_idx = interp.gc.allocate(
        &mut interp.heap,
        HeapValue::Object(JsObject {
            properties: crate::props! {
                "_callback" => callback,
                "_event" => Value::String(event.clone()),
                "_once" => Value::Boolean(true),
            },
            prototype: None,
            extensible: true,
        }),
    );

    let _ = native_event_emitter_prepend_listener(
        interp,
        this,
        &[Value::String(event), Value::Object(wrapper_idx)],
    );
    Ok(this.clone())
}

fn value_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Integer(x), Value::Integer(y)) => x == y,
        (Value::Float(x), Value::Float(y)) => x == y,
        (Value::String(x), Value::String(y)) => x == y,
        (Value::Cons(x), Value::String(y)) => x.flatten() == *y,
        (Value::String(x), Value::Cons(y)) => *x == y.flatten(),
        (Value::Cons(x), Value::Cons(y)) => x.flatten() == y.flatten(),
        (Value::Boolean(x), Value::Boolean(y)) => x == y,
        (Value::Undefined, Value::Undefined) => true,
        (Value::Null, Value::Null) => true,
        (Value::Object(x), Value::Object(y)) => x == y,
        (Value::Array(x), Value::Array(y)) => x == y,
        (Value::Function(x), Value::Function(y)) => x == y,
        (Value::NativeFunction(x), Value::NativeFunction(y)) => x == y,
        _ => false,
    }
}
