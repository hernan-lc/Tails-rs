use crate::errors::{Error, Result};
use crate::objects::Value;
use crate::vm::interpreter::Interpreter;

pub type NativeFn = fn(&mut Interpreter, &Value, &[Value]) -> Result<Value>;

pub static NATIVE_TABLE: &[NativeFn] = &[
    native_console_log,
    native_console_warn,
    native_console_error,
    native_console_info,
    native_object_keys,
    native_object_values,
    native_object_entries,
    native_object_assign,
    native_json_parse,
    native_json_stringify,
    native_parse_int,
    native_parse_float,
    native_is_nan,
    native_is_finite,
    native_set_timeout,
    native_set_interval,
    native_clear_timeout,
    native_clear_interval,
    native_math_abs,
    native_math_floor,
    native_math_ceil,
    native_math_round,
    native_math_min,
    native_math_max,
    native_math_random,
    native_math_pow,
    native_math_sqrt,
    native_math_log,
    native_math_sin,
    native_math_cos,
    native_math_tan,
    native_array_push,
    native_array_pop,
    native_array_shift,
    native_array_unshift,
    native_array_slice,
    native_array_splice,
    native_array_index_of,
    native_array_includes,
    native_array_find,
    native_array_find_index,
    native_array_map,
    native_array_filter,
    native_array_reduce,
    native_array_for_each,
    native_array_some,
    native_array_every,
    native_array_join,
    native_array_reverse,
    native_array_sort,
    native_array_concat,
    native_array_flat,
    native_string_char_at,
    native_string_char_code_at,
    native_string_slice,
    native_string_substring,
    native_string_index_of,
    native_string_includes,
    native_string_replace,
    native_string_split,
    native_string_trim,
    native_string_to_lower_case,
    native_string_to_upper_case,
    native_string_starts_with,
    native_string_ends_with,
    native_string_repeat,
    native_string_pad_start,
    native_string_pad_end,
    native_number_parse_int,
    native_number_parse_float,
    native_number_is_nan,
    native_number_is_finite,
    native_error_constructor,
    native_type_error_constructor,
    native_reference_error_constructor,
    native_syntax_error_constructor,
    native_range_error_constructor,
    native_promise_constructor,
    native_promise_then,
    native_promise_catch,
    native_promise_finally,
    native_promise_resolve,
    native_promise_reject,
    native_promise_all,
    native_promise_race,
];

// ---- Helper ----

fn to_f64(v: &Value) -> f64 {
    match v {
        Value::Integer(n) => *n as f64,
        Value::Float(n) => *n,
        Value::Boolean(b) => if *b { 1.0 } else { 0.0 },
        Value::Null => 0.0,
        Value::Undefined => f64::NAN,
        Value::String(s) => s.parse::<f64>().unwrap_or(f64::NAN),
        _ => f64::NAN,
    }
}

fn to_i64(v: &Value) -> i64 {
    match v {
        Value::Integer(n) => *n,
        Value::Float(n) => *n as i64,
        Value::Boolean(b) => if *b { 1 } else { 0 },
        _ => 0,
    }
}

fn to_string_value(interp: &Interpreter, v: &Value) -> String {
    match v {
        Value::Undefined => "undefined".to_string(),
        Value::Null => "null".to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Integer(n) => n.to_string(),
        Value::Float(n) => {
            if *n == (*n as i64) as f64 {
                (*n as i64).to_string()
            } else {
                n.to_string()
            }
        }
        Value::String(s) => s.clone(),
        Value::Array(arr_idx) => {
            if let crate::vm::interpreter::HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                let parts: Vec<String> = arr.elements.iter().map(|e| to_string_value(interp, e)).collect();
                parts.join(",")
            } else {
                "[Array]".to_string()
            }
        }
        Value::Object(obj_idx) => {
            if let crate::vm::interpreter::HeapValue::Object(obj) = &interp.heap[*obj_idx] {
                let parts: Vec<String> = obj.properties.iter()
                    .map(|(k, v)| format!("\"{}\":{}", k, to_json_value(interp, v)))
                    .collect();
                format!("{{{}}}", parts.join(","))
            } else {
                "[Object]".to_string()
            }
        }
        _ => "[Function]".to_string(),
    }
}

fn to_display_string(interp: &Interpreter, v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        other => to_string_value(interp, other),
    }
}

fn to_json_value(interp: &Interpreter, v: &Value) -> String {
    match v {
        Value::Null => "null".to_string(),
        Value::Undefined => "undefined".to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Integer(n) => n.to_string(),
        Value::Float(n) => {
            if n.is_nan() { "null".to_string() }
            else if *n == (*n as i64) as f64 { (*n as i64).to_string() }
            else { n.to_string() }
        }
        Value::String(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
        Value::Array(arr_idx) => {
            if let crate::vm::interpreter::HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                let parts: Vec<String> = arr.elements.iter().map(|e| to_json_value(interp, e)).collect();
                format!("[{}]", parts.join(","))
            } else { "[]".to_string() }
        }
        Value::Object(obj_idx) => {
            if let crate::vm::interpreter::HeapValue::Object(obj) = &interp.heap[*obj_idx] {
                let parts: Vec<String> = obj.properties.iter()
                    .map(|(k, v)| format!("\"{}\":{}", k, to_json_value(interp, v)))
                    .collect();
                format!("{{{}}}", parts.join(","))
            } else { "{}".to_string() }
        }
        _ => "null".to_string(),
    }
}

fn from_json_value(interp: &mut Interpreter, val: serde_json::Value) -> Value {
    match val {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Boolean(b),
        serde_json::Value::Number(n) => {
            Value::Float(n.as_f64().unwrap_or(f64::NAN))
        }
        serde_json::Value::String(s) => Value::String(s),
        serde_json::Value::Array(arr) => {
            let elems: Vec<Value> = arr.into_iter().map(|v| from_json_value(interp, v)).collect();
            let heap_idx = interp.heap.len();
            interp.heap.push(crate::vm::interpreter::HeapValue::Array(
                crate::vm::interpreter::JsArray { elements: elems },
            ));
            Value::Array(heap_idx)
        }
        serde_json::Value::Object(map) => {
            let heap_idx = interp.heap.len();
            let mut props = std::collections::HashMap::new();
            for (k, v) in map {
                props.insert(k, from_json_value(interp, v));
            }
            interp.heap.push(crate::vm::interpreter::HeapValue::Object(
                crate::vm::interpreter::JsObject { properties: props, prototype: None },
            ));
            Value::Object(heap_idx)
        }
    }
}

// ---- Console ----

