use crate::errors::{Error, Result};
use crate::objects::Value;
use crate::runtime_env::native_fns::constants as c;
use crate::vm::interpreter::{HeapValue, Interpreter, JsObject};

use super::helpers::{to_f64, to_i64, to_string_value};
use std::collections::HashMap;

// ============================================================
// http.createServer(requestHandler) -> server object
// ============================================================
pub(super) fn native_http_create_server(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let handler = args.first().cloned().unwrap_or(Value::Undefined);

    let mut props = HashMap::new();
    props.insert("__handler".into(), handler);
    props.insert("__closed".into(), Value::Boolean(false));
    props.insert("__port".into(), Value::Integer(0));
    // Methods
    props.insert(
        "listen".into(),
        Value::NativeFunction(c::HTTP_SERVER_LISTEN),
    );
    props.insert("close".into(), Value::NativeFunction(c::HTTP_SERVER_CLOSE));

    let idx = interp.heap.len();
    interp.heap.push(HeapValue::Object(JsObject {
        properties: props,
        prototype: None,
        extensible: true,
    }));
    Ok(Value::Object(idx))
}

// server.close() -> marks the server closed so the accept loop exits.
pub(super) fn native_http_server_close(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    if let Value::Object(obj_idx) = this {
        if let HeapValue::Object(obj) = &mut interp.heap[*obj_idx] {
            obj.properties
                .insert("__closed".into(), Value::Boolean(true));
        }
    }
    Ok(Value::Undefined)
}

// ============================================================
// req.on(event, callback) — fires synchronously (single-threaded runtime)
// ============================================================
pub(super) fn native_http_req_on(
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
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    let cb = args.get(1).cloned().unwrap_or(Value::Undefined);

    if event == "data" {
        // Fire the callback immediately with the (already-collected) body.
        let body_val = if let HeapValue::Object(obj) = &interp.heap[obj_idx] {
            obj.properties
                .get("__body")
                .cloned()
                .unwrap_or_else(|| Value::String(String::new()))
        } else {
            Value::String(String::new())
        };
        let _ = interp.call_value(&cb, &Value::Undefined, &[body_val]);
    } else if event == "end" {
        let _ = interp.call_value(&cb, &Value::Undefined, &[]);
    }
    Ok(Value::Undefined)
}

// ============================================================
// res.writeHead(status) / res.write(chunk) / res.end([body])
// ============================================================
pub(super) fn native_http_res_write_head(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    if let Value::Object(obj_idx) = this {
        let status = args.first().map(to_f64).unwrap_or(200.0) as i64;
        if let HeapValue::Object(obj) = &mut interp.heap[*obj_idx] {
            obj.properties
                .insert("statusCode".into(), Value::Integer(status));
            obj.properties
                .insert("__status".into(), Value::Integer(status));
        }
    }
    Ok(Value::Undefined)
}

pub(super) fn native_http_res_write(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    if let Value::Object(obj_idx) = this {
        let chunk = args
            .first()
            .map(|v| to_string_value(interp, v))
            .unwrap_or_default();
        if let HeapValue::Object(obj) = &mut interp.heap[*obj_idx] {
            let prev = obj
                .properties
                .get("__body")
                .and_then(|v| {
                    if let Value::String(s) = v {
                        Some(s.clone())
                    } else {
                        None
                    }
                })
                .unwrap_or_default();
            obj.properties
                .insert("__body".into(), Value::String(prev + &chunk));
        }
    }
    Ok(Value::Undefined)
}

