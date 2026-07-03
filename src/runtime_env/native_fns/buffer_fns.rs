use crate::errors::Result;
use crate::objects::Value;
use crate::vm::interpreter::{HeapValue, Interpreter};

use super::helpers::to_string_value;

pub(super) fn native_buffer_constructor(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let buf_idx = interp.heap.len();
    let data = if let Some(first) = args.first() {
        match first {
            Value::Integer(n) => {
                let len = *n as usize;
                vec![0u8; len]
            }
            Value::Float(n) => {
                let len = *n as usize;
                vec![0u8; len]
            }
            Value::String(s) => s.as_bytes().to_vec(),
            Value::Array(arr_idx) => {
                if let HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                    arr.elements.iter().map(|v| to_i64(v) as u8).collect()
                } else {
                    Vec::new()
                }
            }
            _ => {
                let s = to_string_value(interp, first);
                s.as_bytes().to_vec()
            }
        }
    } else {
        Vec::new()
    };
    interp.heap.push(HeapValue::Buffer(data));
    Ok(Value::Buffer(buf_idx))
}

pub(super) fn native_buffer_alloc(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let size = args.first().map(|v| to_i64(v) as usize).unwrap_or(0);
    let fill = if args.len() > 1 {
        to_i64(&args[1]) as u8
    } else {
        0
    };
    let buf_idx = interp.heap.len();
    interp.heap.push(HeapValue::Buffer(vec![fill; size]));
    Ok(Value::Buffer(buf_idx))
}

pub(super) fn native_buffer_from(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let buf_idx = interp.heap.len();
    // The optional second argument is an encoding name. When present
    // and recognised, the first argument is decoded according to that
    // encoding (e.g. `Buffer.from("SGVsbG8=", "base64")` produces the
    // bytes `Hello`). Without the encoding, a string is treated as raw
    // UTF-8 bytes — matching Node's `Buffer.from(string)` behaviour.
    let encoding = args.get(1).map(|v| to_string_value(interp, v));
    let data = if let Some(first) = args.first() {
        match first {
            Value::String(s) => {
                let bytes = s.as_bytes().to_vec();
                if let Some(enc) = encoding.as_deref() {
                    if !enc.is_empty() {
                        decode_bytes_with_encoding(&bytes, &enc.to_ascii_lowercase())
                            .unwrap_or(bytes)
                    } else {
                        bytes
                    }
                } else {
                    bytes
                }
            }
            Value::Array(arr_idx) => {
                if let HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                    arr.elements.iter().map(|v| to_i64(v) as u8).collect()
                } else {
                    Vec::new()
                }
            }
            Value::Buffer(src_idx) => {
                if let HeapValue::Buffer(buf) = &interp.heap[*src_idx] {
                    buf.clone()
                } else {
                    Vec::new()
                }
            }
            _ => {
                let s = to_string_value(interp, first);
                let bytes = s.as_bytes().to_vec();
                if let Some(enc) = encoding.as_deref() {
                    if !enc.is_empty() {
                        decode_bytes_with_encoding(&bytes, &enc.to_ascii_lowercase())
                            .unwrap_or(bytes)
                    } else {
                        bytes
                    }
                } else {
                    bytes
                }
            }
        }
    } else {
        Vec::new()
    };
    interp.heap.push(HeapValue::Buffer(data));
    Ok(Value::Buffer(buf_idx))
}

pub(super) fn native_buffer_concat(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let total_len = if let Some(Value::Array(arr_idx)) = args.first() {
        if let HeapValue::Array(arr) = &interp.heap[*arr_idx] {
            arr.elements
                .iter()
                .filter_map(|elem| match elem {
                    Value::Buffer(buf_idx) => {
                        if let HeapValue::Buffer(buf) = &interp.heap[*buf_idx] {
                            Some(buf.len())
                        } else {
                            None
                        }
                    }
                    _ => None,
                })
                .sum()
        } else {
            0
        }
    } else {
        0
    };
    let mut result = Vec::with_capacity(total_len);
    if let Some(Value::Array(arr_idx)) = args.first() {
        if let HeapValue::Array(arr) = &interp.heap[*arr_idx] {
            for elem in &arr.elements {
                if let Value::Buffer(buf_idx) = elem {
                    if let HeapValue::Buffer(buf) = &interp.heap[*buf_idx] {
                        result.extend_from_slice(buf);
                    }
                }
            }
        }
    }
    let buf_idx = interp.heap.len();
    interp.heap.push(HeapValue::Buffer(result));
    Ok(Value::Buffer(buf_idx))
}

