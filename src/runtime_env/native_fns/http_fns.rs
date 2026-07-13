use crate::errors::{Error, Result};
use crate::objects::Value;
use crate::props;
use crate::runtime_env::native_fns::constants as c;
use crate::vm::interpreter::{HeapValue, Interpreter, JsObject};

use super::helpers::{to_f64, to_string_value};
use rustc_hash::FxHashMap;
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

    // EventEmitter state so express's `server.once('error', done)` works.
    let listeners_idx = interp
        .gc
        .allocate(&mut interp.heap, HeapValue::Object(JsObject::new()));

    let props = props! {
        "__handler" => handler,
        "__closed" => Value::Boolean(false),
        "__port" => Value::Integer(0),
        "_listeners" => Value::Object(listeners_idx),
        "listen" => Value::NativeFunction(c::HTTP_SERVER_LISTEN),
        "close" => Value::NativeFunction(c::HTTP_SERVER_CLOSE),
        "setTimeout" => Value::NativeFunction(c::HTTP_SERVER_SET_TIMEOUT),
        // Minimal EventEmitter surface used by express/Node http.Server
        "on" => Value::NativeFunction(c::EVENT_EMITTER_ON),
        "addListener" => Value::NativeFunction(c::EVENT_EMITTER_ON),
        "once" => Value::NativeFunction(c::EVENT_EMITTER_ONCE),
        "emit" => Value::NativeFunction(c::EVENT_EMITTER_EMIT),
        "off" => Value::NativeFunction(c::EVENT_EMITTER_OFF),
        "removeListener" => Value::NativeFunction(c::EVENT_EMITTER_OFF),
        "removeAllListeners" => Value::NativeFunction(c::EVENT_EMITTER_REMOVE_ALL_LISTENERS),
    };

    let idx = interp.heap.len();
    interp.heap.push(HeapValue::Object(JsObject {
        properties: props,
        prototype: None,
        extensible: true,
    }));
    Ok(Value::Object(idx))
}

// server.setTimeout(ms) — no-op for this single-threaded runtime, but
// fastify configures server timeouts so the method must exist.
pub(super) fn native_http_server_set_timeout(
    _interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    Ok(Value::Undefined)
}
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
                .unwrap_or_else(|| Value::string(""))
        } else {
            Value::string("")
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
        // Store response headers when provided as the second argument.
        if let Some(Value::Object(hdr_idx)) = args.get(1) {
            if let HeapValue::Object(res_obj) = &mut interp.heap[*obj_idx] {
                res_obj
                    .properties
                    .insert("__headers".into(), Value::Object(*hdr_idx));
            }
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
                        Some(s.to_string())
                    } else {
                        None
                    }
                })
                .unwrap_or_default();
            obj.properties.insert(
                "__body".into(),
                Value::from_string(prev.to_string() + &chunk),
            );
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
                            Some(s.to_string())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_default();
                obj.properties.insert(
                    "__body".into(),
                    Value::from_string(prev.to_string() + &chunk),
                );
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
// res.setHeader(name, value) / res.getHeader(name) / res.removeHeader(name)
//
// Express relies on these (via the http.ServerResponse.prototype chain) to set
// Content-Type / Content-Length etc. Headers are kept in the `__headers`
// sub-object that `handle_one_request` already forwards to write_response.
// lookups are case-insensitive, matching Node's behaviour.
// ============================================================
fn res_header_map_idx(interp: &mut Interpreter, this: &Value) -> Option<usize> {
    let obj_idx = match this {
        Value::Object(i) => *i,
        _ => return None,
    };
    if let HeapValue::Object(obj) = &interp.heap[obj_idx] {
        if let Some(Value::Object(h)) = obj.properties.get("__headers") {
            return Some(*h);
        }
    }
    // Lazily create the __headers object on first use.
    let h_idx = interp
        .gc
        .allocate(&mut interp.heap, HeapValue::Object(JsObject::new()));
    if let HeapValue::Object(obj) = &mut interp.heap[obj_idx] {
        obj.properties
            .insert("__headers".into(), Value::Object(h_idx));
    }
    Some(h_idx)
}

fn header_key(name: &str) -> String {
    name.to_ascii_lowercase()
}

pub(super) fn native_http_res_set_header(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let h_idx = res_header_map_idx(interp, this).unwrap_or(0);
    let name = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    let value = args
        .get(1)
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    if let HeapValue::Object(h) = &mut interp.heap[h_idx] {
        h.properties
            .insert(header_key(&name), Value::from_string(value));
    }
    Ok(Value::Undefined)
}

pub(super) fn native_http_res_get_header(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let obj_idx = match this {
        Value::Object(i) => *i,
        _ => return Ok(Value::Undefined),
    };
    let name = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    let key = header_key(&name);
    if let HeapValue::Object(obj) = &interp.heap[obj_idx] {
        if let Some(Value::Object(h)) = obj.properties.get("__headers") {
            if let HeapValue::Object(hobj) = &interp.heap[*h] {
                if let Some(v) = hobj.properties.get(&key) {
                    return Ok(v.clone());
                }
            }
        }
    }
    Ok(Value::Undefined)
}

