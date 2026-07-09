use crate::errors::{Error, Result};
use crate::objects::Value;
use crate::vm::interpreter::{HeapValue, Interpreter};

use super::helpers::{get_string, to_f64, to_string_value};

pub(super) fn native_string_constructor(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let value = args.first().cloned().unwrap_or(Value::Undefined);
    Ok(Value::from_string(to_string_value(interp, &value)))
}

pub(super) fn native_string_char_at(
    _interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let s = get_string(this).unwrap_or_default();
    let idx = args.first().map(|v| to_f64(v) as usize).unwrap_or(0);
    match s.chars().nth(idx) {
        Some(c) => Ok(Value::from_string(c.to_string())),
        None => Ok(Value::from_string("".to_string())),
    }
}

pub(super) fn native_string_char_code_at(
    _interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let s = get_string(this).unwrap_or_default();
    let idx = args.first().map(|v| to_f64(v) as usize).unwrap_or(0);
    match s.chars().nth(idx) {
        Some(c) => Ok(Value::Float(c as i64 as f64)),
        None => Ok(Value::Float(f64::NAN)),
    }
}

pub(super) fn native_string_slice(
    _interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let s = get_string(this).unwrap_or_default();
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len() as i64;
    let start_raw = args.first().map(|v| to_f64(v) as i64).unwrap_or(0);
    let end_raw = args.get(1).map(|v| to_f64(v) as i64).unwrap_or(len);

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

    let result: String = chars[start..end].iter().collect();
    Ok(Value::from_string(result))
}

pub(super) fn native_string_substring(
    _interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let s = get_string(this).unwrap_or_default();
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len() as i64;
    let start = args
        .first()
        .map(|v| to_f64(v) as i64)
        .unwrap_or(0)
        .max(0)
        .min(len) as usize;
    let end = args
        .get(1)
        .map(|v| to_f64(v) as i64)
        .unwrap_or(len)
        .max(0)
        .min(len) as usize;
    let (start, end) = if start <= end {
        (start, end)
    } else {
        (end, start)
    };
    let result: String = chars[start..end].iter().collect();
    Ok(Value::from_string(result))
}

pub(super) fn native_string_index_of(
    _interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let s = get_string(this).unwrap_or_default();
    let search = match args.first() {
        Some(Value::String(ss)) => ss.as_ref(),
        _ => return Ok(Value::Float(-1.0)),
    };
    match s.find(search) {
        Some(pos) => Ok(Value::Float(pos as f64)),
        None => Ok(Value::Float(-1.0)),
    }
}

pub(super) fn native_string_includes(
    _interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let s = get_string(this).unwrap_or_default();
    let search = match args.first() {
        Some(Value::String(ss)) => ss.as_ref(),
        _ => return Ok(Value::Boolean(false)),
    };
    Ok(Value::Boolean(s.contains(search)))
}

