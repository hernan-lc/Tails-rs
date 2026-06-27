use crate::errors::Result;
use crate::objects::Value;
use crate::vm::interpreter::Interpreter;

pub(super) fn native_promise_constructor(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let executor = args.first().cloned().unwrap_or(Value::Undefined);

    let promise_idx = interp.heap.len();
    interp.heap.push(crate::vm::interpreter::HeapValue::Promise(
        crate::objects::js_promise::JsPromise::new(),
    ));

    if matches!(executor, Value::Function(_) | Value::NativeFunction(_)) {
        let resolve_fn = interp.create_resolve_fn(promise_idx);
        let reject_fn = interp.create_reject_fn(promise_idx);
        interp.call_value(&executor, &Value::Undefined, &[resolve_fn, reject_fn])?;
    }

    Ok(Value::Promise(promise_idx))
}

pub(super) fn native_promise_then(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let callback = args.first().cloned().unwrap_or(Value::Undefined);
    let on_fulfilled = if let Value::Promise(idx) = this {
        *idx
    } else {
        return Ok(Value::Undefined);
    };

    let new_promise_idx = interp.heap.len();
    interp.heap.push(crate::vm::interpreter::HeapValue::Promise(
        crate::objects::js_promise::JsPromise::new(),
    ));

    let state_snapshot =
        if let crate::vm::interpreter::HeapValue::Promise(promise) = &interp.heap[on_fulfilled] {
            promise.state.clone()
        } else {
            return Ok(Value::Promise(new_promise_idx));
        };

    match state_snapshot {
        crate::objects::js_promise::PromiseState::Fulfilled(value) => {
            let result = interp.call_value(&callback, &Value::Undefined, &[value])?;
            interp.resolve_promise(new_promise_idx, result);
        }
        crate::objects::js_promise::PromiseState::Rejected(reason) => {
            interp.reject_promise(new_promise_idx, reason);
        }
        crate::objects::js_promise::PromiseState::Pending => {
            if let crate::vm::interpreter::HeapValue::Promise(promise) =
                &mut interp.heap[on_fulfilled]
            {
                let cb_idx = match callback {
                    Value::Function(idx) => idx,
                    _ => 0,
                };
                promise
                    .then_handlers
                    .push(crate::objects::js_promise::PromiseHandler {
                        callback: cb_idx,
                        resolve: true,
                    });
            }
        }
    }

    Ok(Value::Promise(new_promise_idx))
}

pub(super) fn native_promise_catch(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let callback = args.first().cloned().unwrap_or(Value::Undefined);
    let on_rejected = if let Value::Promise(idx) = this {
        *idx
    } else {
        return Ok(Value::Undefined);
    };

    let new_promise_idx = interp.heap.len();
    interp.heap.push(crate::vm::interpreter::HeapValue::Promise(
        crate::objects::js_promise::JsPromise::new(),
    ));

    let state_snapshot =
        if let crate::vm::interpreter::HeapValue::Promise(promise) = &interp.heap[on_rejected] {
            promise.state.clone()
        } else {
            return Ok(Value::Promise(new_promise_idx));
        };

    match state_snapshot {
        crate::objects::js_promise::PromiseState::Rejected(reason) => {
            let result = interp.call_value(&callback, &Value::Undefined, &[reason])?;
            interp.resolve_promise(new_promise_idx, result);
        }
        crate::objects::js_promise::PromiseState::Fulfilled(value) => {
            interp.resolve_promise(new_promise_idx, value);
        }
        crate::objects::js_promise::PromiseState::Pending => {
            if let crate::vm::interpreter::HeapValue::Promise(promise) =
                &mut interp.heap[on_rejected]
            {
                let cb_idx = match callback {
                    Value::Function(idx) => idx,
                    _ => 0,
                };
                promise
                    .catch_handlers
                    .push(crate::objects::js_promise::PromiseHandler {
                        callback: cb_idx,
                        resolve: false,
                    });
            }
        }
    }

    Ok(Value::Promise(new_promise_idx))
}

pub(super) fn native_promise_finally(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let callback = args.first().cloned().unwrap_or(Value::Undefined);
    let on_finally = if let Value::Promise(idx) = this {
        *idx
    } else {
        return Ok(Value::Undefined);
    };

    let new_promise_idx = interp.heap.len();
    interp.heap.push(crate::vm::interpreter::HeapValue::Promise(
        crate::objects::js_promise::JsPromise::new(),
    ));

    let state_snapshot =
        if let crate::vm::interpreter::HeapValue::Promise(promise) = &interp.heap[on_finally] {
            promise.state.clone()
        } else {
            return Ok(Value::Promise(new_promise_idx));
        };

    match state_snapshot {
        crate::objects::js_promise::PromiseState::Fulfilled(value)
        | crate::objects::js_promise::PromiseState::Rejected(value) => {
            let _ = interp.call_value(&callback, &Value::Undefined, &[])?;
            interp.resolve_promise(new_promise_idx, value);
        }
        crate::objects::js_promise::PromiseState::Pending => {
            if let crate::vm::interpreter::HeapValue::Promise(promise) =
                &mut interp.heap[on_finally]
            {
                let cb_idx = match callback {
                    Value::Function(idx) => idx,
                    _ => 0,
                };
                promise
                    .finally_handlers
                    .push(crate::objects::js_promise::PromiseHandler {
                        callback: cb_idx,
                        resolve: true,
                    });
            }
        }
    }

    Ok(Value::Promise(new_promise_idx))
}