pub(super) fn native_http_res_end(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    if let Value::Object(obj_idx) = this {
        if let Some(chunk) = args.first() {
            let chunk = to_string_value(interp, chunk);
            if let HeapValue::Object(obj) = &mut interp.heap[*obj_idx] {
                let prev = obj
                    .properties
                    .get("__body")
                    .and_then(|v| {
                        if let Value::String(s) = v {
                            Some(s.clone())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_default();
                obj.properties
                    .insert("__body".into(), Value::String(prev + &chunk));
            }
        }
        if let HeapValue::Object(obj) = &mut interp.heap[*obj_idx] {
            obj.properties
                .insert("__ended".into(), Value::Boolean(true));
        }
    }
    Ok(Value::Undefined)
}

// ============================================================
// server.listen(port[, readyCallback[, options]])
//
// Binds a TCP listener, invokes `readyCallback`, then runs a bounded accept
// loop. Each connection is handled synchronously: parse request -> build
// req/res -> call the request handler -> write response.
//
// options (3rd arg):
//   maxConnections: number — stop after handling this many (default: unlimited)
//   timeoutMs:      number — stop after this many ms wall-clock (default: 30000)
// ============================================================
pub(super) fn native_http_server_listen(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let server_idx = match this {
        Value::Object(i) => *i,
        _ => return Ok(Value::Undefined),
    };

    let port = args.first().map(to_f64).unwrap_or(0.0) as u16;
    let ready_cb = args.get(1).cloned();

    let mut max_conn: i64 = -1; // -1 = unlimited
    let mut timeout_ms: u64 = 30_000;
    if let Some(Value::Object(opt_idx)) = args.get(2) {
        if let HeapValue::Object(opt) = &interp.heap[*opt_idx] {
            if let Some(v) = opt.properties.get("maxConnections") {
                max_conn = to_i64(v);
            }
            if let Some(v) = opt.properties.get("timeoutMs") {
                timeout_ms = to_f64(v) as u64;
            }
        }
    }

    let listener = tails_http::bind(port)
        .map_err(|e| Error::RuntimeError(format!("http listen failed on port {}: {}", port, e)))?;
    let local_port = listener.local_addr().map(|a| a.port()).unwrap_or(port);

    if let HeapValue::Object(obj) = &mut interp.heap[server_idx] {
        obj.properties
            .insert("__port".into(), Value::Integer(local_port as i64));
    }

    // Invoke the "listening" callback synchronously.
    if let Some(cb) = ready_cb {
        let _ = interp.call_value(&cb, &Value::Undefined, &[]);
    }

    let start = std::time::Instant::now();
    let poll = std::time::Duration::from_millis(10);
    let mut handled: i64 = 0;

    loop {
        let closed = if let HeapValue::Object(obj) = &interp.heap[server_idx] {
            matches!(obj.properties.get("__closed"), Some(Value::Boolean(true)))
        } else {
            true
        };
        if closed {
            break;
        }
        if start.elapsed().as_millis() as u64 > timeout_ms {
            break;
        }
        if max_conn >= 0 && handled >= max_conn {
            break;
        }

        match listener.accept() {
            Ok((mut stream, _)) => {
                match tails_http::read_request(&mut stream) {
                    Ok(req) => handle_one_request(interp, server_idx, req, &mut stream)?,
                    Err(_) => { /* ignore malformed requests */ }
                }
                handled += 1;
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(poll);
            }
            Err(_) => break,
        }
    }

    Ok(Value::Undefined)
}

/// Build req/res JS objects, invoke the request handler, then write the
/// HTTP response built from the `res` object's `__status`/`__body`.
fn handle_one_request(
    interp: &mut Interpreter,
    server_idx: usize,
    req: tails_http::HttpRequest,
    stream: &mut std::net::TcpStream,
) -> Result<()> {
    // Retrieve the handler (clone first to release the immutable borrow).
    let handler = if let HeapValue::Object(obj) = &interp.heap[server_idx] {
        obj.properties
            .get("__handler")
            .cloned()
            .unwrap_or(Value::Undefined)
    } else {
        Value::Undefined
    };

    // --- req object ---
    let mut hdr_props = HashMap::new();
    for (k, v) in &req.headers {
        hdr_props.insert(k.clone(), Value::String(v.clone()));
    }
    let hdr_idx = interp.heap.len();
    interp.heap.push(HeapValue::Object(JsObject {
        properties: hdr_props,
        prototype: None,
        extensible: true,
    }));

    let mut req_props = HashMap::new();
    req_props.insert("method".into(), Value::String(req.method));
    req_props.insert("url".into(), Value::String(req.path));
    req_props.insert("body".into(), Value::String(req.body.clone()));
    req_props.insert("__body".into(), Value::String(req.body));
    req_props.insert("headers".into(), Value::Object(hdr_idx));
    req_props.insert("on".into(), Value::NativeFunction(c::HTTP_REQ_ON));
    let req_idx = interp.heap.len();
    interp.heap.push(HeapValue::Object(JsObject {
        properties: req_props,
        prototype: None,
        extensible: true,
    }));
    let req_val = Value::Object(req_idx);

    // --- res object ---
    let mut res_props = HashMap::new();
    res_props.insert("statusCode".into(), Value::Integer(200));
    res_props.insert("__status".into(), Value::Integer(200));
    res_props.insert("__body".into(), Value::String(String::new()));
    res_props.insert("__ended".into(), Value::Boolean(false));
    res_props.insert(
        "writeHead".into(),
        Value::NativeFunction(c::HTTP_RES_WRITE_HEAD),
    );
    res_props.insert("write".into(), Value::NativeFunction(c::HTTP_RES_WRITE));
    res_props.insert("end".into(), Value::NativeFunction(c::HTTP_RES_END));
    let res_idx = interp.heap.len();
    interp.heap.push(HeapValue::Object(JsObject {
        properties: res_props,
        prototype: None,
        extensible: true,
    }));
    let res_val = Value::Object(res_idx);

    // --- invoke handler(req, res) ---
    if !matches!(handler, Value::Undefined) {
        let _ = interp.call_value(&handler, &Value::Undefined, &[req_val, res_val]);
    }

    // --- read the response out of `res` (no allocation between call & read) ---
    let (status, body) = if let HeapValue::Object(obj) = &interp.heap[res_idx] {
        let st = obj
            .properties
            .get("__status")
            .map(|v| match v {
                Value::Integer(n) => *n as u16,
                Value::Float(n) => *n as u16,
                _ => 200,
            })
            .unwrap_or(200);
        let bd = obj
            .properties
            .get("__body")
            .and_then(|v| {
                if let Value::String(s) = v {
                    Some(s.clone())
                } else {
                    None
                }
            })
            .unwrap_or_default();
        (st, bd)
    } else {
        (200u16, String::new())
    };

    let empty_headers = HashMap::new();
    tails_http::write_response(
        stream,
        status,
        tails_http::status_text(status),
        &empty_headers,
        &body,
    )
    .map_err(|e| Error::RuntimeError(format!("http write_response failed: {}", e)))?;
    Ok(())
}