fn native_console_log(interp: &mut Interpreter, _this: &Value, args: &[Value]) -> Result<Value> {
    let parts: Vec<String> = args.iter().map(|a| to_display_string(interp, a)).collect();
    println!("{}", parts.join(" "));
    Ok(Value::Undefined)
}

fn native_console_warn(interp: &mut Interpreter, _this: &Value, args: &[Value]) -> Result<Value> {
    let parts: Vec<String> = args.iter().map(|a| to_display_string(interp, a)).collect();
    eprintln!("{}", parts.join(" "));
    Ok(Value::Undefined)
}

fn native_console_error(interp: &mut Interpreter, _this: &Value, args: &[Value]) -> Result<Value> {
    let parts: Vec<String> = args.iter().map(|a| to_display_string(interp, a)).collect();
    eprintln!("{}", parts.join(" "));
    Ok(Value::Undefined)
}

fn native_console_info(interp: &mut Interpreter, _this: &Value, args: &[Value]) -> Result<Value> {
    let parts: Vec<String> = args.iter().map(|a| to_display_string(interp, a)).collect();
    println!("{}", parts.join(" "));
    Ok(Value::Undefined)
}

// ---- Object ----

fn native_object_keys(interp: &mut Interpreter, _this: &Value, args: &[Value]) -> Result<Value> {
    let obj_val = args.first().cloned().unwrap_or(Value::Undefined);
    let mut keys = Vec::new();
    match &obj_val {
        Value::Object(obj_idx) => {
            if let crate::vm::interpreter::HeapValue::Object(obj) = &interp.heap[*obj_idx] {
                for k in obj.properties.keys() {
                    keys.push(Value::String(k.clone()));
                }
            }
        }
        Value::Array(arr_idx) => {
            if let crate::vm::interpreter::HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                for i in 0..arr.elements.len() {
                    keys.push(Value::String(i.to_string()));
                }
            }
        }
        _ => {}
    }
    let heap_idx = interp.heap.len();
    interp.heap.push(crate::vm::interpreter::HeapValue::Array(
        crate::vm::interpreter::JsArray { elements: keys },
    ));
    Ok(Value::Array(heap_idx))
}

fn native_object_values(interp: &mut Interpreter, _this: &Value, args: &[Value]) -> Result<Value> {
    let obj_val = args.first().cloned().unwrap_or(Value::Undefined);
    let mut vals = Vec::new();
    match &obj_val {
        Value::Object(obj_idx) => {
            if let crate::vm::interpreter::HeapValue::Object(obj) = &interp.heap[*obj_idx] {
                for v in obj.properties.values() {
                    vals.push(v.clone());
                }
            }
        }
        Value::Array(arr_idx) => {
            if let crate::vm::interpreter::HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                vals.extend(arr.elements.iter().cloned());
            }
        }
        _ => {}
    }
    let heap_idx = interp.heap.len();
    interp.heap.push(crate::vm::interpreter::HeapValue::Array(
        crate::vm::interpreter::JsArray { elements: vals },
    ));
    Ok(Value::Array(heap_idx))
}

fn native_object_entries(interp: &mut Interpreter, _this: &Value, args: &[Value]) -> Result<Value> {
    let obj_val = args.first().cloned().unwrap_or(Value::Undefined);
    let pairs: Vec<(String, Value)> = match &obj_val {
        Value::Object(obj_idx) => {
            if let crate::vm::interpreter::HeapValue::Object(obj) = &interp.heap[*obj_idx] {
                obj.properties.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
            } else { Vec::new() }
        }
        Value::Array(arr_idx) => {
            if let crate::vm::interpreter::HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                arr.elements.iter().enumerate().map(|(i, v)| (i.to_string(), v.clone())).collect()
            } else { Vec::new() }
        }
        _ => Vec::new(),
    };
    let mut entries = Vec::new();
    for (k, v) in pairs {
        let heap_idx = interp.heap.len();
        interp.heap.push(crate::vm::interpreter::HeapValue::Array(
            crate::vm::interpreter::JsArray { elements: vec![Value::String(k), v] },
        ));
        entries.push(Value::Array(heap_idx));
    }
    let heap_idx = interp.heap.len();
    interp.heap.push(crate::vm::interpreter::HeapValue::Array(
        crate::vm::interpreter::JsArray { elements: entries },
    ));
    Ok(Value::Array(heap_idx))
}

fn native_object_assign(interp: &mut Interpreter, _this: &Value, args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Ok(Value::Undefined);
    }
    let target = args[0].clone();
    if let Value::Object(target_idx) = &target {
        for src in &args[1..] {
            match src {
                Value::Object(src_idx) => {
                    let cloned: Vec<(String, Value)> = if let crate::vm::interpreter::HeapValue::Object(src_obj) = &interp.heap[*src_idx] {
                        src_obj.properties.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
                    } else { Vec::new() };
                    if let crate::vm::interpreter::HeapValue::Object(tgt_obj) = &mut interp.heap[*target_idx] {
                        for (k, v) in cloned {
                            tgt_obj.properties.insert(k, v);
                        }
                    }
                }
                _ => {}
            }
        }
    }
    Ok(target)
}

// ---- JSON ----

fn native_json_parse(interp: &mut Interpreter, _this: &Value, args: &[Value]) -> Result<Value> {
    let s = match args.first() {
        Some(Value::String(s)) => s.as_str(),
        Some(other) => return Err(Error::TypeError(format!("Cannot parse {}", to_string_value(interp, other)))),
        None => return Err(Error::TypeError("JSON.parse requires 1 argument".into())),
    };
    let parsed: serde_json::Value = serde_json::from_str(s)
        .map_err(|e| Error::SyntaxError(format!("JSON parse error: {}", e)))?;
    Ok(from_json_value(interp, parsed))
}

fn native_json_stringify(interp: &mut Interpreter, _this: &Value, args: &[Value]) -> Result<Value> {
    let val = args.first().cloned().unwrap_or(Value::Undefined);
    Ok(Value::String(to_json_value(interp, &val)))
}

// ---- Global functions ----

fn native_parse_int(_interp: &mut Interpreter, _this: &Value, args: &[Value]) -> Result<Value> {
    let s = match args.first() {
        Some(Value::String(s)) => s.clone(),
        Some(v) => to_string_value(_interp, v),
        None => return Ok(Value::Float(f64::NAN)),
    };
    let radix = args.get(1).map(|r| to_i64(r)).unwrap_or(10);
    let trimmed = s.trim().trim_start_matches(|c: char| c.is_ascii_digit() || c == '-' || c == '+' || c.is_alphabetic());
    let actual = s.trim();
    let result = i64::from_str_radix(
        actual.trim_start_matches(|c: char| c == '+' || c == '-'),
        radix as u32,
    );
    match result {
        Ok(n) => {
            if actual.starts_with('-') {
                Ok(Value::Float(-n as f64))
            } else {
                Ok(Value::Float(n as f64))
            }
        }
        Err(_) => Ok(Value::Float(f64::NAN)),
    }
}