pub(super) fn native_http_res_remove_header(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let obj_idx = match this {
        Value::Object(i) => *i,
        _ => return Ok(Value::Undefined),
    };
    let name = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    let key = header_key(&name);
    if let HeapValue::Object(obj) = &mut interp.heap[obj_idx] {
        if let Some(Value::Object(h)) = obj.properties.get("__headers").cloned() {
            if let HeapValue::Object(hobj) = &mut interp.heap[h] {
                hobj.properties.remove(&key);
            }
        }
    }
    Ok(Value::Undefined)
}

//
// Non-blocking: binds a TCP listener, fires `readyCallback`,
// registers an HttpEventSource, and returns immediately.
// The event loop (TailsRuntime::run_event_loop) will poll the
// listener and dispatch requests to the handler callback.
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

    // Parse optional 3rd-arg options.
    let mut max_connections: i64 = -1; // -1 = unlimited
    if let Some(Value::Object(opt_idx)) = args.get(2) {
        if let HeapValue::Object(opt) = &interp.heap[*opt_idx] {
            if let Some(v) = opt.properties.get("maxConnections") {
                max_connections = to_f64(v) as i64;
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

    // Register the listener as an event source so the event loop will poll it.
    interp.pending_event_sources.push(Box::new(HttpEventSource {
        server_idx,
        listener,
        max_connections,
        handled: 0,
    }));

    // Match Node: fire the listening callback asynchronously (next macrotask)
    // so code after `server.listen(...)` runs before the ready callback.
    // e.g. `app.listen(port, () => console.log("listening")); console.log(app);`
    // should print `app` first, then "listening".
    if let Some(cb) = ready_cb {
        if !matches!(cb, Value::Undefined | Value::Null) {
            interp.async_runtime.enqueue_macrotask(cb, 0.0);
        }
    }

    Ok(Value::Undefined)
}

// ── EventSource implementation for HTTP servers ──────────────────────────

/// Wraps a non-blocking TCP listener bound to an HTTP server object.
/// Registered during `server.listen()` and polled by the event loop.
struct HttpEventSource {
    server_idx: usize,
    listener: std::net::TcpListener,
    max_connections: i64, // -1 = unlimited
    handled: i64,
}

impl crate::vm::EventSource for HttpEventSource {
    fn is_active(&self) -> bool {
        if self.max_connections >= 0 && self.handled >= self.max_connections {
            return false;
        }
        true
    }

    fn poll(&mut self, interp: &mut Interpreter) -> Result<()> {
        // Check if the JS server object has been closed.
        let closed = match interp.heap.get(self.server_idx) {
            Some(HeapValue::Object(obj)) => {
                matches!(obj.properties.get("__closed"), Some(Value::Boolean(true)))
            }
            _ => true,
        };
        if closed {
            return Ok(());
        }

        match self.listener.accept() {
            Ok((mut stream, _)) => {
                match tails_http::read_request(&mut stream) {
                    Ok(req) => {
                        handle_one_request(interp, self.server_idx, req, stream)?;
                        self.handled += 1;
                    }
                    Err(_) => { /* ignore malformed requests */ }
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // No pending connections — normal for non-blocking listener.
            }
            Err(_) => {
                // Permanent error on this listener. Nothing to do; next
                // is_active() could return false if we tracked it, but
                // keeping it simple: the server object can be closed from JS.
            }
        }
        Ok(())
    }
}

/// Build req/res JS objects, invoke the request handler, then write the
/// HTTP response built from the `res` object's `__status`/`__body`.
fn handle_one_request(
    interp: &mut Interpreter,
    server_idx: usize,
    req: tails_http::HttpRequest,
    mut stream: std::net::TcpStream,
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
    let mut hdr_props = FxHashMap::default();
    for (k, v) in &req.headers {
        hdr_props.insert(k.clone(), Value::from_string(v.clone()));
    }
    let hdr_idx = interp.heap.len();
    interp.heap.push(HeapValue::Object(JsObject {
        properties: hdr_props.into(),
        prototype: None,
        extensible: true,
    }));

    let req_props = props! {
        "method" => Value::from_string(req.method),
        "url" => Value::from_string(req.path),
        "body" => Value::from_string(req.body.clone()),
        "__body" => Value::from_string(req.body),
        "headers" => Value::Object(hdr_idx),
        "on" => Value::NativeFunction(c::HTTP_REQ_ON),
    };
    let req_idx = interp.heap.len();
    interp.heap.push(HeapValue::Object(JsObject {
        properties: req_props,
        prototype: None,
        extensible: true,
    }));
    let req_val = Value::Object(req_idx);

    // --- res object ---
    let res_props = props! {
        "statusCode" => Value::Integer(200),
        "__status" => Value::Integer(200),
        "__body" => Value::string(""),
        "__ended" => Value::Boolean(false),
        "__headers" => Value::Object({
            let h = interp.heap.len();
            interp.heap.push(HeapValue::Object(JsObject::new()));
            h
        }),
        "writeHead" => Value::NativeFunction(c::HTTP_RES_WRITE_HEAD),
        "write" => Value::NativeFunction(c::HTTP_RES_WRITE),
        "end" => Value::NativeFunction(c::HTTP_RES_END),
        "setHeader" => Value::NativeFunction(c::HTTP_RES_SET_HEADER),
        "getHeader" => Value::NativeFunction(c::HTTP_RES_GET_HEADER),
        "removeHeader" => Value::NativeFunction(c::HTTP_RES_REMOVE_HEADER),
    };
    let res_idx = interp.heap.len();
    interp.heap.push(HeapValue::Object(JsObject {
        properties: res_props,
        prototype: None,
        extensible: true,
    }));
    let res_val = Value::Object(res_idx);

    // --- invoke handler(req, res) ---
    let handler_ret = if !matches!(handler, Value::Undefined) {
        Some(interp.call_value(&handler, &Value::Undefined, &[req_val, res_val])?)
    } else {
        None
    };

    // Async handlers (`async (req, res) => { ... await ...; res.end(...) }`)
    // return a Promise. The response must NOT be written until that promise
    // settles — otherwise we'd send an empty body before `await fs.readFile`
    // resolves. We therefore hand the still-open connection to a dedicated
    // event source that flushes the response once the handler calls
    // `res.end` (or the promise settles), while the event loop keeps driving
    // timers, I/O, and the promise chain to completion.
    if let Some(Value::Promise(pidx)) = handler_ret {
        interp.pending_event_sources.push(Box::new(PendingResponse {
            stream,
            res_idx,
            server_idx,
            handler_promise: Some(pidx),
            done: false,
        }));
        return Ok(());
    }

    // Synchronous handler (or a handler that already ended the response):
    // write the response now.
    write_response_from_res(interp, res_idx, &mut stream)
}

/// Read the status/headers/body accumulated on the `res` object and write the
/// HTTP response to `stream`. Shared by synchronous handlers and the
/// [`PendingResponse`] event source used for async handlers.
fn write_response_from_res(
    interp: &mut Interpreter,
    res_idx: usize,
    stream: &mut std::net::TcpStream,
) -> Result<()> {
    let (status, headers, body) = if let HeapValue::Object(obj) = &interp.heap[res_idx] {
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
                    Some(s.to_string())
                } else {
                    None
                }
            })
            .unwrap_or_default();
        // Read response headers set by writeHead().
        let hdrs = if let Some(Value::Object(hidx)) = obj.properties.get("__headers") {
            if let HeapValue::Object(hobj) = &interp.heap[*hidx] {
                hobj.properties
                    .iter()
                    .filter_map(|(k, v)| {
                        if let Value::String(s) = v {
                            Some((k.to_string(), s.to_string()))
                        } else {
                            None
                        }
                    })
                    .collect()
            } else {
                HashMap::new()
            }
        } else {
            HashMap::new()
        };
        (st, hdrs, bd)
    } else {
        (200u16, HashMap::new(), String::new())
    };

    tails_http::write_response(
        stream,
        status,
        tails_http::status_text(status),
        &headers,
        &body,
    )
    .map_err(|e| Error::RuntimeError(format!("http write_response failed: {}", e)))?;
    Ok(())
}