pub(super) fn native_promise_resolve(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let value = args.first().cloned().unwrap_or(Value::Undefined);

    let promise_idx = interp.heap.len();
    interp.heap.push(crate::vm::interpreter::HeapValue::Promise(
        crate::objects::js_promise::JsPromise::new(),
    ));

    interp.resolve_promise(promise_idx, value);
    Ok(Value::Promise(promise_idx))
}

pub(super) fn native_promise_reject(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let reason = args.first().cloned().unwrap_or(Value::Undefined);

    let promise_idx = interp.heap.len();
    interp.heap.push(crate::vm::interpreter::HeapValue::Promise(
        crate::objects::js_promise::JsPromise::new(),
    ));

    interp.reject_promise(promise_idx, reason);
    Ok(Value::Promise(promise_idx))
}

pub(super) fn native_promise_all(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let promises_arg = args.first().cloned().unwrap_or(Value::Undefined);

    let promise_indices: Vec<usize> = match &promises_arg {
        Value::Array(arr_idx) => {
            if let crate::vm::interpreter::HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                arr.elements
                    .iter()
                    .filter_map(|v| {
                        if let Value::Promise(idx) = v {
                            Some(*idx)
                        } else {
                            None
                        }
                    })
                    .collect()
            } else {
                Vec::new()
            }
        }
        _ => Vec::new(),
    };

    let result_idx = interp.heap.len();
    interp.heap.push(crate::vm::interpreter::HeapValue::Promise(
        crate::objects::js_promise::JsPromise::new(),
    ));

    let mut all_fulfilled = true;
    let mut values = Vec::new();
    let mut first_rejection: Option<Value> = None;

    for &p_idx in &promise_indices {
        if let crate::vm::interpreter::HeapValue::Promise(p) = &interp.heap[p_idx] {
            match &p.state {
                crate::objects::js_promise::PromiseState::Fulfilled(v) => {
                    values.push(v.clone());
                }
                crate::objects::js_promise::PromiseState::Rejected(r) => {
                    if first_rejection.is_none() {
                        first_rejection = Some(r.clone());
                    }
                }
                crate::objects::js_promise::PromiseState::Pending => {
                    all_fulfilled = false;
                }
            }
        }
    }

    if let Some(reason) = first_rejection {
        interp.reject_promise(result_idx, reason);
    } else if all_fulfilled {
        let heap_idx = interp.heap.len();
        interp.heap.push(crate::vm::interpreter::HeapValue::Array(
            crate::vm::interpreter::JsArray { elements: values },
        ));
        interp.resolve_promise(result_idx, Value::Array(heap_idx));
    }

    Ok(Value::Promise(result_idx))
}

pub(super) fn native_promise_race(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let promises_arg = args.first().cloned().unwrap_or(Value::Undefined);

    let promise_indices: Vec<usize> = match &promises_arg {
        Value::Array(arr_idx) => {
            if let crate::vm::interpreter::HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                arr.elements
                    .iter()
                    .filter_map(|v| {
                        if let Value::Promise(idx) = v {
                            Some(*idx)
                        } else {
                            None
                        }
                    })
                    .collect()
            } else {
                Vec::new()
            }
        }
        _ => Vec::new(),
    };

    let result_idx = interp.heap.len();
    interp.heap.push(crate::vm::interpreter::HeapValue::Promise(
        crate::objects::js_promise::JsPromise::new(),
    ));

    for &p_idx in &promise_indices {
        if let crate::vm::interpreter::HeapValue::Promise(p) = &interp.heap[p_idx] {
            match &p.state {
                crate::objects::js_promise::PromiseState::Fulfilled(v) => {
                    interp.resolve_promise(result_idx, v.clone());
                    return Ok(Value::Promise(result_idx));
                }
                crate::objects::js_promise::PromiseState::Rejected(r) => {
                    interp.reject_promise(result_idx, r.clone());
                    return Ok(Value::Promise(result_idx));
                }
                crate::objects::js_promise::PromiseState::Pending => {}
            }
        }
    }

    Ok(Value::Promise(result_idx))
}

