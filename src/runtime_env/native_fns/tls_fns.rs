use crate::errors::{Error, Result};
use crate::objects::Value;
use crate::vm::interpreter::{HeapValue, Interpreter, JsObject};

use super::helpers::to_string_value;

pub(super) fn native_tls_connect(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let opts = args.first().cloned().unwrap_or(Value::Undefined);
    let host = if let Value::Object(idx) = &opts {
        if let HeapValue::Object(obj) = &interp.heap[*idx] {
            obj.properties
                .get("host")
                .map(|v| to_string_value(interp, v))
                .unwrap_or_else(|| "localhost".to_string())
        } else {
            "localhost".to_string()
        }
    } else {
        "localhost".to_string()
    };
    let port = if let Value::Object(idx) = &opts {
        if let HeapValue::Object(obj) = &interp.heap[*idx] {
            obj.properties
                .get("port")
                .map(|v| match v {
                    Value::Integer(n) => *n as u16,
                    Value::Float(n) => *n as u16,
                    _ => 443,
                })
                .unwrap_or(443)
        } else {
            443
        }
    } else {
        443
    };

    let connector = native_tls::TlsConnector::new()
        .map_err(|e| Error::RuntimeError(format!("TLS connector failed: {}", e)))?;
    let stream = std::net::TcpStream::connect(format!("{}:{}", host, port))
        .map_err(|e| Error::RuntimeError(format!("TCP connect failed: {}", e)))?;
    let _tls_stream = connector
        .connect(&host, stream)
        .map_err(|e| Error::RuntimeError(format!("TLS handshake failed: {}", e)))?;

    let socket_idx = interp.gc.allocate(
        &mut interp.heap,
        HeapValue::Object(JsObject {
            properties: crate::props! {
                "host" => Value::String(host),
                "port" => Value::Integer(port as i64),
                "authorized" => Value::Boolean(true),
            },
            prototype: None,
            extensible: true,
        }),
    );
    Ok(Value::Object(socket_idx))
}

pub(super) fn native_tls_create_secure_context(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let opts = args.first().cloned().unwrap_or(Value::Undefined);
    let ctx_idx = interp.gc.allocate(
        &mut interp.heap,
        HeapValue::Object(JsObject {
            properties: crate::props! {
                "options" => opts,
            },
            prototype: None,
            extensible: true,
        }),
    );
    Ok(Value::Object(ctx_idx))
}

pub(super) fn native_tls_socket_write(
    _interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let _data = args.first().cloned().unwrap_or(Value::Undefined);
    Ok(Value::Boolean(true))
}

pub(super) fn native_tls_socket_end(
    _interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    Ok(this.clone())
}

pub(super) fn native_tls_create_server(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let opts = args.first().cloned().unwrap_or(Value::Undefined);
    let server_idx = interp.gc.allocate(
        &mut interp.heap,
        HeapValue::Object(JsObject {
            properties: crate::props! {
                "options" => opts,
            },
            prototype: None,
            extensible: true,
        }),
    );
    Ok(Value::Object(server_idx))
}
