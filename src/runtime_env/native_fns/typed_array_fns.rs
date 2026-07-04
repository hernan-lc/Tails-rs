use crate::errors::{Error, Result};
use crate::objects::js_array::{TypedArray, TypedArrayType};
use crate::objects::Value;
use crate::vm::interpreter::Interpreter;

fn get_typed_array(interp: &Interpreter, idx: usize) -> Result<&TypedArray> {
    if let crate::vm::interpreter::HeapValue::TypedArray(arr) = &interp.heap[idx] {
        Ok(arr)
    } else {
        Err(Error::TypeError("Not a TypedArray".into()))
    }
}

fn get_typed_array_mut(interp: &mut Interpreter, idx: usize) -> Result<&mut TypedArray> {
    if let crate::vm::interpreter::HeapValue::TypedArray(arr) = &mut interp.heap[idx] {
        Ok(arr)
    } else {
        Err(Error::TypeError("Not a TypedArray".into()))
    }
}

fn typed_array_set_from_value(arr: &mut TypedArray, idx: usize, value: &Value) {
    match (&arr.kind, value) {
        (TypedArrayType::Int8Array, Value::Integer(v)) => arr.set_value(idx, *v as i8),
        (TypedArrayType::Int8Array, Value::Float(v)) => arr.set_value(idx, *v as i8),
        (TypedArrayType::Uint8Array | TypedArrayType::Uint8ClampedArray, Value::Integer(v)) => {
            arr.set_value(idx, *v as u8)
        }
        (TypedArrayType::Uint8Array | TypedArrayType::Uint8ClampedArray, Value::Float(v)) => {
            arr.set_value(idx, *v as u8)
        }
        (TypedArrayType::Int16Array, Value::Integer(v)) => arr.set_value(idx, *v as i16),
        (TypedArrayType::Int16Array, Value::Float(v)) => arr.set_value(idx, *v as i16),
        (TypedArrayType::Uint16Array, Value::Integer(v)) => arr.set_value(idx, *v as u16),
        (TypedArrayType::Uint16Array, Value::Float(v)) => arr.set_value(idx, *v as u16),
        (TypedArrayType::Int32Array, Value::Integer(v)) => arr.set_value(idx, *v as i32),
        (TypedArrayType::Int32Array, Value::Float(v)) => arr.set_value(idx, *v as i32),
        (TypedArrayType::Uint32Array, Value::Integer(v)) => arr.set_value(idx, *v as u32),
        (TypedArrayType::Uint32Array, Value::Float(v)) => arr.set_value(idx, *v as u32),
        (TypedArrayType::Float32Array, Value::Float(v)) => arr.set_value(idx, *v as f32),
        (TypedArrayType::Float32Array, Value::Integer(v)) => arr.set_value(idx, *v as f32),
        (TypedArrayType::Float64Array, Value::Float(v)) => arr.set_value(idx, *v),
        (TypedArrayType::Float64Array, Value::Integer(v)) => arr.set_value(idx, *v as f64),
        _ => {}
    }
}

fn typed_array_get_as_value(arr: &TypedArray, idx: usize) -> Value {
    match arr.kind {
        TypedArrayType::Int8Array => arr
            .get::<i8>(idx)
            .map(|v| Value::Integer(v as i64))
            .unwrap_or(Value::Undefined),
        TypedArrayType::Uint8Array | TypedArrayType::Uint8ClampedArray => arr
            .get::<u8>(idx)
            .map(|v| Value::Integer(v as i64))
            .unwrap_or(Value::Undefined),
        TypedArrayType::Int16Array => arr
            .get::<i16>(idx)
            .map(|v| Value::Integer(v as i64))
            .unwrap_or(Value::Undefined),
        TypedArrayType::Uint16Array => arr
            .get::<u16>(idx)
            .map(|v| Value::Integer(v as i64))
            .unwrap_or(Value::Undefined),
        TypedArrayType::Int32Array => arr
            .get::<i32>(idx)
            .map(|v| Value::Integer(v as i64))
            .unwrap_or(Value::Undefined),
        TypedArrayType::Uint32Array => arr
            .get::<u32>(idx)
            .map(|v| Value::Integer(v as i64))
            .unwrap_or(Value::Undefined),
        TypedArrayType::Float32Array => arr
            .get::<f32>(idx)
            .map(|v| Value::Float(v as f64))
            .unwrap_or(Value::Undefined),
        TypedArrayType::Float64Array => arr
            .get::<f64>(idx)
            .map(Value::Float)
            .unwrap_or(Value::Undefined),
        _ => Value::Undefined,
    }
}