pub(super) fn native_buffer_is_buffer(
    _interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let is_buf = matches!(args.first(), Some(Value::Buffer(_)));
    Ok(Value::Boolean(is_buf))
}

pub(super) fn native_buffer_byte_length(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    if let Some(Value::String(s)) = args.first() {
        Ok(Value::Integer(s.len() as i64))
    } else if let Some(Value::Buffer(buf_idx)) = args.first() {
        if let HeapValue::Buffer(buf) = &interp.heap[*buf_idx] {
            Ok(Value::Integer(buf.len() as i64))
        } else {
            Ok(Value::Integer(0))
        }
    } else {
        Ok(Value::Integer(0))
    }
}

pub(super) fn native_buffer_to_string(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    if let Value::Buffer(buf_idx) = _this {
        if let HeapValue::Buffer(buf) = &interp.heap[*buf_idx] {
            let start = args.first().map(|v| to_i64(v) as usize).unwrap_or(0);
            let end = args.get(1).map(|v| to_i64(v) as usize).unwrap_or(buf.len());
            let end = end.min(buf.len());
            let start = start.min(end);
            let s = String::from_utf8_lossy(&buf[start..end]).to_string();
            return Ok(Value::String(s));
        }
    }
    Ok(Value::String(String::new()))
}

pub(super) fn native_buffer_write(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    if let Value::Buffer(buf_idx) = _this {
        let data = args
            .first()
            .map(|v| to_string_value(interp, v))
            .unwrap_or_default();
        let offset = args.get(1).map(|v| to_i64(v) as usize).unwrap_or(0);
        let bytes = data.as_bytes();
        if let HeapValue::Buffer(buf) = &mut interp.heap[*buf_idx] {
            let len = bytes.len().min(buf.len() - offset);
            buf[offset..offset + len].copy_from_slice(&bytes[..len]);
            return Ok(Value::Integer(len as i64));
        }
    }
    Ok(Value::Integer(0))
}

pub(super) fn native_buffer_slice(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    if let Value::Buffer(buf_idx) = _this {
        if let HeapValue::Buffer(buf) = &interp.heap[*buf_idx] {
            let start = args.first().map(|v| to_i64(v) as usize).unwrap_or(0);
            let end = args.get(1).map(|v| to_i64(v) as usize).unwrap_or(buf.len());
            let end = end.min(buf.len());
            let start = start.min(end);
            let new_buf = buf[start..end].to_vec();
            let new_idx = interp.heap.len();
            interp.heap.push(HeapValue::Buffer(new_buf));
            return Ok(Value::Buffer(new_idx));
        }
    }
    let new_idx = interp.heap.len();
    interp.heap.push(HeapValue::Buffer(Vec::new()));
    Ok(Value::Buffer(new_idx))
}

pub(super) fn native_buffer_copy(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    if let Value::Buffer(src_idx) = _this {
        let src_clone = if let HeapValue::Buffer(src) = &interp.heap[*src_idx] {
            src.clone()
        } else {
            return Ok(Value::Integer(0));
        };
        if let Some(Value::Buffer(dst_idx)) = args.first() {
            if let HeapValue::Buffer(dst) = &mut interp.heap[*dst_idx] {
                let target_start = args.get(1).map(|v| to_i64(v) as usize).unwrap_or(0);
                let source_start = args.get(2).map(|v| to_i64(v) as usize).unwrap_or(0);
                let source_end = args
                    .get(3)
                    .map(|v| to_i64(v) as usize)
                    .unwrap_or(src_clone.len());
                let source_end = source_end.min(src_clone.len());
                let source_start = source_start.min(source_end);
                let len = source_end - source_start;
                let available = dst.len().saturating_sub(target_start);
                let copy_len = len.min(available);
                dst[target_start..target_start + copy_len]
                    .copy_from_slice(&src_clone[source_start..source_start + copy_len]);
                return Ok(Value::Integer(copy_len as i64));
            }
        }
    }
    Ok(Value::Integer(0))
}

