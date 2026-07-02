use crate::errors::{Error, Result};
use crate::objects::Value;
use crate::runtime_env::native_fns::constants as c;
use crate::vm::interpreter::{HeapValue, Interpreter, JsObject};

use super::helpers::to_f64;
use std::cell::RefCell;
use rustc_hash::FxHashMap;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Thread-local registry for live TCP streams.
// Each socket object stores a `__stream_id` (Value::Integer) that indexes into
// this map so that write / end / on can reach the underlying `TcpStream`.
// ---------------------------------------------------------------------------
thread_local! {
    static STREAM_REGISTRY: RefCell<HashMap<i64, std::net::TcpStream>> =
        RefCell::new(HashMap::new());
    static NEXT_STREAM_ID: std::cell::Cell<i64> = const { std::cell::Cell::new(1) };
}

fn alloc_stream_id(stream: std::net::TcpStream) -> i64 {
    NEXT_STREAM_ID.with(|id| {
        let n = id.get();
        id.set(n + 1);
        STREAM_REGISTRY.with(|reg| reg.borrow_mut().insert(n, stream));
        n
    })
}

fn with_stream_mut<R>(id: i64, f: impl FnOnce(&mut std::net::TcpStream) -> R) -> Option<R> {
    STREAM_REGISTRY.with(|reg| reg.borrow_mut().get_mut(&id).map(f))
}

fn remove_stream(id: i64) -> Option<std::net::TcpStream> {
    STREAM_REGISTRY.with(|reg| reg.borrow_mut().remove(&id))
}

fn get_stream_id(interp: &Interpreter, obj_idx: usize) -> i64 {
    if let HeapValue::Object(obj) = &interp.heap[obj_idx] {
        obj.properties
            .get("__stream_id")
            .and_then(|v| {
                if let Value::Integer(id) = v {
                    Some(*id)
                } else {
                    None
                }
            })
            .unwrap_or(-1)
    } else {
        -1
    }
}