fn parse_typed_array_type(name: &str) -> Option<TypedArrayType> {
    match name {
        "Int8Array" => Some(TypedArrayType::Int8Array),
        "Uint8Array" => Some(TypedArrayType::Uint8Array),
        "Uint8ClampedArray" => Some(TypedArrayType::Uint8ClampedArray),
        "Int16Array" => Some(TypedArrayType::Int16Array),
        "Uint16Array" => Some(TypedArrayType::Uint16Array),
        "Int32Array" => Some(TypedArrayType::Int32Array),
        "Uint32Array" => Some(TypedArrayType::Uint32Array),
        "Float32Array" => Some(TypedArrayType::Float32Array),
        "Float64Array" => Some(TypedArrayType::Float64Array),
        "BigInt64Array" => Some(TypedArrayType::BigInt64Array),
        "BigUint64Array" => Some(TypedArrayType::BigUint64Array),
        _ => None,
    }
}

pub(super) fn native_typed_array_constructor(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let type_name = match args.first() {
        Some(Value::String(s)) => s.as_str(),
        _ => {
            return Err(Error::TypeError(
                "TypedArray constructor requires type name".into(),
            ))
        }
    };

    let kind = parse_typed_array_type(type_name)
        .ok_or_else(|| Error::TypeError(format!("Unknown TypedArray type: {}", type_name)))?;

    let length = match args.get(1) {
        Some(Value::Integer(i)) => *i as usize,
        Some(Value::Float(f)) => *f as usize,
        Some(Value::Array(arr_idx)) => {
            if let crate::vm::interpreter::HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                arr.elements.len()
            } else {
                0
            }
        }
        _ => 0,
    };

    let typed_array = TypedArray::new(kind.clone(), length);
    let heap_idx = interp.heap.len();
    interp
        .heap
        .push(crate::vm::interpreter::HeapValue::TypedArray(typed_array));
    Ok(Value::TypedArray(heap_idx))
}

fn typed_array_constructor_impl(
    interp: &mut Interpreter,
    kind: TypedArrayType,
    args: &[Value],
) -> Result<Value> {
    match args.first() {
        Some(Value::Integer(length)) => {
            let typed_array = TypedArray::new(kind, *length as usize);
            let heap_idx = interp.heap.len();
            interp
                .heap
                .push(crate::vm::interpreter::HeapValue::TypedArray(typed_array));
            Ok(Value::TypedArray(heap_idx))
        }
        Some(Value::Float(length)) => {
            let typed_array = TypedArray::new(kind, *length as usize);
            let heap_idx = interp.heap.len();
            interp
                .heap
                .push(crate::vm::interpreter::HeapValue::TypedArray(typed_array));
            Ok(Value::TypedArray(heap_idx))
        }
        Some(Value::Array(arr_idx)) => {
            let elements: Vec<Value> =
                if let crate::vm::interpreter::HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                    arr.elements.clone()
                } else {
                    Vec::new()
                };
            let length = elements.len();
            let mut typed_array = TypedArray::new(kind.clone(), length);
            for (i, elem) in elements.iter().enumerate() {
                typed_array_set_from_value(&mut typed_array, i, elem);
            }
            let heap_idx = interp.heap.len();
            interp
                .heap
                .push(crate::vm::interpreter::HeapValue::TypedArray(typed_array));
            Ok(Value::TypedArray(heap_idx))
        }
        _ => {
            let typed_array = TypedArray::new(kind, 0);
            let heap_idx = interp.heap.len();
            interp
                .heap
                .push(crate::vm::interpreter::HeapValue::TypedArray(typed_array));
            Ok(Value::TypedArray(heap_idx))
        }
    }
}

