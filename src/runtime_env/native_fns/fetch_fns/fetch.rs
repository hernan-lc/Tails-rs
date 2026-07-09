use crate::errors::{Error, Result};
use crate::objects::js_promise::JsPromise;
use crate::objects::Value;
use crate::runtime_env::native_fns::helpers::to_string_value;
use crate::vm::interpreter::{HeapValue, Interpreter};

use super::headers::parse_headers;
use super::response::build_response;

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

    let result = execute_fetch(interp, &url, &method, &headers_map, body.as_deref());

    match result {
        Ok(response_value) => {
            let promise = JsPromise::fulfilled(response_value);
            let promise_idx = interp.heap.len();
            interp.heap.push(HeapValue::Promise(promise));
            Ok(Value::Promise(promise_idx))
        }
        Err(e) => {
            let err_msg = Value::from_string(e.to_string());
            let promise = JsPromise::rejected(err_msg);
            let promise_idx = interp.heap.len();
            interp.heap.push(HeapValue::Promise(promise));
            Ok(Value::Promise(promise_idx))
        }
    }
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

fn execute_fetch(
    interp: &mut Interpreter,
    url: &str,
    method: &str,
    headers: &std::collections::HashMap<String, String>,
    body: Option<&str>,
) -> Result<Value> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()
        .map_err(|e| Error::RuntimeError(format!("Failed to create HTTP client: {}", e)))?;

    let mut req = match method.to_uppercase().as_str() {
        "POST" => client.post(url),
        "PUT" => client.put(url),
        "DELETE" => client.delete(url),
        "PATCH" => client.patch(url),
        "HEAD" => client.head(url),
        "OPTIONS" => client.request(reqwest::Method::OPTIONS, url),
        _ => client.get(url),
    };

    for (key, value) in headers {
        req = req.header(key.as_str(), value.as_str());
    }

    if let Some(body_str) = body {
        req = req.body(body_str.to_string());
    }

    let response = req
        .send()
        .map_err(|e| Error::RuntimeError(format!("fetch failed: {}", e)))?;

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

    build_response(interp, body_text, status, &status_text, &headers_raw)
}
