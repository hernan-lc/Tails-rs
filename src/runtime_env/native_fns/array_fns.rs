use crate::errors::{Error, Result};
use crate::objects::Value;
use crate::vm::interpreter::Interpreter;

use super::helpers::{get_array_elements, to_f64, to_string_value};

pub(super) fn native_array_push(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    if let Value::Array(arr_idx) = this {
        if let crate::vm::interpreter::HeapValue::Array(arr) = &mut interp.heap[*arr_idx] {
            for arg in args {
                arr.elements.push(arg.clone());
            }
            return Ok(Value::Float(arr.elements.len() as f64));
        }
    }
    Ok(Value::Float(0.0))
}

pub(super) fn native_array_pop(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    if let Value::Array(arr_idx) = this {
        if let crate::vm::interpreter::HeapValue::Array(arr) = &mut interp.heap[*arr_idx] {
            return Ok(arr.elements.pop().unwrap_or(Value::Undefined));
        }
    }
    Ok(Value::Undefined)
}

pub(super) fn native_array_shift(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    if let Value::Array(arr_idx) = this {
        if let crate::vm::interpreter::HeapValue::Array(arr) = &mut interp.heap[*arr_idx] {
            if arr.elements.is_empty() {
                return Ok(Value::Undefined);
            }
            return Ok(arr.elements.remove(0));
        }
    }
    Ok(Value::Undefined)
}

pub(super) fn native_array_unshift(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    if let Value::Array(arr_idx) = this {
        if let crate::vm::interpreter::HeapValue::Array(arr) = &mut interp.heap[*arr_idx] {
            for (i, arg) in args.iter().enumerate() {
                arr.elements.insert(i, arg.clone());
            }
            return Ok(Value::Float(arr.elements.len() as f64));
        }
    }
    Ok(Value::Float(0.0))
}

pub(super) fn native_array_slice(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let elements = get_array_elements(interp, this)?;
    let start_raw = args.first().map(to_f64).unwrap_or(0.0) as i64;
    let end_raw = args.get(1).map(to_f64).unwrap_or(elements.len() as f64) as i64;

    let len = elements.len() as i64;
    let start = if start_raw < 0 {
        (len + start_raw).max(0)
    } else {
        start_raw.min(len)
    } as usize;
    let end = if end_raw < 0 {
        (len + end_raw).max(0)
    } else {
        end_raw.min(len)
    } as usize;

    let sliced = if start < end {
        elements[start..end].to_vec()
    } else {
        Vec::new()
    };
    let heap_idx = interp.heap.len();
    interp.heap.push(crate::vm::interpreter::HeapValue::Array(
        crate::vm::interpreter::JsArray { elements: sliced },
    ));
    Ok(Value::Array(heap_idx))
}

pub(super) fn native_array_splice(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    if let Value::Array(arr_idx) = this {
        let start_raw = args.first().map(to_f64).unwrap_or(0.0) as i64;
        let delete_count_raw = args.get(1).map(to_f64).unwrap_or(0.0) as i64;

        if let crate::vm::interpreter::HeapValue::Array(arr) = &mut interp.heap[*arr_idx] {
            let len = arr.elements.len() as i64;
            let start = if start_raw < 0 {
                (len + start_raw).max(0)
            } else {
                start_raw.min(len)
            } as usize;
            let delete_count = delete_count_raw.max(0).min(len - start as i64) as usize;

            let removed: Vec<Value> = arr.elements.drain(start..start + delete_count).collect();
            let new_items: Vec<Value> = args[2..].to_vec();
            for (i, item) in new_items.into_iter().enumerate() {
                arr.elements.insert(start + i, item);
            }

            let heap_idx = interp.heap.len();
            interp.heap.push(crate::vm::interpreter::HeapValue::Array(
                crate::vm::interpreter::JsArray { elements: removed },
            ));
            return Ok(Value::Array(heap_idx));
        }
    }
    let heap_idx = interp.heap.len();
    interp.heap.push(crate::vm::interpreter::HeapValue::Array(
        crate::vm::interpreter::JsArray {
            elements: Vec::new(),
        },
    ));
    Ok(Value::Array(heap_idx))
}

pub(super) fn native_array_index_of(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let elements = get_array_elements(interp, this)?;
    let search = args.first().cloned().unwrap_or(Value::Undefined);
    let from = args.get(1).map(|v| to_f64(v) as usize).unwrap_or(0);
    for (i, elem) in elements.iter().enumerate() {
        if i >= from && elem == &search {
            return Ok(Value::Float(i as f64));
        }
    }
    Ok(Value::Float(-1.0))
}

pub(super) fn native_array_includes(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let elements = get_array_elements(interp, this)?;
    let search = args.first().cloned().unwrap_or(Value::Undefined);
    Ok(Value::Boolean(elements.contains(&search)))
}

