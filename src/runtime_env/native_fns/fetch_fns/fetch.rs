use crate::errors::Result;
use crate::objects::js_promise::JsPromise;
use crate::objects::Value;
use crate::runtime_env::native_fns::helpers::to_string_value;
use crate::vm::interpreter::{HeapValue, Interpreter};
use crate::vm::EventSource;

use super::headers::parse_headers;
use super::response::build_response;

use std::sync::mpsc;
use std::thread;

pub(crate) fn native_fetch(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let (url, method, headers_map, body) = if let Some(Value::Object(obj_idx)) = args.first() {
        if let HeapValue::Object(obj) = &interp.heap[*obj_idx] {
            if obj.properties.contains_key("__is_request") {
                let url = obj
                    .properties
                    .get("url")
                    .map(|v| to_string_value(interp, v))
                    .unwrap_or_default();
                let method = obj
                    .properties
                    .get("method")
                    .map(|v| to_string_value(interp, v))
                    .unwrap_or_else(|| "GET".to_string());
                let headers_raw = obj
                    .properties
                    .get("__headers")
                    .map(|v| {
                        if let Value::String(s) = v {
                            s.to_string()
                        } else {
                            String::new()
                        }
                    })
                    .unwrap_or_default();
                let headers_map = parse_headers_to_map(&headers_raw);
                let body = obj.properties.get("__body").and_then(|v| {
                    if let Value::String(s) = v {
                        if s.is_empty() {
                            None
                        } else {
                            Some(s.to_string())
                        }
                    } else {
                        None
                    }
                });
                (url, method, headers_map, body)
            } else {
                let url = to_string_value(interp, args.first().unwrap());
                let (method, headers_map, body) = parse_fetch_options(interp, args);
                (url, method, headers_map, body)
            }
        } else {
            let url = to_string_value(interp, args.first().unwrap());
            let (method, headers_map, body) = parse_fetch_options(interp, args);
            (url, method, headers_map, body)
        }
    } else {
        let url = args
            .first()
            .map(|v| to_string_value(interp, v))
            .unwrap_or_default();
        let (method, headers_map, body) = parse_fetch_args(interp, args);
        (url, method, headers_map, body)
    };

    // Create a *pending* promise. We resolve it later (without blocking the
    // interpreter thread) once the background worker delivers the response.
    let promise_idx = interp.heap.len();
    interp.heap.push(HeapValue::Promise(JsPromise::new()));

    // The blocking HTTP I/O runs on a dedicated worker thread so the main
    // thread stays free to run the event loop — serving HTTP requests, firing
    // timers, draining microtasks, … . This is what makes `await fetch(...)`
    // cooperative instead of deadlocking against an in-process server.
    let (tx, rx) = mpsc::channel();
    let worker_url = url.clone();
    let worker_method = method.clone();
    let worker_headers = headers_map.clone();
    let worker_body = body.clone();
    let _ = thread::Builder::new()
        .name("tails-fetch".to_string())
        .spawn(move || {
            let result = do_fetch_blocking(worker_url, worker_method, worker_headers, worker_body);
            let _ = tx.send(result);
        });

    // Register a source the event loop polls; it resolves the promise once the
    // worker's result arrives.
    interp
        .pending_event_sources
        .push(Box::new(FetchEventSource {
            rx,
            promise_idx,
            done: false,
        }));

    Ok(Value::Promise(promise_idx))
}

fn parse_fetch_args(
    interp: &mut Interpreter,
    args: &[Value],
) -> (
    String,
    std::collections::HashMap<String, String>,
    Option<String>,
) {
    let method = "GET".to_string();
    let headers_map = std::collections::HashMap::new();
    let mut body = None;

    if let Some(Value::Object(opts_idx)) = args.get(1) {
        if let HeapValue::Object(obj) = &interp.heap[*opts_idx] {
            let m = obj
                .properties
                .get("method")
                .map(|v| to_string_value(interp, v))
                .unwrap_or_else(|| "GET".to_string());

            let mut hdrs = std::collections::HashMap::new();
            if let Some(Value::Object(hdr_idx)) = obj.properties.get("headers") {
                if let HeapValue::Object(hdr_obj) = &interp.heap[*hdr_idx] {
                    if let Some(Value::String(h)) = hdr_obj.properties.get("__headers") {
                        for (k, v) in parse_headers(h) {
                            hdrs.insert(k, v);
                        }
                    } else {
                        for (k, v) in &hdr_obj.properties {
                            if !k.starts_with('_') {
                                hdrs.insert(k.to_string(), to_string_value(interp, v));
                            }
                        }
                    }
                }
            }

            body = obj
                .properties
                .get("body")
                .map(|v| to_string_value(interp, v));

            return (m, hdrs, body);
        }
    }

    (method, headers_map, body)
}