pub(super) fn native_int8_array_constructor(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    typed_array_constructor_impl(interp, TypedArrayType::Int8Array, args)
}

pub(super) fn native_uint8_array_constructor(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    typed_array_constructor_impl(interp, TypedArrayType::Uint8Array, args)
}

pub(super) fn native_uint8_clamped_array_constructor(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    typed_array_constructor_impl(interp, TypedArrayType::Uint8ClampedArray, args)
}

pub(super) fn native_int16_array_constructor(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    typed_array_constructor_impl(interp, TypedArrayType::Int16Array, args)
}

pub(super) fn native_uint16_array_constructor(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    typed_array_constructor_impl(interp, TypedArrayType::Uint16Array, args)
}

pub(super) fn native_int32_array_constructor(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    typed_array_constructor_impl(interp, TypedArrayType::Int32Array, args)
}

pub(super) fn native_uint32_array_constructor(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    typed_array_constructor_impl(interp, TypedArrayType::Uint32Array, args)
}

pub(super) fn native_float32_array_constructor(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    typed_array_constructor_impl(interp, TypedArrayType::Float32Array, args)
}

pub(super) fn native_float64_array_constructor(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    typed_array_constructor_impl(interp, TypedArrayType::Float64Array, args)
}

pub(super) fn native_bigint64_array_constructor(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    typed_array_constructor_impl(interp, TypedArrayType::BigInt64Array, args)
}

pub(super) fn native_biguint64_array_constructor(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    typed_array_constructor_impl(interp, TypedArrayType::BigUint64Array, args)
}

pub(super) fn native_typed_array_from(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let type_name = match args.first() {
        Some(Value::String(s)) => s.as_str(),
        _ => {
            return Err(Error::TypeError(
                "TypedArray.from requires type name".into(),
            ))
        }
    };

    let kind = parse_typed_array_type(type_name)
        .ok_or_else(|| Error::TypeError(format!("Unknown TypedArray type: {}", type_name)))?;

    let iterable = args.get(1).cloned().unwrap_or(Value::Undefined);
    let map_fn = args.get(2).cloned();

    let mut elements = Vec::new();

    if let Value::Array(arr_idx) = &iterable {
        // Clone elements to avoid borrow issues
        let arr_elements: Vec<Value> =
            if let crate::vm::interpreter::HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                arr.elements.clone()
            } else {
                Vec::new()
            };
        elements.reserve(arr_elements.len());

        for elem in arr_elements {
            let val = if let Some(map_fn) = &map_fn {
                interp.call_value(map_fn, &Value::Undefined, std::slice::from_ref(&elem))?
            } else {
                elem.clone()
            };
            elements.push(val);
        }
    }

    let length = elements.len();
    let typed_array = TypedArray::new(kind.clone(), length);

    let mut arr = typed_array;
    for (i, elem) in elements.iter().enumerate() {
        typed_array_set_from_value(&mut arr, i, elem);
    }

    let heap_idx = interp.heap.len();
    interp
        .heap
        .push(crate::vm::interpreter::HeapValue::TypedArray(arr));
    Ok(Value::TypedArray(heap_idx))
}

pub(super) fn native_typed_array_of(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let type_name = match args.first() {
        Some(Value::String(s)) => s.as_str(),
        _ => return Err(Error::TypeError("TypedArray.of requires type name".into())),
    };

    let kind = parse_typed_array_type(type_name)
        .ok_or_else(|| Error::TypeError(format!("Unknown TypedArray type: {}", type_name)))?;

    let length = args.len().saturating_sub(1);
    let typed_array = TypedArray::new(kind.clone(), length);

    let mut arr = typed_array;
    for (i, elem) in args.iter().skip(1).enumerate() {
        typed_array_set_from_value(&mut arr, i, elem);
    }

    let heap_idx = interp.heap.len();
    interp
        .heap
        .push(crate::vm::interpreter::HeapValue::TypedArray(arr));
    Ok(Value::TypedArray(heap_idx))
}