pub(super) fn native_array_find(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let elements = get_array_elements(interp, this)?;
    let callback = args.first().cloned().unwrap_or(Value::Undefined);
    for (i, elem) in elements.iter().enumerate() {
        let call_args = vec![elem.clone(), Value::Integer(i as i64), this.clone()];
        let result = interp.call_value(&callback, &Value::Undefined, &call_args)?;
        if interp.is_truthy(&result) {
            return Ok(elem.clone());
        }
    }
    Ok(Value::Undefined)
}

pub(super) fn native_array_find_index(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let elements = get_array_elements(interp, this)?;
    let callback = args.first().cloned().unwrap_or(Value::Undefined);
    for (i, elem) in elements.iter().enumerate() {
        let call_args = vec![elem.clone(), Value::Integer(i as i64), this.clone()];
        let result = interp.call_value(&callback, &Value::Undefined, &call_args)?;
        if interp.is_truthy(&result) {
            return Ok(Value::Float(i as f64));
        }
    }
    Ok(Value::Float(-1.0))
}

pub(super) fn native_array_map(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let elements = get_array_elements(interp, this)?;
    let callback = args.first().cloned().unwrap_or(Value::Undefined);
    let mut results = Vec::with_capacity(elements.len());
    for (i, elem) in elements.iter().enumerate() {
        let call_args = vec![elem.clone(), Value::Integer(i as i64), this.clone()];
        let result = interp.call_value(&callback, &Value::Undefined, &call_args)?;
        results.push(result);
    }
    let heap_idx = interp.heap.len();
    interp.heap.push(crate::vm::interpreter::HeapValue::Array(
        crate::vm::interpreter::JsArray { elements: results },
    ));
    Ok(Value::Array(heap_idx))
}

pub(super) fn native_array_filter(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let elements = get_array_elements(interp, this)?;
    let callback = args.first().cloned().unwrap_or(Value::Undefined);
    let mut results = Vec::with_capacity(elements.len());
    for (i, elem) in elements.iter().enumerate() {
        let call_args = vec![elem.clone(), Value::Integer(i as i64), this.clone()];
        let result = interp.call_value(&callback, &Value::Undefined, &call_args)?;
        if interp.is_truthy(&result) {
            results.push(elem.clone());
        }
    }
    let heap_idx = interp.heap.len();
    interp.heap.push(crate::vm::interpreter::HeapValue::Array(
        crate::vm::interpreter::JsArray { elements: results },
    ));
    Ok(Value::Array(heap_idx))
}

pub(super) fn native_array_reduce(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let elements = get_array_elements(interp, this)?;
    let callback = args.first().cloned().unwrap_or(Value::Undefined);
    let has_init = args.len() > 1;
    let mut acc = if has_init {
        args[1].clone()
    } else {
        Value::Undefined
    };
    let start_idx = if has_init { 0 } else { 1 };

    if !has_init && elements.is_empty() {
        return Err(Error::TypeError(
            "Reduce of empty array with no initial value".into(),
        ));
    }

    if !has_init {
        acc = elements[0].clone();
    }

    for (i, elem) in elements.iter().enumerate().skip(start_idx) {
        let call_args = vec![acc, elem.clone(), Value::Integer(i as i64), this.clone()];
        acc = interp.call_value(&callback, &Value::Undefined, &call_args)?;
    }
    Ok(acc)
}

pub(super) fn native_array_for_each(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let elements = get_array_elements(interp, this)?;
    let callback = args.first().cloned().unwrap_or(Value::Undefined);
    for (i, elem) in elements.iter().enumerate() {
        let call_args = vec![elem.clone(), Value::Integer(i as i64), this.clone()];
        interp.call_value(&callback, &Value::Undefined, &call_args)?;
    }
    Ok(Value::Undefined)
}

pub(super) fn native_array_some(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let elements = get_array_elements(interp, this)?;
    let callback = args.first().cloned().unwrap_or(Value::Undefined);
    for (i, elem) in elements.iter().enumerate() {
        let call_args = vec![elem.clone(), Value::Integer(i as i64), this.clone()];
        let result = interp.call_value(&callback, &Value::Undefined, &call_args)?;
        if interp.is_truthy(&result) {
            return Ok(Value::Boolean(true));
        }
    }
    Ok(Value::Boolean(false))
}

pub(super) fn native_array_every(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let elements = get_array_elements(interp, this)?;
    let callback = args.first().cloned().unwrap_or(Value::Undefined);
    for (i, elem) in elements.iter().enumerate() {
        let call_args = vec![elem.clone(), Value::Integer(i as i64), this.clone()];
        let result = interp.call_value(&callback, &Value::Undefined, &call_args)?;
        if !interp.is_truthy(&result) {
            return Ok(Value::Boolean(false));
        }
    }
    Ok(Value::Boolean(true))
}