fn parse_fetch_options(
    interp: &mut Interpreter,
    args: &[Value],
) -> (
    String,
    std::collections::HashMap<String, String>,
    Option<String>,
) {
    let method = "GET".to_string();
    let headers_map = std::collections::HashMap::new();
    let body = None;

    if let Some(Value::Object(opts_idx)) = args.get(1) {
        if let HeapValue::Object(obj) = &interp.heap[*opts_idx] {
            let m = obj
                .properties
                .get("method")
                .map(|v| to_string_value(interp, v))
                .unwrap_or_else(|| "GET".to_string());

            let mut hdrs = std::collections::HashMap::new();
            if let Some(Value::Object(hdr_idx)) = obj.properties.get("headers") {
                if let HeapValue::Object(hdr_obj) = &interp.heap[*hdr_idx] {
                    for (k, v) in &hdr_obj.properties {
                        if !k.starts_with('_') {
                            hdrs.insert(k.to_string(), to_string_value(interp, v));
                        }
                    }
                }
            }

            let b = obj
                .properties
                .get("body")
                .map(|v| to_string_value(interp, v));

            return (m, hdrs, b);
        }
    }

    (method, headers_map, body)
}

fn parse_headers_to_map(raw: &str) -> std::collections::HashMap<String, String> {
    parse_headers(raw).into_iter().collect()
}

/// Raw, interpreter-independent result of a completed (blocking) HTTP request.
///
/// Produced on the worker thread and shipped to the main thread via a channel;
/// the [`FetchEventSource`] turns it into the JS `Response` value.
struct RawFetchResult {
    status: u16,
    status_text: String,
    headers_raw: String,
    body: String,
}

/// Performs the actual network I/O on a background thread.
///
/// Kept free of any interpreter/heap references so it is safe to run off the
/// main thread. Errors are returned as plain strings (matching the previous
/// `fetch failed: …` message shape) and later surface as a rejected promise.
fn do_fetch_blocking(
    url: String,
    method: String,
    headers: std::collections::HashMap<String, String>,
    body: Option<String>,
) -> std::result::Result<RawFetchResult, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let mut req = match method.to_uppercase().as_str() {
        "POST" => client.post(&url),
        "PUT" => client.put(&url),
        "DELETE" => client.delete(&url),
        "PATCH" => client.patch(&url),
        "HEAD" => client.head(&url),
        "OPTIONS" => client.request(reqwest::Method::OPTIONS, &url),
        _ => client.get(&url),
    };

    for (key, value) in &headers {
        req = req.header(key.as_str(), value.as_str());
    }

    if let Some(body_str) = body {
        req = req.body(body_str);
    }

    let response = req.send().map_err(|e| format!("fetch failed: {}", e))?;

    let status = response.status().as_u16();
    let status_text = response
        .status()
        .canonical_reason()
        .unwrap_or("Unknown")
        .to_string();

    let mut resp_headers = Vec::new();
    for (key, value) in response.headers() {
        if let Ok(val) = value.to_str() {
            resp_headers.push(format!("{}\0{}", key.as_str().to_lowercase(), val));
        }
    }
    let headers_raw = resp_headers.join("\n");

    let body_text = response.text().unwrap_or_default();

    Ok(RawFetchResult {
        status,
        status_text,
        headers_raw,
        body: body_text,
    })
}

/// Drives an in-flight `fetch` request without blocking the interpreter thread.
///
/// The blocking network I/O runs on a worker thread (see [`do_fetch_blocking`]);
/// this source is polled by the event loop and, once the worker delivers its
/// result, resolves the associated promise — enqueuing the `.then`/`.catch`
/// continuations as microtasks for [`crate::vm::interpreter::Interpreter::drain_microtasks`].
struct FetchEventSource {
    rx: mpsc::Receiver<std::result::Result<RawFetchResult, String>>,
    promise_idx: usize,
    done: bool,
}

impl EventSource for FetchEventSource {
    fn is_active(&self) -> bool {
        !self.done
    }

    fn poll(&mut self, interp: &mut Interpreter) -> Result<()> {
        match self.rx.try_recv() {
            Ok(Ok(raw)) => {
                match build_response(
                    interp,
                    raw.body,
                    raw.status,
                    &raw.status_text,
                    &raw.headers_raw,
                ) {
                    Ok(response_value) => interp.resolve_promise(self.promise_idx, response_value),
                    Err(e) => {
                        interp.reject_promise(self.promise_idx, Value::from_string(e.to_string()))
                    }
                }
                self.done = true;
            }
            Ok(Err(msg)) => {
                interp.reject_promise(self.promise_idx, Value::from_string(msg));
                self.done = true;
            }
            Err(mpsc::TryRecvError::Empty) => {
                // Worker hasn't finished yet — remain pending and try again next tick.
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                // The worker ended without sending a result (e.g. panicked).
                if !self.done {
                    interp.reject_promise(
                        self.promise_idx,
                        Value::from_string(
                            "fetch failed: worker thread terminated unexpectedly".to_string(),
                        ),
                    );
                    self.done = true;
                }
            }
        }
        Ok(())
    }
}