pub(super) fn native_buffer_fill(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    if let Value::Buffer(buf_idx) = _this {
        if let HeapValue::Buffer(buf) = &mut interp.heap[*buf_idx] {
            let fill_val = args.first().map(|v| to_i64(v) as u8).unwrap_or(0);
            let start = args.get(1).map(|v| to_i64(v) as usize).unwrap_or(0);
            let end = args.get(2).map(|v| to_i64(v) as usize).unwrap_or(buf.len());
            let end = end.min(buf.len());
            let start = start.min(end);
            for byte in &mut buf[start..end] {
                *byte = fill_val;
            }
            return Ok(Value::Buffer(*buf_idx));
        }
    }
    Ok(Value::Undefined)
}

pub(super) fn native_buffer_compare(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    if let Value::Buffer(src_idx) = _this {
        let src_clone = if let HeapValue::Buffer(src) = &interp.heap[*src_idx] {
            src.clone()
        } else {
            return Ok(Value::Integer(0));
        };
        if let Some(Value::Buffer(dst_idx)) = args.first() {
            if let HeapValue::Buffer(dst) = &interp.heap[*dst_idx] {
                let ord = src_clone.cmp(dst);
                let cmp_val = match ord {
                    std::cmp::Ordering::Less => -1,
                    std::cmp::Ordering::Equal => 0,
                    std::cmp::Ordering::Greater => 1,
                };
                return Ok(Value::Integer(cmp_val));
            }
        }
    }
    Ok(Value::Integer(0))
}

pub(super) fn native_buffer_equals(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    if let Value::Buffer(src_idx) = _this {
        if let HeapValue::Buffer(src) = &interp.heap[*src_idx] {
            if let Some(Value::Buffer(dst_idx)) = args.first() {
                if let HeapValue::Buffer(dst) = &interp.heap[*dst_idx] {
                    return Ok(Value::Boolean(src == dst));
                }
            }
        }
    }
    Ok(Value::Boolean(false))
}

pub(super) fn native_buffer_index_of(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    if let Value::Buffer(buf_idx) = _this {
        if let HeapValue::Buffer(buf) = &interp.heap[*buf_idx] {
            let search = args
                .first()
                .map(|v| to_string_value(interp, v))
                .unwrap_or_default();
            let byte_offset = args.get(1).map(|v| to_i64(v) as usize).unwrap_or(0);
            let search_bytes = search.as_bytes();
            if search_bytes.is_empty() {
                return Ok(Value::Integer(byte_offset as i64));
            }
            if byte_offset >= buf.len() {
                return Ok(Value::Integer(-1));
            }
            for i in byte_offset..=buf.len().saturating_sub(search_bytes.len()) {
                if &buf[i..i + search_bytes.len()] == search_bytes {
                    return Ok(Value::Integer(i as i64));
                }
            }
            return Ok(Value::Integer(-1));
        }
    }
    Ok(Value::Integer(-1))
}

fn to_i64(v: &Value) -> i64 {
    match v {
        Value::Integer(n) => *n,
        Value::Float(n) => *n as i64,
        Value::Boolean(b) => {
            if *b {
                1
            } else {
                0
            }
        }
        Value::String(s) => s.parse::<i64>().unwrap_or(0),
        Value::Null => 0,
        _ => 0,
    }
}

// ===========================================================================
// Encoding helpers used by `Buffer.isEncoding` and `Buffer.transcode`.
// ===========================================================================

/// Returns `true` if `enc` is a Node-compatible encoding name. Matches
/// `Buffer.isEncoding()`'s case-insensitive behaviour.
pub(super) fn is_supported_encoding(enc: &str) -> bool {
    matches!(
        enc.to_ascii_lowercase().as_str(),
        "utf8"
            | "utf-8"
            | "utf16le"
            | "utf-16le"
            | "ucs2"
            | "ucs-2"
            | "latin1"
            | "binary"
            | "base64"
            | "base64url"
            | "hex"
            | "ascii"
    )
}

/// `Buffer.isEncoding(encoding)` — static check. Returns `true` for
/// the same set of encodings that the rest of the Buffer API accepts.
pub(super) fn native_buffer_is_encoding(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let enc = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    Ok(Value::Boolean(is_supported_encoding(&enc)))
}