pub(super) fn native_array_join(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let elements = get_array_elements(interp, this)?;
    let sep = match args.first() {
        Some(Value::String(s)) => s.clone(),
        Some(v) => to_string_value(interp, v),
        None => ",".to_string(),
    };
    let parts: Vec<String> = elements
        .iter()
        .map(|e| to_string_value(interp, e))
        .collect();
    Ok(Value::String(parts.join(&sep)))
}

pub(super) fn native_array_reverse(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    if let Value::Array(arr_idx) = this {
        if let crate::vm::interpreter::HeapValue::Array(arr) = &mut interp.heap[*arr_idx] {
            arr.elements.reverse();
        }
    }
    Ok(this.clone())
}

pub(super) fn native_array_sort(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    if let Value::Array(arr_idx) = this {
        let elements: Vec<Value> =
            if let crate::vm::interpreter::HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                arr.elements.clone()
            } else {
                return Ok(this.clone());
            };
        let mut indexed: Vec<(String, Value)> = elements
            .iter()
            .map(|e| (to_string_value(interp, e), e.clone()))
            .collect();
        indexed.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        let sorted: Vec<Value> = indexed.into_iter().map(|(_, v)| v).collect();
        if let crate::vm::interpreter::HeapValue::Array(arr) = &mut interp.heap[*arr_idx] {
            arr.elements = sorted;
        }
    }
    Ok(this.clone())
}

pub(super) fn native_array_concat(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let mut result = get_array_elements(interp, this)?;
    for arg in args {
        match arg {
            Value::Array(arr_idx) => {
                if let crate::vm::interpreter::HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                    result.extend(arr.elements.iter().cloned());
                }
            }
            other => result.push(other.clone()),
        }
    }
    let heap_idx = interp.heap.len();
    interp.heap.push(crate::vm::interpreter::HeapValue::Array(
        crate::vm::interpreter::JsArray { elements: result },
    ));
    Ok(Value::Array(heap_idx))
}

pub(super) fn native_array_flat(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let depth = args.first().map(|v| to_f64(v) as i64).unwrap_or(1);
    fn flat_recursive(interp: &Interpreter, elements: &[Value], depth: i64) -> Vec<Value> {
        let mut result = Vec::new();
        for elem in elements {
            if depth > 0 {
                if let Value::Array(arr_idx) = elem {
                    if let crate::vm::interpreter::HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                        result.extend(flat_recursive(interp, &arr.elements, depth - 1));
                        continue;
                    }
                }
            }
            result.push(elem.clone());
        }
        result
    }
    let elements = get_array_elements(interp, this)?;
    let flat = flat_recursive(interp, &elements, depth);
    let heap_idx = interp.heap.len();
    interp.heap.push(crate::vm::interpreter::HeapValue::Array(
        crate::vm::interpreter::JsArray { elements: flat },
    ));
    Ok(Value::Array(heap_idx))
}

fn normalize_index(index: i64, len: i64) -> i64 {
    if index < 0 {
        (len + index).max(0)
    } else {
        index.min(len)
    }
}

pub(super) fn native_array_copy_within(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    if let Value::Array(arr_idx) = this {
        if let crate::vm::interpreter::HeapValue::Array(arr) = &mut interp.heap[*arr_idx] {
            let len = arr.elements.len() as i64;
            let target = normalize_index(args.first().map(|v| to_f64(v) as i64).unwrap_or(0), len);
            let start = normalize_index(args.get(1).map(|v| to_f64(v) as i64).unwrap_or(0), len);
            let end = normalize_index(args.get(2).map(|v| to_f64(v) as i64).unwrap_or(len), len);

            if target < start {
                for i in start..end {
                    if i >= 0 && i < len && target + (i - start) >= 0 && target + (i - start) < len
                    {
                        let val = arr.elements[i as usize].clone();
                        arr.elements[(target + (i - start)) as usize] = val;
                    }
                }
            } else {
                for i in (start..end).rev() {
                    if i >= 0 && i < len && target + (i - start) >= 0 && target + (i - start) < len
                    {
                        let val = arr.elements[i as usize].clone();
                        arr.elements[(target + (i - start)) as usize] = val;
                    }
                }
            }
        }
    }
    Ok(this.clone())
}

pub(super) fn native_array_fill(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let value = args.first().cloned().unwrap_or(Value::Undefined);
    if let Value::Array(arr_idx) = this {
        if let crate::vm::interpreter::HeapValue::Array(arr) = &mut interp.heap[*arr_idx] {
            let len = arr.elements.len() as i64;
            let start = normalize_index(args.get(1).map(|v| to_f64(v) as i64).unwrap_or(0), len);
            let end = normalize_index(args.get(2).map(|v| to_f64(v) as i64).unwrap_or(len), len);
            for i in start..end {
                if i >= 0 && i < len {
                    arr.elements[i as usize] = value.clone();
                }
            }
        }
    }
    Ok(this.clone())
}