fn native_parse_float(interp: &mut Interpreter, _this: &Value, args: &[Value]) -> Result<Value> {
    let s = match args.first() {
        Some(Value::String(s)) => s.clone(),
        Some(v) => to_string_value(interp, v),
        None => return Ok(Value::Float(f64::NAN)),
    };
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Ok(Value::Float(f64::NAN));
    }
    match trimmed.parse::<f64>() {
        Ok(n) => Ok(Value::Float(n)),
        Err(_) => Ok(Value::Float(f64::NAN)),
    }
}

fn native_is_nan(_interp: &mut Interpreter, _this: &Value, args: &[Value]) -> Result<Value> {
    let n = args.first().map(|v| to_f64(v)).unwrap_or(f64::NAN);
    Ok(Value::Boolean(n.is_nan()))
}

fn native_is_finite(_interp: &mut Interpreter, _this: &Value, args: &[Value]) -> Result<Value> {
    let n = args.first().map(|v| to_f64(v)).unwrap_or(f64::NAN);
    Ok(Value::Boolean(n.is_finite()))
}

// ---- Timer stubs ----

fn native_set_timeout(interp: &mut Interpreter, _this: &Value, args: &[Value]) -> Result<Value> {
    let callback = args.first().cloned().unwrap_or(Value::Undefined);
    let _delay = args.get(1).map(|v| to_f64(v)).unwrap_or(0.0);
    let id = interp.async_runtime.enqueue_macrotask(callback);
    Ok(Value::Float(id as f64))
}

fn native_set_interval(interp: &mut Interpreter, _this: &Value, args: &[Value]) -> Result<Value> {
    let callback = args.first().cloned().unwrap_or(Value::Undefined);
    let delay = args.get(1).map(|v| to_f64(v)).unwrap_or(0.0);
    let id = interp.async_runtime.enqueue_interval(callback, delay);
    Ok(Value::Float(id as f64))
}

fn native_clear_timeout(interp: &mut Interpreter, _this: &Value, args: &[Value]) -> Result<Value> {
    if let Some(Value::Float(id)) = args.first() {
        interp.async_runtime.cancel_timer(*id as u32);
    }
    Ok(Value::Undefined)
}

fn native_clear_interval(interp: &mut Interpreter, _this: &Value, args: &[Value]) -> Result<Value> {
    if let Some(Value::Float(id)) = args.first() {
        interp.async_runtime.cancel_timer(*id as u32);
    }
    Ok(Value::Undefined)
}

// ---- Math ----

fn native_math_abs(_interp: &mut Interpreter, _this: &Value, args: &[Value]) -> Result<Value> {
    let n = args.first().map(|v| to_f64(v)).unwrap_or(0.0);
    Ok(Value::Float(n.abs()))
}

fn native_math_floor(_interp: &mut Interpreter, _this: &Value, args: &[Value]) -> Result<Value> {
    let n = args.first().map(|v| to_f64(v)).unwrap_or(0.0);
    Ok(Value::Float(n.floor()))
}

fn native_math_ceil(_interp: &mut Interpreter, _this: &Value, args: &[Value]) -> Result<Value> {
    let n = args.first().map(|v| to_f64(v)).unwrap_or(0.0);
    Ok(Value::Float(n.ceil()))
}

fn native_math_round(_interp: &mut Interpreter, _this: &Value, args: &[Value]) -> Result<Value> {
    let n = args.first().map(|v| to_f64(v)).unwrap_or(0.0);
    Ok(Value::Float(n.round()))
}

fn native_math_min(_interp: &mut Interpreter, _this: &Value, args: &[Value]) -> Result<Value> {
    let mut result = f64::INFINITY;
    for arg in args {
        let n = to_f64(arg);
        if n < result { result = n; }
    }
    Ok(Value::Float(result))
}

fn native_math_max(_interp: &mut Interpreter, _this: &Value, args: &[Value]) -> Result<Value> {
    let mut result = f64::NEG_INFINITY;
    for arg in args {
        let n = to_f64(arg);
        if n > result { result = n; }
    }
    Ok(Value::Float(result))
}

fn native_math_random(_interp: &mut Interpreter, _this: &Value, _args: &[Value]) -> Result<Value> {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    let s = RandomState::new();
    let mut hasher = s.build_hasher();
    hasher.write_u64(std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64);
    let bits = hasher.finish();
    Ok(Value::Float((bits as f64) / (u64::MAX as f64)))
}

fn native_math_pow(_interp: &mut Interpreter, _this: &Value, args: &[Value]) -> Result<Value> {
    let base = args.first().map(|v| to_f64(v)).unwrap_or(0.0);
    let exp = args.get(1).map(|v| to_f64(v)).unwrap_or(0.0);
    Ok(Value::Float(base.powf(exp)))
}

fn native_math_sqrt(_interp: &mut Interpreter, _this: &Value, args: &[Value]) -> Result<Value> {
    let n = args.first().map(|v| to_f64(v)).unwrap_or(0.0);
    Ok(Value::Float(n.sqrt()))
}

fn native_math_log(_interp: &mut Interpreter, _this: &Value, args: &[Value]) -> Result<Value> {
    let n = args.first().map(|v| to_f64(v)).unwrap_or(0.0);
    Ok(Value::Float(n.ln()))
}

fn native_math_sin(_interp: &mut Interpreter, _this: &Value, args: &[Value]) -> Result<Value> {
    let n = args.first().map(|v| to_f64(v)).unwrap_or(0.0);
    Ok(Value::Float(n.sin()))
}

fn native_math_cos(_interp: &mut Interpreter, _this: &Value, args: &[Value]) -> Result<Value> {
    let n = args.first().map(|v| to_f64(v)).unwrap_or(0.0);
    Ok(Value::Float(n.cos()))
}

fn native_math_tan(_interp: &mut Interpreter, _this: &Value, args: &[Value]) -> Result<Value> {
    let n = args.first().map(|v| to_f64(v)).unwrap_or(0.0);
    Ok(Value::Float(n.tan()))
}

// ---- Array prototype methods ----

fn get_array_elements(interp: &Interpreter, v: &Value) -> Result<Vec<Value>> {
    match v {
        Value::Array(arr_idx) => {
            if let crate::vm::interpreter::HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                Ok(arr.elements.clone())
            } else {
                Ok(Vec::new())
            }
        }
        _ => Ok(Vec::new()),
    }
}