/// Event source that owns the TCP connection for an in-flight async request
/// handler. The `run_event_loop` polls it each tick; once the handler's
/// `res.end` flags the response as ended (or the handler promise settles) it
/// writes the response and retires itself.
struct PendingResponse {
    stream: std::net::TcpStream,
    res_idx: usize,
    server_idx: usize,
    handler_promise: Option<usize>,
    done: bool,
}

impl crate::vm::EventSource for PendingResponse {
    fn is_active(&self) -> bool {
        !self.done
    }

    fn poll(&mut self, interp: &mut Interpreter) -> Result<()> {
        // Stop if the server was closed from JS.
        let closed = match interp.heap.get(self.server_idx) {
            Some(HeapValue::Object(obj)) => {
                matches!(obj.properties.get("__closed"), Some(Value::Boolean(true)))
            }
            _ => true,
        };
        if closed {
            self.done = true;
            return Ok(());
        }

        // Flush once the handler has ended the response, or once its promise
        // has settled (so a handler that forgets to call `res.end` can't hang
        // the connection forever).
        let ended = match interp.heap.get(self.res_idx) {
            Some(HeapValue::Object(obj)) => {
                matches!(obj.properties.get("__ended"), Some(Value::Boolean(true)))
            }
            _ => true,
        };
        let settled = match self.handler_promise {
            Some(pidx) => matches!(
                interp.heap.get(pidx),
                Some(HeapValue::Promise(p)) if matches!(p.state,
                    crate::objects::js_promise::PromiseState::Fulfilled(_)
                        | crate::objects::js_promise::PromiseState::Rejected(_))
            ),
            None => false,
        };

        if ended || settled {
            let _ = write_response_from_res(interp, self.res_idx, &mut self.stream);
            self.done = true;
        }
        Ok(())
    }
}
