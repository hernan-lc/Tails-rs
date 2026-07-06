use crate::objects::Value;
use crate::props;
use crate::runtime_env::native_fns::helpers::to_string_value;
use crate::vm::interpreter::{HeapValue, Interpreter, JsObject};

use super::headers::{create_headers_props, get_string_prop};

pub(crate) fn native_request_constructor(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> crate::errors::Result<Value> {
    let url = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();

    let mut method = "GET".to_string();
    let mut headers_raw = String::new();
    let mut body: Option<String> = None;

    if let Some(Value::Object(obj_idx)) = args.first() {
        if let HeapValue::Object(obj) = &interp.heap[*obj_idx] {
            if obj.properties.contains_key("__is_request") {
                let cloned_url = obj
                    .properties
                    .get("url")
                    .map(|v| to_string_value(interp, v))
                    .unwrap_or_default();
                let cloned_method = obj
                    .properties
                    .get("method")
                    .map(|v| to_string_value(interp, v))
                    .unwrap_or_default();
                headers_raw = obj
                    .properties
                    .get("__headers")
                    .map(|v| {
                        if let Value::String(s) = v {
                            s.clone()
                        } else {
                            String::new()
                        }
                    })
                    .unwrap_or_default();
                body = get_string_prop(obj, "__body");
                method = cloned_method;
                let init_url = url;
                let _ = init_url;

                if let Some(Value::Object(init_idx)) = args.get(1) {
                    if let HeapValue::Object(init_obj) = &interp.heap[*init_idx] {
                        if let Some(m) = init_obj.properties.get("method") {
                            method = to_string_value(interp, m);
                        }
                    }
                }

                let mut props = props! {
                    "url" => Value::String(cloned_url),
                    "method" => Value::String(method.to_uppercase()),
                    "__headers" => Value::String(headers_raw.clone()),
                    "__body" => body.clone().map(Value::String).unwrap_or(Value::Null),
                    "bodyUsed" => Value::Boolean(false),
                    "__is_request" => Value::Boolean(true),
                    "__method" => Value::String(method.to_uppercase()),
                };

                let h_idx = interp.heap.len();
                let h_props = create_headers_props(&headers_raw);
                interp.heap.push(HeapValue::Object(JsObject {
                    properties: h_props,
                    prototype: None,
                    extensible: true,
                }));
                props.insert("headers".into(), Value::Object(h_idx));

                let idx = interp.heap.len();
                interp.heap.push(HeapValue::Object(JsObject {
                    properties: props,
                    prototype: None,
                    extensible: true,
                }));
                return Ok(Value::Object(idx));
            }
        }
    }

    if let Some(Value::Object(init_idx)) = args.get(1) {
        if let HeapValue::Object(init_obj) = &interp.heap[*init_idx] {
            if let Some(m) = init_obj.properties.get("method") {
                method = to_string_value(interp, m);
            }
            if let Some(Value::Object(hdr_idx)) = init_obj.properties.get("headers") {
                if let HeapValue::Object(hdr_obj) = &interp.heap[*hdr_idx] {
                    if let Some(Value::String(h)) = hdr_obj.properties.get("__headers") {
                        headers_raw = h.clone();
                    } else {
                        let mut header_strs = Vec::new();
                        for (k, v) in &hdr_obj.properties {
                            if !k.starts_with('_') {
                                let val = to_string_value(interp, v);
                                header_strs.push(format!("{}\0{}", k.to_lowercase(), val));
                            }
                        }
                        headers_raw = header_strs.join("\n");
                    }
                }
            }
            if let Some(b) = init_obj.properties.get("body") {
                body = Some(to_string_value(interp, b));
            }
        }
    }

    let mut props = props! {
        "url" => Value::String(url),
        "method" => Value::String(method.to_uppercase()),
        "__headers" => Value::String(headers_raw.clone()),
        "__body" => body.map(Value::String).unwrap_or(Value::Null),
        "bodyUsed" => Value::Boolean(false),
        "__is_request" => Value::Boolean(true),
    };

    let h_idx = interp.heap.len();
    let h_props = create_headers_props(&headers_raw);
    interp.heap.push(HeapValue::Object(JsObject {
        properties: h_props,
        prototype: None,
        extensible: true,
    }));
    props.insert("headers".into(), Value::Object(h_idx));

    let idx = interp.heap.len();
    interp.heap.push(HeapValue::Object(JsObject {
        properties: props,
        prototype: None,
        extensible: true,
    }));
    Ok(Value::Object(idx))
}