pub(super) fn native_promise_all_settled(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let promises_arg = args.first().cloned().unwrap_or(Value::Undefined);

    let promise_indices: Vec<usize> = match &promises_arg {
        Value::Array(arr_idx) => {
            if let crate::vm::interpreter::HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                arr.elements
                    .iter()
                    .filter_map(|v| {
                        if let Value::Promise(idx) = v {
                            Some(*idx)
                        } else {
                            None
                        }
                    })
                    .collect()
            } else {
                Vec::new()
            }
        }
        _ => Vec::new(),
    };

    let result_idx = interp.heap.len();
    interp.heap.push(crate::vm::interpreter::HeapValue::Promise(
        crate::objects::js_promise::JsPromise::new(),
    ));

    let mut all_settled = true;
    let mut results = Vec::new();

    for &p_idx in &promise_indices {
        if let crate::vm::interpreter::HeapValue::Promise(p) = &interp.heap[p_idx] {
            match &p.state {
                crate::objects::js_promise::PromiseState::Fulfilled(v) => {
                    let mut props = std::collections::HashMap::new();
                    props.insert("status".to_string(), Value::String("fulfilled".to_string()));
                    props.insert("value".to_string(), v.clone());
                    let obj_idx = interp.heap.len();
                    interp.heap.push(crate::vm::interpreter::HeapValue::Object(
                        crate::vm::interpreter::JsObject {
                            properties: props,
                            prototype: None,
                            extensible: true,
                        },
                    ));
                    results.push(Value::Object(obj_idx));
                }
                crate::objects::js_promise::PromiseState::Rejected(r) => {
                    let mut props = std::collections::HashMap::new();
                    props.insert("status".to_string(), Value::String("rejected".to_string()));
                    props.insert("reason".to_string(), r.clone());
                    let obj_idx = interp.heap.len();
                    interp.heap.push(crate::vm::interpreter::HeapValue::Object(
                        crate::vm::interpreter::JsObject {
                            properties: props,
                            prototype: None,
                            extensible: true,
                        },
                    ));
                    results.push(Value::Object(obj_idx));
                }
                crate::objects::js_promise::PromiseState::Pending => {
                    all_settled = false;
                }
            }
        }
    }

    if all_settled {
        let heap_idx = interp.heap.len();
        interp.heap.push(crate::vm::interpreter::HeapValue::Array(
            crate::vm::interpreter::JsArray { elements: results },
        ));
        interp.resolve_promise(result_idx, Value::Array(heap_idx));
    }

    Ok(Value::Promise(result_idx))
}

pub(super) fn native_promise_any(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let promises_arg = args.first().cloned().unwrap_or(Value::Undefined);

    let promise_indices: Vec<usize> = match &promises_arg {
        Value::Array(arr_idx) => {
            if let crate::vm::interpreter::HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                arr.elements
                    .iter()
                    .filter_map(|v| {
                        if let Value::Promise(idx) = v {
                            Some(*idx)
                        } else {
                            None
                        }
                    })
                    .collect()
            } else {
                Vec::new()
            }
        }
        _ => Vec::new(),
    };

    let result_idx = interp.heap.len();
    interp.heap.push(crate::vm::interpreter::HeapValue::Promise(
        crate::objects::js_promise::JsPromise::new(),
    ));

    let mut all_rejected = true;
    let mut rejections = Vec::new();

    for &p_idx in &promise_indices {
        if let crate::vm::interpreter::HeapValue::Promise(p) = &interp.heap[p_idx] {
            match &p.state {
                crate::objects::js_promise::PromiseState::Fulfilled(v) => {
                    interp.resolve_promise(result_idx, v.clone());
                    return Ok(Value::Promise(result_idx));
                }
                crate::objects::js_promise::PromiseState::Rejected(r) => {
                    rejections.push(r.clone());
                }
                crate::objects::js_promise::PromiseState::Pending => {
                    all_rejected = false;
                }
            }
        }
    }

    if all_rejected && !rejections.is_empty() {
        // Create an AggregateError-like object
        let mut props = std::collections::HashMap::new();
        props.insert(
            "message".to_string(),
            Value::String("All promises were rejected".to_string()),
        );
        let errors_idx = interp.heap.len();
        interp.heap.push(crate::vm::interpreter::HeapValue::Array(
            crate::vm::interpreter::JsArray {
                elements: rejections,
            },
        ));
        props.insert("errors".to_string(), Value::Array(errors_idx));
        let err_obj_idx = interp.heap.len();
        interp.heap.push(crate::vm::interpreter::HeapValue::Object(
            crate::vm::interpreter::JsObject {
                properties: props,
                prototype: None,
                extensible: true,
            },
        ));
        interp.reject_promise(result_idx, Value::Object(err_obj_idx));
    }

    Ok(Value::Promise(result_idx))
}

pub(super) fn native_promise_with_resolvers(
    interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let promise_idx = interp.heap.len();
    interp.heap.push(crate::vm::interpreter::HeapValue::Promise(
        crate::objects::js_promise::JsPromise::new(),
    ));

    let resolve_fn = interp.create_resolve_fn(promise_idx);
    let reject_fn = interp.create_reject_fn(promise_idx);

    let mut props = std::collections::HashMap::new();
    props.insert("promise".to_string(), Value::Promise(promise_idx));
    props.insert("resolve".to_string(), resolve_fn);
    props.insert("reject".to_string(), reject_fn);
    let obj_idx = interp.heap.len();
    interp.heap.push(crate::vm::interpreter::HeapValue::Object(
        crate::vm::interpreter::JsObject {
            properties: props,
            prototype: None,
            extensible: true,
        },
    ));

    Ok(Value::Object(obj_idx))
}