pub(super) fn native_typed_array_get(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let idx = match this {
        Value::TypedArray(idx) => *idx,
        _ => return Err(Error::TypeError("Not a TypedArray".into())),
    };

    let index = match args.first() {
        Some(Value::Integer(i)) => *i as usize,
        Some(Value::Float(f)) => *f as usize,
        _ => return Ok(Value::Undefined),
    };

    let arr = get_typed_array(interp, idx)?;
    Ok(typed_array_get_as_value(arr, index))
}

pub(super) fn native_typed_array_set(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let idx = match this {
        Value::TypedArray(idx) => *idx,
        _ => return Err(Error::TypeError("Not a TypedArray".into())),
    };

    match args.first() {
        Some(Value::TypedArray(src_idx)) => {
            let src_idx = *src_idx;
            let (src_kind, src_byte_length) =
                if let crate::vm::interpreter::HeapValue::TypedArray(ta) = &interp.heap[src_idx] {
                    (ta.kind.clone(), ta.byte_length)
                } else {
                    return Err(Error::TypeError("Not a TypedArray".into()));
                };
            let offset = match args.get(1) {
                Some(Value::Integer(i)) => *i as usize,
                Some(Value::Float(f)) => *f as usize,
                _ => 0,
            };
            let element_size = TypedArray::element_size(&src_kind);
            let src_elems = src_byte_length / element_size;
            let dst_byte_length = {
                let arr = get_typed_array(interp, idx)?;
                arr.byte_length
            };
            let byte_offset = offset * element_size;
            let count = std::cmp::min(
                (dst_byte_length - byte_offset) / element_size,
                src_elems - offset,
            );
            let mut src_values: Vec<i64> = Vec::with_capacity(count);
            if let crate::vm::interpreter::HeapValue::TypedArray(src_arr) = &interp.heap[src_idx] {
                for i in 0..count {
                    let val = src_arr.get::<i64>(i + offset).unwrap_or(0);
                    src_values.push(val);
                }
            }
            let arr = get_typed_array_mut(interp, idx)?;
            if byte_offset + count * element_size <= arr.byte_length {
                for (i, val) in src_values.iter().enumerate() {
                    // Use src_kind (source type) to interpret the i64 values
                    match src_kind {
                        TypedArrayType::Int8Array => arr.set_value(i + offset, *val as i8),
                        TypedArrayType::Uint8Array | TypedArrayType::Uint8ClampedArray => {
                            arr.set_value(i + offset, *val as u8)
                        }
                        TypedArrayType::Int16Array => arr.set_value(i + offset, *val as i16),
                        TypedArrayType::Uint16Array => arr.set_value(i + offset, *val as u16),
                        TypedArrayType::Int32Array => arr.set_value(i + offset, *val as i32),
                        TypedArrayType::Uint32Array => arr.set_value(i + offset, *val as u32),
                        TypedArrayType::Float32Array => arr.set_value(i + offset, *val as f32),
                        TypedArrayType::Float64Array => arr.set_value(i + offset, *val as f64),
                        _ => {}
                    }
                }
            }
        }
        Some(Value::Array(arr_idx)) => {
            let arr_idx = *arr_idx;
            let elements: Vec<Value> =
                if let crate::vm::interpreter::HeapValue::Array(crate::vm::interpreter::JsArray {
                    elements: elems,
                }) = &interp.heap[arr_idx]
                {
                    elems.clone()
                } else {
                    Vec::new()
                };
            let dst_kind = {
                let arr = get_typed_array(interp, idx)?;
                arr.kind.clone()
            };
            let dst_byte_length = {
                let arr = get_typed_array(interp, idx)?;
                arr.byte_length
            };
            let offset = match args.get(1) {
                Some(Value::Integer(i)) => *i as usize,
                Some(Value::Float(f)) => *f as usize,
                _ => 0,
            };
            let arr = get_typed_array_mut(interp, idx)?;
            for (i, elem) in elements.iter().enumerate() {
                let target_idx = offset + i;
                if target_idx * TypedArray::element_size(&dst_kind) >= dst_byte_length {
                    break;
                }
                typed_array_set_from_value(arr, target_idx, elem);
            }
        }
        _ => {
            // Single value: set(index, value)
            let index = match args.first() {
                Some(Value::Integer(i)) => *i as usize,
                Some(Value::Float(f)) => *f as usize,
                _ => return Err(Error::TypeError("Invalid index".into())),
            };

            let value = args.get(1).cloned().unwrap_or(Value::Undefined);
            let arr = get_typed_array_mut(interp, idx)?;
            typed_array_set_from_value(arr, index, &value);
        }
    }

    Ok(Value::Undefined)
}