pub(super) fn native_array_find_last(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let callback = args.first().cloned().unwrap_or(Value::Undefined);
    let elements = get_array_elements(interp, this)?;
    for (i, elem) in elements.iter().enumerate().rev() {
        let result = interp.call_value(
            &callback,
            &Value::Undefined,
            &[elem.clone(), Value::Integer(i as i64), this.clone()],
        )?;
        if super::helpers::is_truthy(&result) {
            return Ok(elem.clone());
        }
    }
    Ok(Value::Undefined)
}

pub(super) fn native_array_find_last_index(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let callback = args.first().cloned().unwrap_or(Value::Undefined);
    let elements = get_array_elements(interp, this)?;
    for (i, elem) in elements.iter().enumerate().rev() {
        let result = interp.call_value(
            &callback,
            &Value::Undefined,
            &[elem.clone(), Value::Integer(i as i64), this.clone()],
        )?;
        if super::helpers::is_truthy(&result) {
            return Ok(Value::Integer(i as i64));
        }
    }
    Ok(Value::Float(-1.0))
}

pub(super) fn native_array_flat_map(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let callback = args.first().cloned().unwrap_or(Value::Undefined);
    let elements = get_array_elements(interp, this)?;
    let mut result = Vec::new();
    for (i, elem) in elements.iter().enumerate() {
        let mapped = interp.call_value(
            &callback,
            &Value::Undefined,
            &[elem.clone(), Value::Integer(i as i64), this.clone()],
        )?;
        if let Value::Array(arr_idx) = &mapped {
            if let crate::vm::interpreter::HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                result.extend(arr.elements.iter().cloned());
                continue;
            }
        }
        result.push(mapped);
    }
    let heap_idx = interp.heap.len();
    interp.heap.push(crate::vm::interpreter::HeapValue::Array(
        crate::vm::interpreter::JsArray { elements: result },
    ));
    Ok(Value::Array(heap_idx))
}

pub(super) fn native_array_last_index_of(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let search = args.first().cloned().unwrap_or(Value::Undefined);
    let elements = get_array_elements(interp, this)?;
    let from_index = args
        .get(1)
        .map(|v| to_f64(v) as i64)
        .unwrap_or(elements.len() as i64 - 1);
    let len = elements.len() as i64;
    let start = if from_index < 0 {
        len + from_index
    } else {
        from_index.min(len - 1)
    };
    for i in (0..=start).rev() {
        if i >= 0 && (i as usize) < elements.len() && elements[i as usize] == search {
            return Ok(Value::Integer(i));
        }
    }
    Ok(Value::Float(-1.0))
}

pub(super) fn native_array_is_array(
    _interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    Ok(Value::Boolean(matches!(
        args.first(),
        Some(Value::Array(_))
    )))
}

pub(super) fn native_array_from(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let source = args.first().cloned().unwrap_or(Value::Undefined);
    let map_fn = args.get(1).cloned();
    let mut elements = Vec::new();

    match &source {
        Value::Array(arr_idx) => {
            let source_elements =
                if let crate::vm::interpreter::HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                    arr.elements.clone()
                } else {
                    Vec::new()
                };
            for (i, elem) in source_elements.iter().enumerate() {
                if let Some(ref callback) = map_fn {
                    let mapped = interp.call_value(
                        callback,
                        &Value::Undefined,
                        &[elem.clone(), Value::Integer(i as i64)],
                    )?;
                    elements.push(mapped);
                } else {
                    elements.push(elem.clone());
                }
            }
        }
        Value::String(s) => {
            for (i, c) in s.chars().enumerate() {
                let val = Value::String(c.to_string());
                if let Some(ref callback) = map_fn {
                    let mapped = interp.call_value(
                        callback,
                        &Value::Undefined,
                        &[val, Value::Integer(i as i64)],
                    )?;
                    elements.push(mapped);
                } else {
                    elements.push(val);
                }
            }
        }
        _ => {
            // Other types not supported directly - return empty array
        }
    }

    let heap_idx = interp.heap.len();
    interp.heap.push(crate::vm::interpreter::HeapValue::Array(
        crate::vm::interpreter::JsArray { elements },
    ));
    Ok(Value::Array(heap_idx))
}

pub(super) fn native_array_of(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let elements = args.to_vec();
    let heap_idx = interp.heap.len();
    interp.heap.push(crate::vm::interpreter::HeapValue::Array(
        crate::vm::interpreter::JsArray { elements },
    ));
    Ok(Value::Array(heap_idx))
}