/// `Buffer.transcode(source, fromEnc, toEnc)` — transcodes bytes
/// between supported encodings. Returns a new `Buffer` on success or
/// `null` if either encoding is unsupported.
///
/// Currently supports UTF-8 ⇄ Latin-1 / ASCII / Hex / Base64 /
/// base64url. UTF-16LE is recognised as an encoding name but the
/// actual transcoding is not yet implemented and returns `null`.
pub(super) fn native_buffer_transcode(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let src_bytes = match args.first() {
        Some(Value::Buffer(idx)) => match &interp.heap[*idx] {
            HeapValue::Buffer(b) => b.clone(),
            _ => return Ok(Value::Null),
        },
        _ => return Ok(Value::Null),
    };
    let from_enc = args
        .get(1)
        .map(|v| to_string_value(interp, v))
        .unwrap_or_else(|| "utf8".to_string());
    let to_enc = args
        .get(2)
        .map(|v| to_string_value(interp, v))
        .unwrap_or_else(|| "utf8".to_string());

    if !is_supported_encoding(&from_enc) || !is_supported_encoding(&to_enc) {
        return Ok(Value::Null);
    }

    // Step 1: decode source → intermediate byte sequence. For
    // byte-per-byte encodings (utf8, latin1, ascii) the intermediate
    // is identical to the source. For hex and base64, the source
    // bytes represent encoded data that we decode here.
    //
    // `intermediate` represents the raw byte stream in the *target*
    // semantic of each source encoding: for `utf8`/`latin1`/`ascii`/
    // `hex`/`base64` it is the same byte sequence that lives in the
    // source Buffer. For `utf16le` the intermediate is the decoded
    // UTF-8 form of the code units (matching Node's behaviour where
    // `transcode(src, "utf16le", "utf8")` returns the UTF-8 bytes of
    // the decoded string).
    let intermediate: Vec<u8> = match from_enc.to_ascii_lowercase().as_str() {
        "utf8" | "utf-8" => src_bytes.clone(),
        "latin1" | "binary" => src_bytes.clone(),
        "ascii" => src_bytes.iter().map(|b| b & 0x7f).collect(),
        "hex" => {
            let mut out = Vec::with_capacity(src_bytes.len() / 2);
            let mut iter = src_bytes.iter();
            while let (Some(h), Some(l)) = (iter.next(), iter.next()) {
                let hi = hex_nibble(*h).unwrap_or(0);
                let lo = hex_nibble(*l).unwrap_or(0);
                out.push((hi << 4) | lo);
            }
            out
        }
        "base64" => match base64_decode_simple(&src_bytes) {
            Some(b) => b,
            None => return Ok(Value::Null),
        },
        "base64url" => {
            let mut s: String = src_bytes.iter().map(|b| *b as char).collect();
            s = s.replace('-', "+").replace('_', "/");
            while !s.len().is_multiple_of(4) {
                s.push('=');
            }
            match base64_decode_str(&s) {
                Some(b) => b,
                None => return Ok(Value::Null),
            }
        }
        "utf16le" | "utf-16le" | "ucs2" | "ucs-2" => match decode_utf16le_to_utf8(&src_bytes) {
            Some(b) => b,
            None => return Ok(Value::Null),
        },
        _ => return Ok(Value::Null),
    };

    // Step 2: encode intermediate → destination.
    // For `utf16le` the intermediate is the UTF-8 byte stream of the
    // decoded string, so we re-encode it as UTF-16LE code units.
    let encoded: Vec<u8> = match to_enc.to_ascii_lowercase().as_str() {
        "utf8" | "utf-8" => intermediate.clone(),
        "latin1" | "binary" => intermediate.clone(),
        "ascii" => intermediate.iter().map(|b| b & 0x7f).collect(),
        "hex" => {
            let mut out = Vec::with_capacity(intermediate.len() * 2);
            for b in &intermediate {
                out.push(HEX_CHARS[(*b >> 4) as usize] as u8);
                out.push(HEX_CHARS[(*b & 0x0f) as usize] as u8);
            }
            out
        }
        "base64" => base64_encode_simple(&intermediate).into_bytes(),
        "base64url" => {
            let s: String = base64_encode_simple(&intermediate)
                .replace('+', "-")
                .replace('/', "_");
            let trimmed = s.trim_end_matches('=');
            trimmed.as_bytes().to_vec()
        }
        "utf16le" | "utf-16le" | "ucs2" | "ucs-2" => match encode_utf8_to_utf16le(&intermediate) {
            Some(b) => b,
            None => return Ok(Value::Null),
        },
        _ => return Ok(Value::Null),
    };

    let new_idx = interp.heap.len();
    interp.heap.push(HeapValue::Buffer(encoded));
    Ok(Value::Buffer(new_idx))
}