// ============================================================
// net.createConnection(port[, host][, connectListener])
// ============================================================
pub(super) fn native_net_create_connection(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let port = args.first().map(to_f64).unwrap_or(0.0) as u16;
    let host = args
        .get(1)
        .and_then(|v| {
            if let Value::String(s) = v {
                Some(s.clone())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "127.0.0.1".to_string());

    // The connect listener can be 2nd or 3rd arg depending on whether host was given.
    let connect_cb = if args.len() >= 3 {
        args.get(2).cloned()
    } else if args.len() == 2 {
        match args.get(1) {
            Some(Value::Function(_)) | Some(Value::NativeFunction(_)) => args.get(1).cloned(),
            _ => None,
        }
    } else {
        None
    };

    // Connect synchronously.
    let stream = tails_net::connect(&host, port).map_err(|e| {
        Error::RuntimeError(format!(
            "net.createConnection failed ({}:{}): {}",
            host, port, e
        ))
    })?;

    let stream_id = alloc_stream_id(stream);

    // Build the socket JS object.
    let mut props = FxHashMap::default();
    props.insert("__stream_id".into(), Value::Integer(stream_id));
    props.insert("readyState".into(), Value::String("open".into()));
    props.insert("write".into(), Value::NativeFunction(c::NET_SOCKET_WRITE));
    props.insert("end".into(), Value::NativeFunction(c::NET_SOCKET_END));
    props.insert("on".into(), Value::NativeFunction(c::NET_SOCKET_ON));

    let sock_idx = interp.heap.len();
    interp.heap.push(HeapValue::Object(JsObject {
        properties: props,
        prototype: None,
        extensible: true,
    }));
    let sock_val = Value::Object(sock_idx);

    // Invoke the connect listener synchronously.
    if let Some(cb) = connect_cb {
        let _ = interp.call_value(&cb, &Value::Undefined, std::slice::from_ref(&sock_val));
    }

    // After the callback (which may have called write + end), attempt to read
    // any data the server sent back and fire 'data' event listeners.
    // Skip the read poll entirely if no 'data' listeners were registered —
    // this avoids blocking when the caller doesn't need the response
    // (e.g. fire-and-forget benchmark patterns).
    let has_data_listeners = if let HeapValue::Object(obj) = &interp.heap[sock_idx] {
        obj.properties.contains_key("__listeners_data")
    } else {
        false
    };

    let sid = get_stream_id(interp, sock_idx);
    if has_data_listeners {
        let mut collected = Vec::new();
        for _ in 0..100 {
            if let Some(result) = with_stream_mut(sid, tails_net::read_available) {
                match result {
                    Ok(data) if !data.is_empty() => {
                        collected.extend_from_slice(&data);
                        break;
                    }
                    Ok(_) => std::thread::sleep(std::time::Duration::from_millis(2)),
                    Err(_) => break,
                }
            } else {
                break;
            }
        }

        if !collected.is_empty() {
            let data_val = Value::String(String::from_utf8_lossy(&collected).into_owned());
            fire_listeners(interp, sock_idx, "data", &[data_val]);
        }
    }

    // Clean up.
    remove_stream(sid);
    if let HeapValue::Object(obj) = &mut interp.heap[sock_idx] {
        obj.properties
            .insert("readyState".into(), Value::String("closed".into()));
    }
    fire_listeners(interp, sock_idx, "close", &[]);

    Ok(sock_val)
}

// ============================================================
// socket.write(data)
// ============================================================
pub(super) fn native_net_socket_write(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let obj_idx = match this {
        Value::Object(i) => *i,
        _ => return Ok(Value::Boolean(false)),
    };
    let sid = get_stream_id(interp, obj_idx);
    if sid < 0 {
        return Ok(Value::Boolean(false));
    }

    let data = args
        .first()
        .and_then(|v| {
            if let Value::String(s) = v {
                Some(s.as_bytes().to_vec())
            } else {
                None
            }
        })
        .unwrap_or_default();

    let ok = with_stream_mut(sid, |stream| tails_net::write(stream, &data))
        .map(|r| r.is_ok())
        .unwrap_or(false);

    Ok(Value::Boolean(ok))
}

// ============================================================
// socket.end([data])
// ============================================================
pub(super) fn native_net_socket_end(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let obj_idx = match this {
        Value::Object(i) => *i,
        _ => return Ok(Value::Undefined),
    };
    let sid = get_stream_id(interp, obj_idx);
    if sid < 0 {
        return Ok(Value::Undefined);
    }

    // Write trailing data before shutting down.
    if let Some(Value::String(s)) = args.first() {
        let _ = with_stream_mut(sid, |stream| tails_net::write(stream, s.as_bytes()));
    }
    // Shut down the write half (sends FIN).
    let _ = with_stream_mut(sid, |stream| tails_net::shutdown(stream));

    Ok(Value::Undefined)
}

// ============================================================
// socket.on(event, listener)
// ============================================================
pub(super) fn native_net_socket_on(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let obj_idx = match this {
        Value::Object(i) => *i,
        _ => return Ok(Value::Undefined),
    };
    let event = args
        .first()
        .and_then(|v| {
            if let Value::String(s) = v {
                Some(s.clone())
            } else {
                None
            }
        })
        .unwrap_or_default();
    let cb = args.get(1).cloned().unwrap_or(Value::Undefined);
    let key = format!("__listeners_{}", event);

    let has = if let HeapValue::Object(obj) = &interp.heap[obj_idx] {
        obj.properties.contains_key(&key)
    } else {
        return Ok(Value::Undefined);
    };

    let arr_idx = if !has {
        let idx = interp.heap.len();
        interp
            .heap
            .push(HeapValue::Array(crate::vm::interpreter::JsArray {
                elements: Vec::new(),
            }));
        if let HeapValue::Object(obj) = &mut interp.heap[obj_idx] {
            obj.properties.insert(key, Value::Array(idx));
        }
        idx
    } else {
        if let HeapValue::Object(obj) = &interp.heap[obj_idx] {
            if let Some(Value::Array(idx)) = obj.properties.get(&key) {
                *idx
            } else {
                return Ok(Value::Undefined);
            }
        } else {
            return Ok(Value::Undefined);
        }
    };

    if let HeapValue::Array(arr) = &mut interp.heap[arr_idx] {
        arr.elements.push(cb);
    }
    Ok(Value::Undefined)
}

// ============================================================
// helper: fire stored event listeners
// ============================================================
fn fire_listeners(interp: &mut Interpreter, obj_idx: usize, event: &str, call_args: &[Value]) {
    let key = format!("__listeners_{}", event);
    let listeners: Vec<Value> = if let HeapValue::Object(obj) = &interp.heap[obj_idx] {
        obj.properties
            .get(&key)
            .and_then(|v| {
                if let Value::Array(arr_idx) = v {
                    if let HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                        Some(arr.elements.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    for cb in &listeners {
        let _ = interp.call_value(cb, &Value::Undefined, call_args);
    }
}
