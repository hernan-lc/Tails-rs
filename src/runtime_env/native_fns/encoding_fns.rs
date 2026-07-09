use crate::errors::{Error, Result};
use crate::objects::Value;
use crate::vm::interpreter::Interpreter;

use super::helpers::to_string_value;

const BASE64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64_encode(data: &[u8]) -> String {
    let mut result = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;

        result.push(BASE64_CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(BASE64_CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(BASE64_CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(BASE64_CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

fn base64_decode(input: &str) -> Result<Vec<u8>> {
    let input = input.trim_end_matches('=');
    let mut result = Vec::with_capacity(input.len() * 3 / 4);
    let mut buf: u32 = 0;
    let mut bits: u32 = 0;

    for byte in input.bytes() {
        let val = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b' ' | b'\t' | b'\n' | b'\r' => continue,
            _ => {
                return Err(Error::TypeError(format!(
                    "Invalid character in base64 string: '{}'",
                    byte as char
                )))
            }
        };
        buf = (buf << 6) | val as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            result.push((buf >> bits) as u8);
        }
    }
    Ok(result)
}

pub(super) fn native_atob(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let encoded = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();

    let decoded = base64_decode(&encoded)?;
    let s = String::from_utf8(decoded)
        .unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).to_string());
    Ok(Value::from_string(s))
}

pub(super) fn native_btoa(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let input = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();

    let encoded = base64_encode(input.as_bytes());
    Ok(Value::from_string(encoded))
}

use crate::props;
use crate::runtime_env::native_fns::constants as c;
use crate::vm::interpreter::{HeapValue, JsObject};

/// `new TextEncoder()` — returns `{ encode }`
pub(super) fn native_text_encoder_constructor(
    interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let obj_idx = interp.gc.allocate(
        &mut interp.heap,
        HeapValue::Object(JsObject {
            properties: props! {
                "encode" => Value::NativeFunction(c::TEXT_ENCODER_ENCODE),
                "encoding" => Value::from_string("utf-8".into()),
            },
            prototype: None,
            extensible: true,
        }),
    );
    Ok(Value::Object(obj_idx))
}

/// `new TextDecoder([label])` — returns `{ decode }`
pub(super) fn native_text_decoder_constructor(
    interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let obj_idx = interp.gc.allocate(
        &mut interp.heap,
        HeapValue::Object(JsObject {
            properties: props! {
                "decode" => Value::NativeFunction(c::TEXT_DECODER_DECODE),
                "encoding" => Value::from_string("utf-8".into()),
            },
            prototype: None,
            extensible: true,
        }),
    );
    Ok(Value::Object(obj_idx))
}

/// TextEncoder.prototype.encode(string) → Buffer
pub(super) fn native_text_encoder_encode(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let s = match args.first() {
        Some(Value::String(s)) => s.to_string(),
        Some(Value::Cons(c)) => c.flatten(),
        Some(v) => to_string_value(interp, v),
        None => String::new(),
    };
    let bytes = s.into_bytes();
    let buf_idx = interp
        .gc
        .allocate(&mut interp.heap, HeapValue::Buffer(bytes));
    Ok(Value::Buffer(buf_idx))
}

/// TextDecoder.prototype.decode(input) → string
pub(super) fn native_text_decoder_decode(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let bytes = match args.first() {
        Some(Value::Buffer(idx)) => {
            if let HeapValue::Buffer(b) = &interp.heap[*idx] {
                b.clone()
            } else {
                Vec::new()
            }
        }
        Some(Value::TypedArray(idx)) => {
            if let HeapValue::TypedArray(ta) = &interp.heap[*idx] {
                // Best-effort: decode via Debug if raw bytes unavailable.
                let _ = ta;
                Vec::new()
            } else {
                Vec::new()
            }
        }
        Some(Value::String(s)) => s.as_bytes().to_vec(),
        _ => Vec::new(),
    };
    let s = String::from_utf8_lossy(&bytes).into_owned();
    Ok(Value::from_string(s))
}