pub(super) fn native_typed_array_length(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let idx = match this {
        Value::TypedArray(idx) => *idx,
        _ => return Err(Error::TypeError("Not a TypedArray".into())),
    };

    let arr = get_typed_array(interp, idx)?;
    let element_size = TypedArray::element_size(&arr.kind);
    let length = arr.byte_length / element_size;
    Ok(Value::Float(length as f64))
}

pub(super) fn native_typed_array_byte_length(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let idx = match this {
        Value::TypedArray(idx) => *idx,
        _ => return Err(Error::TypeError("Not a TypedArray".into())),
    };

    let arr = get_typed_array(interp, idx)?;
    Ok(Value::Float(arr.byte_length as f64))
}

pub(super) fn native_typed_array_byte_offset(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let idx = match this {
        Value::TypedArray(idx) => *idx,
        _ => return Err(Error::TypeError("Not a TypedArray".into())),
    };

    let arr = get_typed_array(interp, idx)?;
    Ok(Value::Float(arr.byte_offset as f64))
}

pub(super) fn native_typed_array_subarray(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let idx = match this {
        Value::TypedArray(idx) => *idx,
        _ => return Err(Error::TypeError("Not a TypedArray".into())),
    };

    let begin = match args.first() {
        Some(Value::Integer(i)) => *i as usize,
        Some(Value::Float(f)) => *f as usize,
        _ => 0,
    };

    let end = match args.get(1) {
        Some(Value::Integer(i)) => *i as usize,
        Some(Value::Float(f)) => *f as usize,
        _ => {
            let arr = get_typed_array(interp, idx)?;
            let element_size = TypedArray::element_size(&arr.kind);
            arr.byte_length / element_size
        }
    };

    let arr = get_typed_array(interp, idx)?;
    let element_size = TypedArray::element_size(&arr.kind);
    let byte_begin = begin * element_size;
    let byte_end = end * element_size;
    let new_byte_length = byte_end.saturating_sub(byte_begin);

    let new_buffer = arr.buffer[byte_begin..byte_begin + new_byte_length].to_vec();
    let new_arr = TypedArray::from_buffer(arr.kind.clone(), new_buffer, 0);

    let heap_idx = interp.heap.len();
    interp
        .heap
        .push(crate::vm::interpreter::HeapValue::TypedArray(new_arr));
    Ok(Value::TypedArray(heap_idx))
}

pub(super) fn native_typed_array_slice(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let idx = match this {
        Value::TypedArray(idx) => *idx,
        _ => return Err(Error::TypeError("Not a TypedArray".into())),
    };

    let begin = match args.first() {
        Some(Value::Integer(i)) => *i as usize,
        Some(Value::Float(f)) => *f as usize,
        _ => 0,
    };

    let end = match args.get(1) {
        Some(Value::Integer(i)) => *i as usize,
        Some(Value::Float(f)) => *f as usize,
        _ => {
            let arr = get_typed_array(interp, idx)?;
            let element_size = TypedArray::element_size(&arr.kind);
            arr.byte_length / element_size
        }
    };

    let arr = get_typed_array(interp, idx)?;
    let element_size = TypedArray::element_size(&arr.kind);
    let byte_begin = begin * element_size;
    let byte_end = end * element_size;
    let new_byte_length = byte_end.saturating_sub(byte_begin);

    let new_buffer = arr.buffer[byte_begin..byte_begin + new_byte_length].to_vec();
    let new_arr = TypedArray::from_buffer(arr.kind.clone(), new_buffer, 0);

    let heap_idx = interp.heap.len();
    interp
        .heap
        .push(crate::vm::interpreter::HeapValue::TypedArray(new_arr));
    Ok(Value::TypedArray(heap_idx))
}