fn native_array_push(interp: &mut Interpreter, this: &Value, args: &[Value]) -> Result<Value> {
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

fn native_array_pop(interp: &mut Interpreter, this: &Value, _args: &[Value]) -> Result<Value> {
    if let Value::Array(arr_idx) = this {
        if let crate::vm::interpreter::HeapValue::Array(arr) = &mut interp.heap[*arr_idx] {
            return Ok(arr.elements.pop().unwrap_or(Value::Undefined));
        }
    }
    Ok(Value::Undefined)
}

fn native_array_shift(interp: &mut Interpreter, this: &Value, _args: &[Value]) -> Result<Value> {
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

fn native_array_unshift(interp: &mut Interpreter, this: &Value, args: &[Value]) -> Result<Value> {
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

fn native_array_slice(interp: &mut Interpreter, this: &Value, args: &[Value]) -> Result<Value> {
    let elements = get_array_elements(interp, this)?;
    let start_raw = args.first().map(|v| to_f64(v)).unwrap_or(0.0) as i64;
    let end_raw = args.get(1).map(|v| to_f64(v)).unwrap_or(elements.len() as f64) as i64;

    let len = elements.len() as i64;
    let start = if start_raw < 0 { (len + start_raw).max(0) } else { start_raw.min(len) } as usize;
    let end = if end_raw < 0 { (len + end_raw).max(0) } else { end_raw.min(len) } as usize;

    let sliced = if start < end { elements[start..end].to_vec() } else { Vec::new() };
    let heap_idx = interp.heap.len();
    interp.heap.push(crate::vm::interpreter::HeapValue::Array(
        crate::vm::interpreter::JsArray { elements: sliced },
    ));
    Ok(Value::Array(heap_idx))
}

fn native_array_splice(interp: &mut Interpreter, this: &Value, args: &[Value]) -> Result<Value> {
    if let Value::Array(arr_idx) = this {
        let start_raw = args.first().map(|v| to_f64(v)).unwrap_or(0.0) as i64;
        let delete_count_raw = args.get(1).map(|v| to_f64(v)).unwrap_or(0.0) as i64;

        if let crate::vm::interpreter::HeapValue::Array(arr) = &mut interp.heap[*arr_idx] {
            let len = arr.elements.len() as i64;
            let start = if start_raw < 0 { (len + start_raw).max(0) } else { start_raw.min(len) } as usize;
            let delete_count = delete_count_raw.max(0).min((len - start as i64) as i64) as usize;

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
        crate::vm::interpreter::JsArray { elements: Vec::new() },
    ));
    Ok(Value::Array(heap_idx))
}

fn native_array_index_of(interp: &mut Interpreter, this: &Value, args: &[Value]) -> Result<Value> {
    let elements = get_array_elements(interp, this)?;
    let search = args.first().cloned().unwrap_or(Value::Undefined);
    let from = args.get(1).map(|v| to_f64(v) as usize).unwrap_or(0);
    for (i, elem) in elements.iter().enumerate() {
        if i >= from {
            if elem == &search {
                return Ok(Value::Float(i as f64));
            }
        }
    }
    Ok(Value::Float(-1.0))
}

fn native_array_includes(interp: &mut Interpreter, this: &Value, args: &[Value]) -> Result<Value> {
    let elements = get_array_elements(interp, this)?;
    let search = args.first().cloned().unwrap_or(Value::Undefined);
    Ok(Value::Boolean(elements.contains(&search)))
}

fn native_array_find(interp: &mut Interpreter, this: &Value, args: &[Value]) -> Result<Value> {
    let elements = get_array_elements(interp, this)?;
    let callback = args.first().cloned().unwrap_or(Value::Undefined);
    for (i, elem) in elements.iter().enumerate() {
        let mut call_args = vec![elem.clone(), Value::Integer(i as i64), this.clone()];
        let result = interp.call_value(&callback, &Value::Undefined, &call_args)?;
        if interp.is_truthy(&result) {
            return Ok(elem.clone());
        }
    }
    Ok(Value::Undefined)
}

fn native_array_find_index(interp: &mut Interpreter, this: &Value, args: &[Value]) -> Result<Value> {
    let elements = get_array_elements(interp, this)?;
    let callback = args.first().cloned().unwrap_or(Value::Undefined);
    for (i, elem) in elements.iter().enumerate() {
        let mut call_args = vec![elem.clone(), Value::Integer(i as i64), this.clone()];
        let result = interp.call_value(&callback, &Value::Undefined, &call_args)?;
        if interp.is_truthy(&result) {
            return Ok(Value::Float(i as f64));
        }
    }
    Ok(Value::Float(-1.0))
}

fn native_array_map(interp: &mut Interpreter, this: &Value, args: &[Value]) -> Result<Value> {
    let elements = get_array_elements(interp, this)?;
    let callback = args.first().cloned().unwrap_or(Value::Undefined);
    let mut results = Vec::new();
    for (i, elem) in elements.iter().enumerate() {
        let mut call_args = vec![elem.clone(), Value::Integer(i as i64), this.clone()];
        let result = interp.call_value(&callback, &Value::Undefined, &call_args)?;
        results.push(result);
    }
    let heap_idx = interp.heap.len();
    interp.heap.push(crate::vm::interpreter::HeapValue::Array(
        crate::vm::interpreter::JsArray { elements: results },
    ));
    Ok(Value::Array(heap_idx))
}

fn native_array_filter(interp: &mut Interpreter, this: &Value, args: &[Value]) -> Result<Value> {
    let elements = get_array_elements(interp, this)?;
    let callback = args.first().cloned().unwrap_or(Value::Undefined);
    let mut results = Vec::new();
    for (i, elem) in elements.iter().enumerate() {
        let mut call_args = vec![elem.clone(), Value::Integer(i as i64), this.clone()];
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

fn native_array_reduce(interp: &mut Interpreter, this: &Value, args: &[Value]) -> Result<Value> {
    let elements = get_array_elements(interp, this)?;
    let callback = args.first().cloned().unwrap_or(Value::Undefined);
    let has_init = args.len() > 1;
    let mut acc = if has_init { args[1].clone() } else { Value::Undefined };
    let start_idx = if has_init { 0 } else { 1 };

    if !has_init && elements.is_empty() {
        return Err(Error::TypeError("Reduce of empty array with no initial value".into()));
    }

    if !has_init {
        acc = elements[0].clone();
    }

    for i in start_idx..elements.len() {
        let call_args = vec![acc, elements[i].clone(), Value::Integer(i as i64), this.clone()];
        acc = interp.call_value(&callback, &Value::Undefined, &call_args)?;
    }
    Ok(acc)
}

fn native_array_for_each(interp: &mut Interpreter, this: &Value, args: &[Value]) -> Result<Value> {
    let elements = get_array_elements(interp, this)?;
    let callback = args.first().cloned().unwrap_or(Value::Undefined);
    for (i, elem) in elements.iter().enumerate() {
        let mut call_args = vec![elem.clone(), Value::Integer(i as i64), this.clone()];
        interp.call_value(&callback, &Value::Undefined, &call_args)?;
    }
    Ok(Value::Undefined)
}

fn native_array_some(interp: &mut Interpreter, this: &Value, args: &[Value]) -> Result<Value> {
    let elements = get_array_elements(interp, this)?;
    let callback = args.first().cloned().unwrap_or(Value::Undefined);
    for (i, elem) in elements.iter().enumerate() {
        let mut call_args = vec![elem.clone(), Value::Integer(i as i64), this.clone()];
        let result = interp.call_value(&callback, &Value::Undefined, &call_args)?;
        if interp.is_truthy(&result) {
            return Ok(Value::Boolean(true));
        }
    }
    Ok(Value::Boolean(false))
}

fn native_array_every(interp: &mut Interpreter, this: &Value, args: &[Value]) -> Result<Value> {
    let elements = get_array_elements(interp, this)?;
    let callback = args.first().cloned().unwrap_or(Value::Undefined);
    for (i, elem) in elements.iter().enumerate() {
        let mut call_args = vec![elem.clone(), Value::Integer(i as i64), this.clone()];
        let result = interp.call_value(&callback, &Value::Undefined, &call_args)?;
        if !interp.is_truthy(&result) {
            return Ok(Value::Boolean(false));
        }
    }
    Ok(Value::Boolean(true))
}

fn native_array_join(interp: &mut Interpreter, this: &Value, args: &[Value]) -> Result<Value> {
    let elements = get_array_elements(interp, this)?;
    let sep = match args.first() {
        Some(Value::String(s)) => s.clone(),
        Some(v) => to_string_value(interp, v),
        None => ",".to_string(),
    };
    let parts: Vec<String> = elements.iter().map(|e| to_string_value(interp, e)).collect();
    Ok(Value::String(parts.join(&sep)))
}

fn native_array_reverse(interp: &mut Interpreter, this: &Value, _args: &[Value]) -> Result<Value> {
    if let Value::Array(arr_idx) = this {
        if let crate::vm::interpreter::HeapValue::Array(arr) = &mut interp.heap[*arr_idx] {
            arr.elements.reverse();
        }
    }
    Ok(this.clone())
}

fn native_array_sort(interp: &mut Interpreter, this: &Value, _args: &[Value]) -> Result<Value> {
    if let Value::Array(arr_idx) = this {
        let elements: Vec<Value> = if let crate::vm::interpreter::HeapValue::Array(arr) = &interp.heap[*arr_idx] {
            arr.elements.clone()
        } else { return Ok(this.clone()); };
        let mut indexed: Vec<(String, Value)> = elements.iter()
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

fn native_array_concat(interp: &mut Interpreter, this: &Value, args: &[Value]) -> Result<Value> {
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

fn native_array_flat(interp: &mut Interpreter, this: &Value, args: &[Value]) -> Result<Value> {
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

// ---- String prototype methods ----

fn get_string(this: &Value) -> Option<String> {
    match this {
        Value::String(s) => Some(s.clone()),
        _ => None,
    }
}

fn native_string_char_at(_interp: &mut Interpreter, this: &Value, args: &[Value]) -> Result<Value> {
    let s = get_string(this).unwrap_or_default();
    let idx = args.first().map(|v| to_f64(v) as usize).unwrap_or(0);
    match s.chars().nth(idx) {
        Some(c) => Ok(Value::String(c.to_string())),
        None => Ok(Value::String("".to_string())),
    }
}

fn native_string_char_code_at(_interp: &mut Interpreter, this: &Value, args: &[Value]) -> Result<Value> {
    let s = get_string(this).unwrap_or_default();
    let idx = args.first().map(|v| to_f64(v) as usize).unwrap_or(0);
    match s.chars().nth(idx) {
        Some(c) => Ok(Value::Float(c as i64 as f64)),
        None => Ok(Value::Float(f64::NAN)),
    }
}

fn native_string_slice(_interp: &mut Interpreter, this: &Value, args: &[Value]) -> Result<Value> {
    let s = get_string(this).unwrap_or_default();
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len() as i64;
    let start_raw = args.first().map(|v| to_f64(v) as i64).unwrap_or(0);
    let end_raw = args.get(1).map(|v| to_f64(v) as i64).unwrap_or(len);

    let start = if start_raw < 0 { (len + start_raw).max(0) } else { start_raw.min(len) } as usize;
    let end = if end_raw < 0 { (len + end_raw).max(0) } else { end_raw.min(len) } as usize;

    let result: String = chars[start..end].iter().collect();
    Ok(Value::String(result))
}

fn native_string_substring(_interp: &mut Interpreter, this: &Value, args: &[Value]) -> Result<Value> {
    let s = get_string(this).unwrap_or_default();
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len() as i64;
    let start = args.first().map(|v| to_f64(v) as i64).unwrap_or(0).max(0).min(len) as usize;
    let end = args.get(1).map(|v| to_f64(v) as i64).unwrap_or(len).max(0).min(len) as usize;
    let (start, end) = if start <= end { (start, end) } else { (end, start) };
    let result: String = chars[start..end].iter().collect();
    Ok(Value::String(result))
}

fn native_string_index_of(_interp: &mut Interpreter, this: &Value, args: &[Value]) -> Result<Value> {
    let s = get_string(this).unwrap_or_default();
    let search = match args.first() {
        Some(Value::String(ss)) => ss.as_str(),
        _ => return Ok(Value::Float(-1.0)),
    };
    match s.find(search) {
        Some(pos) => Ok(Value::Float(pos as f64)),
        None => Ok(Value::Float(-1.0)),
    }
}

fn native_string_includes(_interp: &mut Interpreter, this: &Value, args: &[Value]) -> Result<Value> {
    let s = get_string(this).unwrap_or_default();
    let search = match args.first() {
        Some(Value::String(ss)) => ss.as_str(),
        _ => return Ok(Value::Boolean(false)),
    };
    Ok(Value::Boolean(s.contains(search)))
}

fn native_string_replace(interp: &mut Interpreter, this: &Value, args: &[Value]) -> Result<Value> {
    let s = get_string(this).unwrap_or_default();
    let pattern = match args.first() {
        Some(Value::String(ss)) => ss.as_str(),
        _ => return Ok(Value::String(s)),
    };
    let replacement = match args.get(1) {
        Some(v) => to_string_value(interp, v),
        None => "undefined".to_string(),
    };
    match s.find(pattern) {
        Some(pos) => {
            let end = pos + pattern.len();
            let result = format!("{}{}{}", &s[..pos], replacement, &s[end..]);
            Ok(Value::String(result))
        }
        None => Ok(Value::String(s)),
    }
}

fn native_string_split(interp: &mut Interpreter, this: &Value, args: &[Value]) -> Result<Value> {
    let s = get_string(this).unwrap_or_default();
    let sep = match args.first() {
        Some(Value::String(ss)) => ss.as_str(),
        _ => return {
            let heap_idx = interp.heap.len();
            interp.heap.push(crate::vm::interpreter::HeapValue::Array(
                crate::vm::interpreter::JsArray { elements: vec![Value::String(s)] },
            ));
            Ok(Value::Array(heap_idx))
        },
    };
    let parts: Vec<Value> = s.split(sep).map(|p| Value::String(p.to_string())).collect();
    let heap_idx = interp.heap.len();
    interp.heap.push(crate::vm::interpreter::HeapValue::Array(
        crate::vm::interpreter::JsArray { elements: parts },
    ));
    Ok(Value::Array(heap_idx))
}

fn native_string_trim(_interp: &mut Interpreter, this: &Value, _args: &[Value]) -> Result<Value> {
    let s = get_string(this).unwrap_or_default();
    Ok(Value::String(s.trim().to_string()))
}

fn native_string_to_lower_case(_interp: &mut Interpreter, this: &Value, _args: &[Value]) -> Result<Value> {
    let s = get_string(this).unwrap_or_default();
    Ok(Value::String(s.to_lowercase()))
}

fn native_string_to_upper_case(_interp: &mut Interpreter, this: &Value, _args: &[Value]) -> Result<Value> {
    let s = get_string(this).unwrap_or_default();
    Ok(Value::String(s.to_uppercase()))
}

fn native_string_starts_with(_interp: &mut Interpreter, this: &Value, args: &[Value]) -> Result<Value> {
    let s = get_string(this).unwrap_or_default();
    let prefix = match args.first() {
        Some(Value::String(ss)) => ss.as_str(),
        _ => return Ok(Value::Boolean(false)),
    };
    Ok(Value::Boolean(s.starts_with(prefix)))
}

fn native_string_ends_with(_interp: &mut Interpreter, this: &Value, args: &[Value]) -> Result<Value> {
    let s = get_string(this).unwrap_or_default();
    let suffix = match args.first() {
        Some(Value::String(ss)) => ss.as_str(),
        _ => return Ok(Value::Boolean(false)),
    };
    Ok(Value::Boolean(s.ends_with(suffix)))
}

fn native_string_repeat(_interp: &mut Interpreter, this: &Value, args: &[Value]) -> Result<Value> {
    let s = get_string(this).unwrap_or_default();
    let count = args.first().map(|v| to_f64(v) as usize).unwrap_or(0);
    if count == 0 || s.is_empty() {
        return Ok(Value::String("".to_string()));
    }
    let result: String = s.repeat(count);
    Ok(Value::String(result))
}

fn native_string_pad_start(interp: &mut Interpreter, this: &Value, args: &[Value]) -> Result<Value> {
    let s = get_string(this).unwrap_or_default();
    let target_len = args.first().map(|v| to_f64(v) as usize).unwrap_or(0);
    let pad_char = match args.get(1) {
        Some(Value::String(ss)) => ss.chars().next().unwrap_or(' '),
        _ => ' ',
    };
    if s.len() >= target_len {
        return Ok(Value::String(s));
    }
    let pad_count = target_len - s.len();
    let padding: String = std::iter::repeat(pad_char).take(pad_count).collect();
    Ok(Value::String(format!("{}{}", padding, s)))
}

fn native_string_pad_end(interp: &mut Interpreter, this: &Value, args: &[Value]) -> Result<Value> {
    let s = get_string(this).unwrap_or_default();
    let target_len = args.first().map(|v| to_f64(v) as usize).unwrap_or(0);
    let pad_char = match args.get(1) {
        Some(Value::String(ss)) => ss.chars().next().unwrap_or(' '),
        _ => ' ',
    };
    if s.len() >= target_len {
        return Ok(Value::String(s));
    }
    let pad_count = target_len - s.len();
    let padding: String = std::iter::repeat(pad_char).take(pad_count).collect();
    Ok(Value::String(format!("{}{}", s, padding)))
}

// ---- Number methods ----

fn native_number_parse_int(interp: &mut Interpreter, this: &Value, args: &[Value]) -> Result<Value> {
    native_parse_int(interp, this, args)
}

fn native_number_parse_float(interp: &mut Interpreter, this: &Value, args: &[Value]) -> Result<Value> {
    native_parse_float(interp, this, args)
}

fn native_number_is_nan(interp: &mut Interpreter, this: &Value, args: &[Value]) -> Result<Value> {
    native_is_nan(interp, this, args)
}

fn native_number_is_finite(interp: &mut Interpreter, this: &Value, args: &[Value]) -> Result<Value> {
    native_is_finite(interp, this, args)
}

// ---- Error constructors ----

fn native_error_constructor(interp: &mut Interpreter, _this: &Value, args: &[Value]) -> Result<Value> {
    let message = args.first().map(|v| to_string_value(interp, v)).unwrap_or_default();
    let obj_idx = interp.heap.len();
    let mut props = std::collections::HashMap::new();
    props.insert("message".into(), Value::String(message.clone()));
    props.insert("name".into(), Value::String("Error".into()));
    props.insert("stack".into(), Value::String(format!("Error{}", if message.is_empty() { String::new() } else { format!(": {}", message) })));

    let proto_idx = find_error_ctor_proto(interp);

    interp.heap.push(crate::vm::interpreter::HeapValue::Object(crate::vm::interpreter::JsObject {
        properties: props,
        prototype: proto_idx,
    }));
    Ok(Value::Object(obj_idx))
}

fn native_type_error_constructor(interp: &mut Interpreter, _this: &Value, args: &[Value]) -> Result<Value> {
    let message = args.first().map(|v| to_string_value(interp, v)).unwrap_or_default();
    let obj_idx = interp.heap.len();
    let mut props = std::collections::HashMap::new();
    props.insert("message".into(), Value::String(message));
    props.insert("name".into(), Value::String("TypeError".into()));
    props.insert("stack".into(), Value::String("TypeError".into()));
    
    let proto_idx = find_error_proto(interp, "TypeError");
    interp.heap.push(crate::vm::interpreter::HeapValue::Object(crate::vm::interpreter::JsObject {
        properties: props,
        prototype: proto_idx,
    }));
    Ok(Value::Object(obj_idx))
}

fn native_reference_error_constructor(interp: &mut Interpreter, _this: &Value, args: &[Value]) -> Result<Value> {
    let message = args.first().map(|v| to_string_value(interp, v)).unwrap_or_default();
    let obj_idx = interp.heap.len();
    let mut props = std::collections::HashMap::new();
    props.insert("message".into(), Value::String(message));
    props.insert("name".into(), Value::String("ReferenceError".into()));
    props.insert("stack".into(), Value::String("ReferenceError".into()));
    
    let proto_idx = find_error_proto(interp, "ReferenceError");
    interp.heap.push(crate::vm::interpreter::HeapValue::Object(crate::vm::interpreter::JsObject {
        properties: props,
        prototype: proto_idx,
    }));
    Ok(Value::Object(obj_idx))
}

fn native_syntax_error_constructor(interp: &mut Interpreter, _this: &Value, args: &[Value]) -> Result<Value> {
    let message = args.first().map(|v| to_string_value(interp, v)).unwrap_or_default();
    let obj_idx = interp.heap.len();
    let mut props = std::collections::HashMap::new();
    props.insert("message".into(), Value::String(message));
    props.insert("name".into(), Value::String("SyntaxError".into()));
    props.insert("stack".into(), Value::String("SyntaxError".into()));
    
    let proto_idx = find_error_proto(interp, "SyntaxError");
    interp.heap.push(crate::vm::interpreter::HeapValue::Object(crate::vm::interpreter::JsObject {
        properties: props,
        prototype: proto_idx,
    }));
    Ok(Value::Object(obj_idx))
}

fn native_range_error_constructor(interp: &mut Interpreter, _this: &Value, args: &[Value]) -> Result<Value> {
    let message = args.first().map(|v| to_string_value(interp, v)).unwrap_or_default();
    let obj_idx = interp.heap.len();
    let mut props = std::collections::HashMap::new();
    props.insert("message".into(), Value::String(message));
    props.insert("name".into(), Value::String("RangeError".into()));
    props.insert("stack".into(), Value::String("RangeError".into()));
    
    let proto_idx = find_error_proto(interp, "RangeError");
    interp.heap.push(crate::vm::interpreter::HeapValue::Object(crate::vm::interpreter::JsObject {
        properties: props,
        prototype: proto_idx,
    }));
    Ok(Value::Object(obj_idx))
}

fn find_error_ctor_proto(interp: &Interpreter) -> Option<usize> {
    for hv in &interp.heap {
        if let crate::vm::interpreter::HeapValue::Object(obj) = hv {
            if obj.properties.contains_key("prototype")
                && !obj.properties.contains_key("name")
            {
                if let Some(Value::Object(proto_idx)) = obj.properties.get("prototype") {
                    return Some(*proto_idx);
                }
            }
        }
    }
    None
}

fn find_error_proto(interp: &Interpreter, type_name: &str) -> Option<usize> {
    for (i, hv) in interp.heap.iter().enumerate() {
        if let crate::vm::interpreter::HeapValue::Object(obj) = hv {
            if let Some(Value::String(name)) = obj.properties.get("name") {
                if name == type_name {
                    return Some(i);
                }
            }
        }
    }
    None
}

// ---- Promise ----

fn native_promise_constructor(interp: &mut Interpreter, _this: &Value, args: &[Value]) -> Result<Value> {
    let executor = args.first().cloned().unwrap_or(Value::Undefined);

    let promise_idx = interp.heap.len();
    interp.heap.push(crate::vm::interpreter::HeapValue::Promise(
        crate::objects::js_promise::JsPromise::new(),
    ));

    if matches!(executor, Value::Function(_) | Value::NativeFunction(_)) {
        let resolve_fn = interp.create_resolve_fn(promise_idx);
        let reject_fn = interp.create_reject_fn(promise_idx);
        interp.call_value(&executor, &Value::Undefined, &[resolve_fn, reject_fn])?;
    }

    Ok(Value::Promise(promise_idx))
}

fn native_promise_then(interp: &mut Interpreter, this: &Value, args: &[Value]) -> Result<Value> {
    let callback = args.first().cloned().unwrap_or(Value::Undefined);
    let on_fulfilled = if let Value::Promise(idx) = this {
        *idx
    } else {
        return Ok(Value::Undefined);
    };

    let new_promise_idx = interp.heap.len();
    interp.heap.push(crate::vm::interpreter::HeapValue::Promise(
        crate::objects::js_promise::JsPromise::new(),
    ));

    let state_snapshot = if let crate::vm::interpreter::HeapValue::Promise(promise) = &interp.heap[on_fulfilled] {
        promise.state.clone()
    } else {
        return Ok(Value::Promise(new_promise_idx));
    };

    match state_snapshot {
        crate::objects::js_promise::PromiseState::Fulfilled(value) => {
            let result = interp.call_value(&callback, &Value::Undefined, &[value])?;
            interp.resolve_promise(new_promise_idx, result);
        }
        crate::objects::js_promise::PromiseState::Rejected(reason) => {
            interp.reject_promise(new_promise_idx, reason);
        }
        crate::objects::js_promise::PromiseState::Pending => {
            if let crate::vm::interpreter::HeapValue::Promise(promise) = &mut interp.heap[on_fulfilled] {
                let cb_idx = match callback {
                    Value::Function(idx) => idx,
                    _ => 0,
                };
                promise.then_handlers.push(crate::objects::js_promise::PromiseHandler {
                    callback: cb_idx,
                    resolve: true,
                });
            }
        }
    }

    Ok(Value::Promise(new_promise_idx))
}

fn native_promise_catch(interp: &mut Interpreter, this: &Value, args: &[Value]) -> Result<Value> {
    let callback = args.first().cloned().unwrap_or(Value::Undefined);
    let on_rejected = if let Value::Promise(idx) = this {
        *idx
    } else {
        return Ok(Value::Undefined);
    };

    let new_promise_idx = interp.heap.len();
    interp.heap.push(crate::vm::interpreter::HeapValue::Promise(
        crate::objects::js_promise::JsPromise::new(),
    ));

    let state_snapshot = if let crate::vm::interpreter::HeapValue::Promise(promise) = &interp.heap[on_rejected] {
        promise.state.clone()
    } else {
        return Ok(Value::Promise(new_promise_idx));
    };

    match state_snapshot {
        crate::objects::js_promise::PromiseState::Rejected(reason) => {
            let result = interp.call_value(&callback, &Value::Undefined, &[reason])?;
            interp.resolve_promise(new_promise_idx, result);
        }
        crate::objects::js_promise::PromiseState::Fulfilled(value) => {
            interp.resolve_promise(new_promise_idx, value);
        }
        crate::objects::js_promise::PromiseState::Pending => {
            if let crate::vm::interpreter::HeapValue::Promise(promise) = &mut interp.heap[on_rejected] {
                let cb_idx = match callback {
                    Value::Function(idx) => idx,
                    _ => 0,
                };
                promise.catch_handlers.push(crate::objects::js_promise::PromiseHandler {
                    callback: cb_idx,
                    resolve: false,
                });
            }
        }
    }

    Ok(Value::Promise(new_promise_idx))
}

fn native_promise_finally(interp: &mut Interpreter, this: &Value, args: &[Value]) -> Result<Value> {
    let callback = args.first().cloned().unwrap_or(Value::Undefined);
    let on_finally = if let Value::Promise(idx) = this {
        *idx
    } else {
        return Ok(Value::Undefined);
    };

    let new_promise_idx = interp.heap.len();
    interp.heap.push(crate::vm::interpreter::HeapValue::Promise(
        crate::objects::js_promise::JsPromise::new(),
    ));

    let state_snapshot = if let crate::vm::interpreter::HeapValue::Promise(promise) = &interp.heap[on_finally] {
        promise.state.clone()
    } else {
        return Ok(Value::Promise(new_promise_idx));
    };

    match state_snapshot {
        crate::objects::js_promise::PromiseState::Fulfilled(value) | crate::objects::js_promise::PromiseState::Rejected(value) => {
            let _ = interp.call_value(&callback, &Value::Undefined, &[])?;
            interp.resolve_promise(new_promise_idx, value);
        }
        crate::objects::js_promise::PromiseState::Pending => {
            if let crate::vm::interpreter::HeapValue::Promise(promise) = &mut interp.heap[on_finally] {
                let cb_idx = match callback {
                    Value::Function(idx) => idx,
                    _ => 0,
                };
                promise.finally_handlers.push(crate::objects::js_promise::PromiseHandler {
                    callback: cb_idx,
                    resolve: true,
                });
            }
        }
    }

    Ok(Value::Promise(new_promise_idx))
}

fn native_promise_resolve(interp: &mut Interpreter, _this: &Value, args: &[Value]) -> Result<Value> {
    let value = args.first().cloned().unwrap_or(Value::Undefined);

    let promise_idx = interp.heap.len();
    interp.heap.push(crate::vm::interpreter::HeapValue::Promise(
        crate::objects::js_promise::JsPromise::new(),
    ));

    interp.resolve_promise(promise_idx, value);
    Ok(Value::Promise(promise_idx))
}

fn native_promise_reject(interp: &mut Interpreter, _this: &Value, args: &[Value]) -> Result<Value> {
    let reason = args.first().cloned().unwrap_or(Value::Undefined);

    let promise_idx = interp.heap.len();
    interp.heap.push(crate::vm::interpreter::HeapValue::Promise(
        crate::objects::js_promise::JsPromise::new(),
    ));

    interp.reject_promise(promise_idx, reason);
    Ok(Value::Promise(promise_idx))
}

fn native_promise_all(interp: &mut Interpreter, _this: &Value, args: &[Value]) -> Result<Value> {
    let promises_arg = args.first().cloned().unwrap_or(Value::Undefined);

    let promise_indices: Vec<usize> = match &promises_arg {
        Value::Array(arr_idx) => {
            if let crate::vm::interpreter::HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                arr.elements.iter().filter_map(|v| {
                    if let Value::Promise(idx) = v { Some(*idx) } else { None }
                }).collect()
            } else {
                Vec::new()
            }
        }
        _ => Vec::new(),
    };

    let result_idx = interp.heap.len();
    interp.heap.push(crate::vm::interpreter::HeapValue::Promise(
        crate::objects::js_promise::JsPromise::new(),
    ));

    let mut all_fulfilled = true;
    let mut values = Vec::new();
    let mut first_rejection: Option<Value> = None;

    for &p_idx in &promise_indices {
        if let crate::vm::interpreter::HeapValue::Promise(p) = &interp.heap[p_idx] {
            match &p.state {
                crate::objects::js_promise::PromiseState::Fulfilled(v) => {
                    values.push(v.clone());
                }
                crate::objects::js_promise::PromiseState::Rejected(r) => {
                    if first_rejection.is_none() {
                        first_rejection = Some(r.clone());
                    }
                }
                crate::objects::js_promise::PromiseState::Pending => {
                    all_fulfilled = false;
                }
            }
        }
    }

    if let Some(reason) = first_rejection {
        interp.reject_promise(result_idx, reason);
    } else if all_fulfilled {
        let heap_idx = interp.heap.len();
        interp.heap.push(crate::vm::interpreter::HeapValue::Array(
            crate::vm::interpreter::JsArray { elements: values },
        ));
        interp.resolve_promise(result_idx, Value::Array(heap_idx));
    }

    Ok(Value::Promise(result_idx))
}

fn native_promise_race(interp: &mut Interpreter, _this: &Value, args: &[Value]) -> Result<Value> {
    let promises_arg = args.first().cloned().unwrap_or(Value::Undefined);

    let promise_indices: Vec<usize> = match &promises_arg {
        Value::Array(arr_idx) => {
            if let crate::vm::interpreter::HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                arr.elements.iter().filter_map(|v| {
                    if let Value::Promise(idx) = v { Some(*idx) } else { None }
                }).collect()
            } else {
                Vec::new()
            }
        }
        _ => Vec::new(),
    };

    let result_idx = interp.heap.len();
    interp.heap.push(crate::vm::interpreter::HeapValue::Promise(
        crate::objects::js_promise::JsPromise::new(),
    ));

    for &p_idx in &promise_indices {
        if let crate::vm::interpreter::HeapValue::Promise(p) = &interp.heap[p_idx] {
            match &p.state {
                crate::objects::js_promise::PromiseState::Fulfilled(v) => {
                    interp.resolve_promise(result_idx, v.clone());
                    return Ok(Value::Promise(result_idx));
                }
                crate::objects::js_promise::PromiseState::Rejected(r) => {
                    interp.reject_promise(result_idx, r.clone());
                    return Ok(Value::Promise(result_idx));
                }
                crate::objects::js_promise::PromiseState::Pending => {}
            }
        }
    }

    Ok(Value::Promise(result_idx))
}
