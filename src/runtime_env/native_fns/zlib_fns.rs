use crate::errors::{Error, Result};
use crate::objects::Value;
use crate::vm::interpreter::{HeapValue, Interpreter};

use super::helpers::to_string_value;

use base64::Engine;

fn get_option_int(interp: &Interpreter, opts: &Value, key: &str) -> Option<i64> {
    if let Value::Object(idx) = opts {
        if let HeapValue::Object(obj) = &interp.heap[*idx] {
            if let Some(val) = obj.properties.get(key) {
                match val {
                    Value::Integer(n) => Some(*n),
                    Value::Float(n) => Some(*n as i64),
                    _ => None,
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}

pub(super) fn native_zlib_gzip_sync(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let input = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    let level = args.get(1).and_then(|v| get_option_int(interp, v, "level"));
    let encoder = if let Some(lvl) = level {
        flate2::write::GzEncoder::new(
            Vec::new(),
            flate2::Compression::new(lvl as u32),
        )
    } else {
        flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast())
    };
    use std::io::Write;
    let mut encoder = encoder;
    encoder
        .write_all(input.as_bytes())
        .map_err(|e| Error::RuntimeError(format!("gzip failed: {}", e)))?;
    let compressed = encoder
        .finish()
        .map_err(|e| Error::RuntimeError(format!("gzip finish failed: {}", e)))?;
    Ok(Value::String(
        base64::engine::general_purpose::STANDARD.encode(&compressed),
    ))
}

pub(super) fn native_zlib_gunzip_sync(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let input = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    let data = base64::engine::general_purpose::STANDARD
        .decode(&input)
        .map_err(|e| Error::RuntimeError(format!("base64 decode failed: {}", e)))?;
    use std::io::Read;
    let mut decoder = flate2::read::GzDecoder::new(&data[..]);
    let mut output = String::new();
    decoder
        .read_to_string(&mut output)
        .map_err(|e| Error::RuntimeError(format!("gunzip failed: {}", e)))?;
    Ok(Value::String(output))
}

pub(super) fn native_zlib_deflate_sync(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let input = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    let level = args.get(1).and_then(|v| get_option_int(interp, v, "level"));
    let encoder = if let Some(lvl) = level {
        flate2::write::DeflateEncoder::new(
            Vec::new(),
            flate2::Compression::new(lvl as u32),
        )
    } else {
        flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::fast())
    };
    use std::io::Write;
    let mut encoder = encoder;
    encoder
        .write_all(input.as_bytes())
        .map_err(|e| Error::RuntimeError(format!("deflate failed: {}", e)))?;
    let compressed = encoder
        .finish()
        .map_err(|e| Error::RuntimeError(format!("deflate finish failed: {}", e)))?;
    Ok(Value::String(
        base64::engine::general_purpose::STANDARD.encode(&compressed),
    ))
}

pub(super) fn native_zlib_inflate_sync(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let input = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    let data = base64::engine::general_purpose::STANDARD
        .decode(&input)
        .map_err(|e| Error::RuntimeError(format!("base64 decode failed: {}", e)))?;
    use std::io::Read;
    let mut decoder = flate2::read::DeflateDecoder::new(&data[..]);
    let mut output = String::new();
    decoder
        .read_to_string(&mut output)
        .map_err(|e| Error::RuntimeError(format!("inflate failed: {}", e)))?;
    Ok(Value::String(output))
}

pub(super) fn native_zlib_deflate_raw_sync(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    native_zlib_deflate_sync(interp, _this, args)
}

pub(super) fn native_zlib_inflate_raw_sync(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    native_zlib_inflate_sync(interp, _this, args)
}

pub(super) fn native_zlib_gzip(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    native_zlib_gzip_sync(interp, this, args)
}

pub(super) fn native_zlib_gunzip(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    native_zlib_gunzip_sync(interp, this, args)
}

pub(super) fn native_zlib_deflate(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    native_zlib_deflate_sync(interp, this, args)
}

pub(super) fn native_zlib_inflate(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    native_zlib_inflate_sync(interp, this, args)
}
