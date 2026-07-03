use crate::errors::Result;
use crate::objects::Value;
use crate::vm::interpreter::{HeapValue, Interpreter, JsArray, JsObject};

fn get_object_prop(interp: &Interpreter, idx: usize, key: &str) -> Option<Value> {
    if let HeapValue::Object(obj) = &interp.heap[idx] {
        obj.properties.get(key).cloned()
    } else {
        None
    }
}

pub(super) fn native_readable_stream_read(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let obj_idx = match this {
        Value::Object(idx) => *idx,
        _ => return Ok(Value::Null),
    };
    let state_idx = match get_object_prop(interp, obj_idx, "_readableState") {
        Some(Value::Object(idx)) => idx,
        _ => return Ok(Value::Null),
    };
    let buf_idx = match get_object_prop(interp, state_idx, "_buffer") {
        Some(Value::Array(idx)) => idx,
        _ => return Ok(Value::Null),
    };
    let chunk = if let HeapValue::Array(arr) = &interp.heap[buf_idx] {
        arr.elements.first().cloned()
    } else {
        None
    };
    if let Some(c) = chunk {
        if let HeapValue::Array(arr) = &mut interp.heap[buf_idx] {
            if !arr.elements.is_empty() {
                arr.elements.remove(0);
            }
        }
        Ok(c)
    } else {
        Ok(Value::Null)
    }
}

pub(super) fn native_readable_stream_pipe(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let dest = args.first().cloned().unwrap_or(Value::Undefined);
    let obj_idx = match this {
        Value::Object(idx) => *idx,
        _ => return Ok(this.clone()),
    };
    let state_idx = match get_object_prop(interp, obj_idx, "_readableState") {
        Some(Value::Object(idx)) => idx,
        _ => return Ok(this.clone()),
    };
    let arr_idx = interp.heap.len();
    interp
        .heap
        .push(HeapValue::Array(JsArray { elements: vec![dest] }));
    if let HeapValue::Object(state) = &mut interp.heap[state_idx] {
        state
            .properties
            .insert("_pipes".to_string(), Value::Array(arr_idx));
    }
    Ok(this.clone())
}

pub(super) fn native_readable_stream_unpipe(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let obj_idx = match this {
        Value::Object(idx) => *idx,
        _ => return Ok(this.clone()),
    };
    if let Some(Value::Object(state_idx)) = get_object_prop(interp, obj_idx, "_readableState") {
        if let HeapValue::Object(state) = &mut interp.heap[state_idx] {
            state.properties.remove("_pipes");
        }
    }
    Ok(this.clone())
}

pub(super) fn native_readable_stream_push(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let chunk = args.first().cloned().unwrap_or(Value::Undefined);
    let obj_idx = match this {
        Value::Object(idx) => *idx,
        _ => return Ok(Value::Boolean(false)),
    };

    let state_idx = match get_object_prop(interp, obj_idx, "_readableState") {
        Some(Value::Object(idx)) => idx,
        _ => {
            let new_idx = interp
                .gc
                .allocate(&mut interp.heap, HeapValue::Object(JsObject::new()));
            if let HeapValue::Object(obj) = &mut interp.heap[obj_idx] {
                obj.properties
                    .insert("_readableState".into(), Value::Object(new_idx));
            }
            new_idx
        }
    };

    let buf_idx = match get_object_prop(interp, state_idx, "_buffer") {
        Some(Value::Array(idx)) => idx,
        _ => {
            let new_idx = interp.heap.len();
            interp
                .heap
                .push(HeapValue::Array(JsArray { elements: Vec::new() }));
            if let HeapValue::Object(state) = &mut interp.heap[state_idx] {
                state
                    .properties
                    .insert("_buffer".into(), Value::Array(new_idx));
            }
            new_idx
        }
    };

    if let HeapValue::Array(arr) = &mut interp.heap[buf_idx] {
        arr.elements.push(chunk);
    }
    Ok(Value::Boolean(true))
}

pub(super) fn native_readable_stream_destroy(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let obj_idx = match this {
        Value::Object(idx) => *idx,
        _ => return Ok(this.clone()),
    };
    if let HeapValue::Object(obj) = &mut interp.heap[obj_idx] {
        obj.properties
            .insert("_readableState".into(), Value::Null);
    }
    Ok(this.clone())
}

pub(super) fn native_writable_stream_write(
    _interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    Ok(Value::Boolean(true))
}

pub(super) fn native_writable_stream_end(
    _interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    Ok(this.clone())
}

pub(super) fn native_writable_stream_destroy(
    _interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    Ok(this.clone())
}

pub(super) fn native_writable_stream_cork(
    _interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    Ok(this.clone())
}

pub(super) fn native_writable_stream_uncork(
    _interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    Ok(this.clone())
}

pub(super) fn native_stream_constructor(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let obj_idx = match this {
        Value::Object(idx) => *idx,
        _ => return Ok(this.clone()),
    };
    let state_idx = interp
        .gc
        .allocate(&mut interp.heap, HeapValue::Object(JsObject::new()));
    if let HeapValue::Object(obj) = &mut interp.heap[obj_idx] {
        obj.properties
            .insert("_readableState".into(), Value::Object(state_idx));
        obj.properties
            .insert("_writableState".into(), Value::Object(state_idx));
    }
    Ok(this.clone())
}

pub(super) fn native_passthrough_constructor(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    native_stream_constructor(interp, this, args)
}

pub(super) fn native_stream_pipeline(
    _interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    Ok(Value::Undefined)
}

pub(super) fn native_stream_finished(
    _interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    Ok(Value::Undefined)
}
