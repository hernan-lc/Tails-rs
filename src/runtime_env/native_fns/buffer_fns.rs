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
    let data = if let Some(first) = args.first() {
        match first {
            Value::String(s) => s.as_bytes().to_vec(),
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
                s.as_bytes().to_vec()
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
    let mut result = Vec::new();
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