pub(super) fn native_string_replace(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let s = get_string(this).unwrap_or_default();
    let search = args.first().cloned().unwrap_or(Value::Undefined);
    let replacer = args.get(1).cloned().unwrap_or(Value::Undefined);

    // Function replacer: String.prototype.replace(pattern, (match, ...groups, offset, string) => ...)
    let is_fn = matches!(replacer, Value::Function(_) | Value::NativeFunction(_));

    match search {
        Value::RegExp(idx) => {
            // Collect matches first to avoid borrow issues across call_value.
            let (matches, global) = {
                let HeapValue::RegExp(ref regexp) = &interp.heap[idx] else {
                    return Err(Error::TypeError("Not a RegExp".into()));
                };
                let global = regexp.global;
                let compiled = regexp.compiled.as_ref();
                let mut matches: Vec<(usize, usize, String, Vec<String>)> = Vec::new();
                if let Some(compiled) = compiled {
                    match compiled {
                        crate::vm::interpreter::JsCompiledRegex::Simple(r) => {
                            for caps in r.captures_iter(&s) {
                                let m = caps.get(0).unwrap();
                                let groups: Vec<String> = caps
                                    .iter()
                                    .skip(1)
                                    .map(|g| g.map(|x| x.as_str().to_string()).unwrap_or_default())
                                    .collect();
                                matches.push((m.start(), m.end(), m.as_str().to_string(), groups));
                                if !global {
                                    break;
                                }
                            }
                        }
                        crate::vm::interpreter::JsCompiledRegex::Advanced(r) => {
                            for caps in r.captures_iter(&s).flatten() {
                                let m = caps.get(0).unwrap();
                                let groups: Vec<String> = (1..caps.len())
                                    .map(|i| {
                                        caps.get(i)
                                            .map(|x| x.as_str().to_string())
                                            .unwrap_or_default()
                                    })
                                    .collect();
                                matches.push((m.start(), m.end(), m.as_str().to_string(), groups));
                                if !global {
                                    break;
                                }
                            }
                        }
                    }
                }
                (matches, global)
            };
            let _ = global;
            if matches.is_empty() {
                return Ok(Value::from_string(s));
            }
            let mut out = String::new();
            let mut last = 0usize;
            for (start, end, full, groups) in matches {
                out.push_str(&s[last..start]);
                if is_fn {
                    let mut call_args = vec![Value::from_string(full)];
                    for g in groups {
                        call_args.push(Value::from_string(g));
                    }
                    call_args.push(Value::Integer(start as i64));
                    call_args.push(Value::from_string(s.clone()));
                    let rep = interp.call_value(&replacer, &Value::Undefined, &call_args)?;
                    out.push_str(&to_string_value(interp, &rep));
                } else {
                    out.push_str(&to_string_value(interp, &replacer));
                }
                last = end;
            }
            out.push_str(&s[last..]);
            Ok(Value::from_string(out))
        }
        Value::String(pattern) => {
            if is_fn {
                if let Some(pos) = s.find(pattern.as_ref()) {
                    let end = pos + pattern.len();
                    let call_args = vec![
                        Value::from_string(pattern.to_string()),
                        Value::Integer(pos as i64),
                        Value::from_string(s.clone()),
                    ];
                    let rep = interp.call_value(&replacer, &Value::Undefined, &call_args)?;
                    let rep_s = to_string_value(interp, &rep);
                    return Ok(Value::from_string(format!(
                        "{}{}{}",
                        &s[..pos],
                        rep_s,
                        &s[end..]
                    )));
                }
                return Ok(Value::from_string(s));
            }
            let replacement = to_string_value(interp, &replacer);
            match s.find(pattern.as_ref()) {
                Some(pos) => {
                    let end = pos + pattern.len();
                    Ok(Value::from_string(format!(
                        "{}{}{}",
                        &s[..pos],
                        replacement,
                        &s[end..]
                    )))
                }
                None => Ok(Value::from_string(s)),
            }
        }
        _ => Ok(Value::from_string(s)),
    }
}

pub(super) fn native_string_split(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let s = get_string(this).unwrap_or_default();
    let sep = match args.first() {
        Some(Value::String(ss)) => ss.as_ref(),
        _ => {
            return {
                let heap_idx = interp.heap.len();
                interp.heap.push(crate::vm::interpreter::HeapValue::Array(
                    crate::vm::interpreter::JsArray {
                        elements: vec![Value::from_string(s)],
                    },
                ));
                Ok(Value::Array(heap_idx))
            }
        }
    };
    let parts: Vec<Value> = s
        .split(sep)
        .map(|p| Value::from_string(p.to_string()))
        .collect();
    let heap_idx = interp.heap.len();
    interp.heap.push(crate::vm::interpreter::HeapValue::Array(
        crate::vm::interpreter::JsArray { elements: parts },
    ));
    Ok(Value::Array(heap_idx))
}

pub(super) fn native_string_trim(
    _interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let s = get_string(this).unwrap_or_default();
    Ok(Value::from_string(s.trim().to_string()))
}

pub(super) fn native_string_trim_start(
    _interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let s = get_string(this).unwrap_or_default();
    Ok(Value::from_string(s.trim_start().to_string()))
}

pub(super) fn native_string_trim_end(
    _interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let s = get_string(this).unwrap_or_default();
    Ok(Value::from_string(s.trim_end().to_string()))
}

pub(super) fn native_string_to_lower_case(
    _interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let s = get_string(this).unwrap_or_default();
    Ok(Value::from_string(s.to_lowercase()))
}

pub(super) fn native_string_to_upper_case(
    _interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let s = get_string(this).unwrap_or_default();
    Ok(Value::from_string(s.to_uppercase()))
}

pub(super) fn native_string_starts_with(
    _interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let s = get_string(this).unwrap_or_default();
    let prefix = match args.first() {
        Some(Value::String(ss)) => ss.as_ref(),
        _ => return Ok(Value::Boolean(false)),
    };
    Ok(Value::Boolean(s.starts_with(prefix)))
}

pub(super) fn native_string_ends_with(
    _interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let s = get_string(this).unwrap_or_default();
    let suffix = match args.first() {
        Some(Value::String(ss)) => ss.as_ref(),
        _ => return Ok(Value::Boolean(false)),
    };
    Ok(Value::Boolean(s.ends_with(suffix)))
}