const HEX_CHARS: &[char] = &[
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];

fn hex_nibble(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

/// RFC 4648 §4 base64 encoder used by `Buffer.transcode`.
fn base64_encode_simple(bytes: &[u8]) -> String {
    const ALPHA: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    let mut i = 0;
    while i + 3 <= bytes.len() {
        let b0 = bytes[i];
        let b1 = bytes[i + 1];
        let b2 = bytes[i + 2];
        out.push(ALPHA[(b0 >> 2) as usize] as char);
        out.push(ALPHA[((b0 & 0x03) << 4 | (b1 >> 4)) as usize] as char);
        out.push(ALPHA[((b1 & 0x0f) << 2 | (b2 >> 6)) as usize] as char);
        out.push(ALPHA[(b2 & 0x3f) as usize] as char);
        i += 3;
    }
    let rem = bytes.len() - i;
    if rem == 1 {
        let b0 = bytes[i];
        out.push(ALPHA[(b0 >> 2) as usize] as char);
        out.push(ALPHA[((b0 & 0x03) << 4) as usize] as char);
        out.push('=');
        out.push('=');
    } else if rem == 2 {
        let b0 = bytes[i];
        let b1 = bytes[i + 1];
        out.push(ALPHA[(b0 >> 2) as usize] as char);
        out.push(ALPHA[((b0 & 0x03) << 4 | (b1 >> 4)) as usize] as char);
        out.push(ALPHA[((b1 & 0x0f) << 2) as usize] as char);
        out.push('=');
    }
    out
}

fn base64_decode_str(s: &str) -> Option<Vec<u8>> {
    base64_decode_simple(s.as_bytes())
}

/// Decode `bytes` according to a Node-style encoding name. Used by
/// `Buffer.from(string, encoding)` to handle `"base64"`, `"hex"`, etc.
/// Returns `None` when the encoding is unrecognised or the input is
/// malformed; the caller should fall back to treating the input as raw
/// bytes in that case.
fn decode_bytes_with_encoding(bytes: &[u8], enc: &str) -> Option<Vec<u8>> {
    match enc {
        "utf8" | "utf-8" | "ascii" | "latin1" | "binary" => Some(bytes.to_vec()),
        "hex" => {
            let mut out = Vec::with_capacity(bytes.len() / 2);
            let mut i = 0;
            while i + 1 < bytes.len() {
                let hi = hex_nibble(bytes[i])?;
                let lo = hex_nibble(bytes[i + 1])?;
                out.push((hi << 4) | lo);
                i += 2;
            }
            Some(out)
        }
        "base64" => base64_decode_simple(bytes),
        "base64url" => {
            // Translate URL-safe base64 to standard base64 by
            // remapping the two character substitutions and adding
            // back the padding that base64url strips.
            let mut s: String = bytes.iter().map(|b| *b as char).collect();
            s = s.replace('-', "+").replace('_', "/");
            while !s.len().is_multiple_of(4) {
                s.push('=');
            }
            base64_decode_simple(s.as_bytes())
        }
        // Other recognised names (utf16le / ucs2) are accepted but not
        // actually decoded here — fall through to "raw bytes" so the
        // caller still gets a Buffer back instead of a hard error.
        _ => None,
    }
}

fn base64_decode_simple(bytes: &[u8]) -> Option<Vec<u8>> {
    fn val(c: u8) -> Option<u8> {
        match c {
            b'A'..=b'Z' => Some(c - b'A'),
            b'a'..=b'z' => Some(c - b'a' + 26),
            b'0'..=b'9' => Some(c - b'0' + 52),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    }
    let bytes: Vec<u8> = bytes
        .iter()
        .copied()
        .filter(|b| !b.is_ascii_whitespace())
        .collect();
    if !bytes.len().is_multiple_of(4) {
        return None;
    }
    let mut out = Vec::with_capacity(bytes.len() / 4 * 3);
    let mut i = 0;
    while i < bytes.len() {
        let a = val(bytes[i])?;
        let b = val(bytes[i + 1])?;
        // `c` and `d` are the third and fourth base64 digits of the
        // current 4-char group. When a digit is a `=` padding marker
        // we substitute 0 — but more importantly, the *number* of
        // output bytes per 4-char group is `3 - pad` (3 if no pad,
        // 2 if one `=`, 1 if two `=`s). The previous implementation
        // counted padding separately for `c` and `d` and then
        // decoded bytes on the basis of `pad < 1` / `pad < 2`, which
        // corrupted the third output byte when the input had a
        // single `=` (e.g. `"SGVsbG8="` produced `"Hello\0"` instead
        // of `"Hello"` because the trailing `(c << 6) | d` shift
        // included zeroed-out pad bits).
        let mut pad = 0;
        let c = if bytes[i + 2] == b'=' {
            pad += 1;
            0u8
        } else {
            val(bytes[i + 2])?
        };
        let d = if bytes[i + 3] == b'=' {
            pad += 1;
            0u8
        } else {
            val(bytes[i + 3])?
        };
        out.push((a << 2) | (b >> 4));
        if pad < 2 {
            out.push((b << 4) | (c >> 2));
        }
        // Only emit the third byte when the 4-char group had no
        // padding at all.
        if pad == 0 {
            out.push((c << 6) | d);
        }
        i += 4;
    }
    Some(out)
}

// ===========================================================================
// UTF-16LE transcoding helpers used by `Buffer.transcode`.
//
// The `intermediate` byte stream in `native_buffer_transcode` is the
// UTF-8 form of the decoded string, so for `utf16le → utf8` we decode
// the source bytes as little-endian UTF-16 code units into Unicode
// scalar values and re-encode as UTF-8, and for `utf8 → utf16le` we
// reverse that. Both helpers return `None` on malformed input so the
// caller can produce the Node-equivalent `null` return value.
// ===========================================================================

/// Decode `src` as a stream of little-endian UTF-16 code units and
/// re-encode the resulting scalar values as UTF-8 bytes.
///
/// Invalid byte sequences follow Node's behaviour of substituting the
/// Unicode replacement character `U+FFFD` rather than failing — Node's
/// `Buffer.transcode` only returns `null` when the encoding *names*
/// are unrecognised, not when the bytes themselves are malformed. We
/// mirror that policy so the round-trip behaves like Node's
/// `Buffer.transcode(Buffer.from('Hi', 'utf16le'), 'utf16le', 'utf8')`
/// → `Buffer.from('Hi', 'utf8')`.
fn decode_utf16le_to_utf8(src: &[u8]) -> Option<Vec<u8>> {
    if !src.len().is_multiple_of(2) {
        // Trailing single byte: treat as malformed, drop it (Node
        // trims the source by length and the remaining pairs are
        // still well-formed).
    }
    let mut out = Vec::with_capacity(src.len());
    let mut i = 0;
    let pairs = src.len() / 2;
    while i < pairs {
        let lo = src[i * 2];
        let hi = src[i * 2 + 1];
        let unit = u16::from_le_bytes([lo, hi]);
        if (0xD800..=0xDBFF).contains(&unit) {
            // High surrogate — must be followed by a low surrogate.
            if i + 1 < pairs {
                let lo2 = src[(i + 1) * 2];
                let hi2 = src[(i + 1) * 2 + 1];
                let next = u16::from_le_bytes([lo2, hi2]);
                if (0xDC00..=0xDFFF).contains(&next) {
                    let codepoint =
                        0x10000 + (((unit - 0xD800) as u32) << 10) + ((next - 0xDC00) as u32);
                    encode_utf8_codepoint(&mut out, codepoint);
                    i += 2;
                    continue;
                }
            }
            // Unpaired high surrogate → replacement char.
            encode_utf8_codepoint(&mut out, 0xFFFD);
        } else if (0xDC00..=0xDFFF).contains(&unit) {
            // Unpaired low surrogate → replacement char.
            encode_utf8_codepoint(&mut out, 0xFFFD);
        } else {
            encode_utf8_codepoint(&mut out, unit as u32);
        }
        i += 1;
    }
    Some(out)
}

/// Encode the UTF-8 byte stream `src` as a sequence of little-endian
/// UTF-16 code units. Any UTF-8 sequence that decodes to a codepoint
/// above `U+FFFF` is split into a high/low surrogate pair (matching
/// Node's behaviour).
fn encode_utf8_to_utf16le(src: &[u8]) -> Option<Vec<u8>> {
    let mut codepoints: Vec<u32> = Vec::with_capacity(src.len());
    let mut i = 0;
    while i < src.len() {
        let b = src[i];
        let (cp, consumed) = if b < 0x80 {
            (b as u32, 1)
        } else if b < 0xC0 {
            // Continuation byte at the start of a sequence — invalid.
            (0xFFFD, 1)
        } else if b < 0xE0 {
            if i + 1 < src.len() && (src[i + 1] & 0xC0) == 0x80 {
                let cp = ((b as u32 & 0x1F) << 6) | (src[i + 1] as u32 & 0x3F);
                (cp, 2)
            } else {
                (0xFFFD, 1)
            }
        } else if b < 0xF0 {
            if i + 2 < src.len() && (src[i + 1] & 0xC0) == 0x80 && (src[i + 2] & 0xC0) == 0x80 {
                let cp = ((b as u32 & 0x0F) << 12)
                    | ((src[i + 1] as u32 & 0x3F) << 6)
                    | (src[i + 2] as u32 & 0x3F);
                (cp, 3)
            } else {
                (0xFFFD, 1)
            }
        } else {
            // 4-byte sequence
            if i + 3 < src.len()
                && (src[i + 1] & 0xC0) == 0x80
                && (src[i + 2] & 0xC0) == 0x80
                && (src[i + 3] & 0xC0) == 0x80
            {
                let cp = ((b as u32 & 0x07) << 18)
                    | ((src[i + 1] as u32 & 0x3F) << 12)
                    | ((src[i + 2] as u32 & 0x3F) << 6)
                    | (src[i + 3] as u32 & 0x3F);
                (cp, 4)
            } else {
                (0xFFFD, 1)
            }
        };
        codepoints.push(cp);
        i += consumed;
    }
    let mut out = Vec::with_capacity(codepoints.len() * 2);
    for cp in codepoints {
        if cp <= 0xFFFF {
            let bytes = (cp as u16).to_le_bytes();
            out.push(bytes[0]);
            out.push(bytes[1]);
        } else {
            let cp = cp - 0x10000;
            let high = 0xD800 + ((cp >> 10) as u16);
            let low = 0xDC00 + ((cp & 0x3FF) as u16);
            let bytes = high.to_le_bytes();
            out.push(bytes[0]);
            out.push(bytes[1]);
            let bytes = low.to_le_bytes();
            out.push(bytes[0]);
            out.push(bytes[1]);
        }
    }
    Some(out)
}

/// Append the UTF-8 encoding of `cp` to `out`. Caller must ensure
/// `cp <= 0x10FFFF` (the standard Unicode range). Used by
/// `decode_utf16le_to_utf8`.
fn encode_utf8_codepoint(out: &mut Vec<u8>, cp: u32) {
    if cp < 0x80 {
        out.push(cp as u8);
    } else if cp < 0x800 {
        out.push(0xC0 | (cp >> 6) as u8);
        out.push(0x80 | (cp & 0x3F) as u8);
    } else if cp < 0x10000 {
        out.push(0xE0 | (cp >> 12) as u8);
        out.push(0x80 | ((cp >> 6) & 0x3F) as u8);
        out.push(0x80 | (cp & 0x3F) as u8);
    } else {
        out.push(0xF0 | (cp >> 18) as u8);
        out.push(0x80 | ((cp >> 12) & 0x3F) as u8);
        out.push(0x80 | ((cp >> 6) & 0x3F) as u8);
        out.push(0x80 | (cp & 0x3F) as u8);
    }
}
