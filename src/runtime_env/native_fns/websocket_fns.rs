use crate::errors::{Error, Result};
use crate::objects::Value;
use crate::props;
use crate::runtime_env::native_fns::constants as c;
use crate::vm::interpreter::{HeapValue, Interpreter, JsObject};

use super::helpers::to_string_value;

pub(super) fn native_websocket_constructor(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let url = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();

    if url.is_empty() {
        return Err(Error::TypeError("WebSocket requires a URL".into()));
    }

    let props = props! {
        "url" => Value::String(url),
        "readyState" => Value::Integer(0),
        "bufferedAmount" => Value::Integer(0),
        "binaryType" => Value::String("blob".into()),
        "protocol" => Value::String(String::new()),
        "extensions" => Value::String(String::new()),
        "send" => Value::NativeFunction(c::URL_TO_JSON),
        "close" => Value::NativeFunction(c::HEADERS_CONSTRUCTOR),
        "addEventListener" => Value::NativeFunction(c::HEADERS_APPEND),
        "removeEventListener" => Value::NativeFunction(c::HEADERS_GET),
    };

    let ws_idx = interp.heap.len();
    interp.heap.push(HeapValue::Object(JsObject {
        properties: props,
        prototype: None,
        extensible: true,
    }));

    Ok(Value::Object(ws_idx))
}

pub(super) fn native_websocket_send(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    if let Value::Object(obj_idx) = _this {
        let message = args
            .first()
            .map(|v| to_string_value(interp, v))
            .unwrap_or_default();

        if let HeapValue::Object(obj) = &mut interp.heap[*obj_idx] {
            let msg_len = message.len();
            obj.properties
                .insert("__pendingMessage".into(), Value::String(message));

            let buffered = obj
                .properties
                .get("bufferedAmount")
                .and_then(|v| {
                    if let Value::Integer(i) = v {
                        Some(*i)
                    } else {
                        None
                    }
                })
                .unwrap_or(0);
            obj.properties.insert(
                "bufferedAmount".into(),
                Value::Integer(buffered + msg_len as i64),
            );
        }
    }
    Ok(Value::Undefined)
}

pub(super) fn native_websocket_close(
    interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    if let Value::Object(obj_idx) = _this {
        if let HeapValue::Object(obj) = &mut interp.heap[*obj_idx] {
            obj.properties
                .insert("readyState".into(), Value::Integer(3)); // CLOSED
        }
    }
    Ok(Value::Undefined)
}

pub(super) fn native_websocket_add_event_listener(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    if let Value::Object(obj_idx) = _this {
        let event_type = args
            .first()
            .map(|v| to_string_value(interp, v))
            .unwrap_or_default();
        let callback = args.get(1).cloned().unwrap_or(Value::Undefined);
        let listeners_key = format!("__listeners_{}", event_type);

        let has_listeners = if let HeapValue::Object(obj) = &interp.heap[*obj_idx] {
            obj.properties.contains_key(&listeners_key)
        } else {
            return Ok(Value::Undefined);
        };

        let arr_idx = if !has_listeners {
            let new_arr_idx = interp.heap.len();
            interp
                .heap
                .push(HeapValue::Array(crate::vm::interpreter::JsArray {
                    elements: Vec::new(),
                }));
            new_arr_idx
        } else {
            if let HeapValue::Object(obj) = &interp.heap[*obj_idx] {
                if let Some(Value::Array(arr_idx)) = obj.properties.get(&listeners_key) {
                    *arr_idx
                } else {
                    return Ok(Value::Undefined);
                }
            } else {
                return Ok(Value::Undefined);
            }
        };

        if !has_listeners {
            if let HeapValue::Object(obj) = &mut interp.heap[*obj_idx] {
                obj.properties.insert(listeners_key, Value::Array(arr_idx));
            }
        }

        if let HeapValue::Array(arr) = &mut interp.heap[arr_idx] {
            arr.elements.push(callback);
        }
    }
    Ok(Value::Undefined)
}

pub(super) fn native_websocket_remove_event_listener(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    if let Value::Object(obj_idx) = _this {
        let event_type = args
            .first()
            .map(|v| to_string_value(interp, v))
            .unwrap_or_default();
        let callback = args.get(1).cloned().unwrap_or(Value::Undefined);
        let listeners_key = format!("__listeners_{}", event_type);

        let arr_idx = if let HeapValue::Object(obj) = &interp.heap[*obj_idx] {
            if let Some(Value::Array(arr_idx)) = obj.properties.get(&listeners_key) {
                Some(*arr_idx)
            } else {
                None
            }
        } else {
            None
        };

        if let Some(arr_idx) = arr_idx {
            if let HeapValue::Array(arr) = &mut interp.heap[arr_idx] {
                // Use PartialEq for comparison instead of debug format
                arr.elements.retain(|v| *v != callback);
            }
        }
    }
    Ok(Value::Undefined)
}