pub(super) fn native_string_repeat(
    _interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let s = get_string(this).unwrap_or_default();
    let count = args.first().map(|v| to_f64(v) as usize).unwrap_or(0);
    if count == 0 || s.is_empty() {
        return Ok(Value::from_string("".to_string()));
    }
    let result: String = s.repeat(count);
    Ok(Value::from_string(result))
}

fn pad_string(s: &str, target_len: usize, pad_char: char, pad_start: bool) -> String {
    if s.len() >= target_len {
        return s.to_string();
    }
    let pad_count = target_len - s.len();
    let padding: String = std::iter::repeat_n(pad_char, pad_count).collect();
    if pad_start {
        format!("{}{}", padding, s)
    } else {
        format!("{}{}", s, padding)
    }
}

pub(super) fn native_string_pad_start(
    _interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let s = get_string(this).unwrap_or_default();
    let target_len = args.first().map(|v| to_f64(v) as usize).unwrap_or(0);
    let pad_char = match args.get(1) {
        Some(Value::String(ss)) => ss.chars().next().unwrap_or(' '),
        _ => ' ',
    };
    Ok(Value::from_string(pad_string(
        &s, target_len, pad_char, true,
    )))
}

pub(super) fn native_string_pad_end(
    _interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let s = get_string(this).unwrap_or_default();
    let target_len = args.first().map(|v| to_f64(v) as usize).unwrap_or(0);
    let pad_char = match args.get(1) {
        Some(Value::String(ss)) => ss.chars().next().unwrap_or(' '),
        _ => ' ',
    };
    Ok(Value::from_string(pad_string(
        &s, target_len, pad_char, false,
    )))
}

pub(super) fn native_string_match_all(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    use crate::vm::interpreter::{HeapValue, JsArray};

    let s = get_string(this).unwrap_or_default();
    let pattern = args.first().cloned().unwrap_or(Value::Undefined);

    match &pattern {
        Value::RegExp(idx) => {
            if let HeapValue::RegExp(re) = &interp.heap[*idx] {
                if let Some(ref compiled) = re.compiled {
                    let matches: Vec<String> = match compiled {
                        crate::vm::interpreter::JsCompiledRegex::Simple(r) => {
                            r.find_iter(&s).map(|m| m.as_str().to_string()).collect()
                        }
                        crate::vm::interpreter::JsCompiledRegex::Advanced(r) => r
                            .find_iter(&s)
                            .filter_map(|m| m.ok())
                            .map(|m| m.as_str().to_string())
                            .collect(),
                    };
                    let mut results = Vec::with_capacity(matches.len());
                    for match_str in matches {
                        let arr_idx = interp.gc.allocate(
                            &mut interp.heap,
                            HeapValue::Array(JsArray {
                                elements: vec![Value::from_string(match_str)],
                            }),
                        );
                        results.push(Value::Array(arr_idx));
                    }
                    let result_arr = interp.gc.allocate(
                        &mut interp.heap,
                        HeapValue::Array(JsArray { elements: results }),
                    );
                    return Ok(Value::Array(result_arr));
                }
            }
            let empty = interp.gc.allocate(
                &mut interp.heap,
                HeapValue::Array(JsArray { elements: vec![] }),
            );
            Ok(Value::Array(empty))
        }
        Value::String(p) => {
            let re = match fancy_regex::Regex::new(p) {
                Ok(r) => r,
                Err(_) => {
                    let empty = interp.gc.allocate(
                        &mut interp.heap,
                        HeapValue::Array(JsArray { elements: vec![] }),
                    );
                    return Ok(Value::Array(empty));
                }
            };
            let mut results = Vec::new();
            for m in re.find_iter(&s).flatten() {
                let match_str = m.as_str().to_string();
                let arr_idx = interp.gc.allocate(
                    &mut interp.heap,
                    HeapValue::Array(JsArray {
                        elements: vec![Value::from_string(match_str)],
                    }),
                );
                results.push(Value::Array(arr_idx));
            }
            let result_arr = interp.gc.allocate(
                &mut interp.heap,
                HeapValue::Array(JsArray { elements: results }),
            );
            Ok(Value::Array(result_arr))
        }
        _ => {
            let empty = interp.gc.allocate(
                &mut interp.heap,
                HeapValue::Array(JsArray { elements: vec![] }),
            );
            Ok(Value::Array(empty))
        }
    }
}
