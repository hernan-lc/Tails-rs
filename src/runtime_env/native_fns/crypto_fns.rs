use crate::errors::Result;
use crate::objects::Value;
use crate::runtime_env::native_fns::constants as c;
use crate::vm::interpreter::{HeapValue, Interpreter, JsObject};
use crate::props;

use super::helpers::to_string_value;

pub(super) fn native_crypto_random_bytes(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let size = args
        .first()
        .map(|v| match v {
            Value::Integer(n) => *n as usize,
            Value::Float(n) => *n as usize,
            _ => 0,
        })
        .unwrap_or(0);

    let mut buf = vec![0u8; size];
    getrandom::getrandom(&mut buf).map_err(|e| {
        crate::errors::Error::RuntimeError(format!("crypto.randomBytes failed: {}", e))
    })?;

    let heap_idx = interp.heap.len();
    interp.heap.push(HeapValue::Buffer(buf));
    Ok(Value::Buffer(heap_idx))
}

pub(super) fn native_crypto_random_uuid(
    _interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let mut bytes = [0u8; 16];
    getrandom::getrandom(&mut bytes).map_err(|e| {
        crate::errors::Error::RuntimeError(format!("crypto.randomUUID failed: {}", e))
    })?;

    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;

    let uuid = format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5],
        bytes[6], bytes[7],
        bytes[8], bytes[9],
        bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
    );

    Ok(Value::String(uuid))
}

pub(super) fn native_crypto_create_hash(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let algorithm = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_else(|| "sha256".to_string());

    // Create data buffer first
    let data_buf_idx = interp.heap.len();
    interp.heap.push(HeapValue::Buffer(Vec::new()));

    // Create the hash object
    let props = props! {
        "_algorithm" => Value::String(algorithm),
        "_data" => Value::Object(data_buf_idx),
        "update" => Value::NativeFunction(c::CRYPTO_HASH_UPDATE),
        "digest" => Value::NativeFunction(c::CRYPTO_HASH_DIGEST),
    };

    let hash_obj_idx = interp.heap.len();
    interp.heap.push(HeapValue::Object(JsObject {
        properties: props,
        prototype: None,
        extensible: true,
    }));
    Ok(Value::Object(hash_obj_idx))
}

pub(super) fn native_crypto_hash_update(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let data = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();

    if let Value::Object(obj_idx) = this {
        // Get the data buffer index first
        let data_buf_idx = match &interp.heap[*obj_idx] {
            HeapValue::Object(obj) => match obj.properties.get("_data") {
                Some(Value::Object(idx)) => *idx,
                _ => return Ok(this.clone()),
            },
            _ => return Ok(this.clone()),
        };

        // Extend the buffer
        if let HeapValue::Buffer(existing) = &mut interp.heap[data_buf_idx] {
            existing.extend_from_slice(data.as_bytes());
        }
    }
    Ok(this.clone())
}

pub(super) fn native_crypto_hash_digest(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    if let Value::Object(obj_idx) = this {
        // Get algorithm and data
        let (algorithm, data) = match &interp.heap[*obj_idx] {
            HeapValue::Object(obj) => {
                let alg = match obj.properties.get("_algorithm") {
                    Some(Value::String(s)) => s.clone(),
                    Some(Value::Cons(c)) => c.flatten(),
                    _ => "sha256".to_string(),
                };
                let buf_idx = match obj.properties.get("_data") {
                    Some(Value::Object(idx)) => *idx,
                    _ => return Ok(Value::String("".to_string())),
                };
                let buf = match &interp.heap[buf_idx] {
                    HeapValue::Buffer(b) => b.clone(),
                    _ => Vec::new(),
                };
                (alg, buf)
            }
            _ => return Ok(Value::String("".to_string())),
        };

        let hash_hex = match algorithm.as_str() {
            "sha224" => {
                use sha2::{Digest, Sha224};
                let mut hasher = Sha224::new();
                hasher.update(&data);
                hex_encode(&hasher.finalize())
            }
            "sha256" => {
                use sha2::{Digest, Sha256};
                let mut hasher = Sha256::new();
                hasher.update(&data);
                hex_encode(&hasher.finalize())
            }
            "sha384" => {
                use sha2::{Digest, Sha384};
                let mut hasher = Sha384::new();
                hasher.update(&data);
                hex_encode(&hasher.finalize())
            }
            "sha512" => {
                use sha2::{Digest, Sha512};
                let mut hasher = Sha512::new();
                hasher.update(&data);
                hex_encode(&hasher.finalize())
            }
            _ => {
                use sha2::{Digest, Sha256};
                let mut hasher = Sha256::new();
                hasher.update(&data);
                hex_encode(&hasher.finalize())
            }
        };

        return Ok(Value::String(hash_hex));
    }
    Ok(Value::String("".to_string()))
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}
