use crate::errors::{Error, Result};
use crate::objects::Value;
use crate::vm::interpreter::{HeapValue, Interpreter, JsRegExp};
use crate::well_known as wk;

fn get_regexp_idx(this: &Value) -> Option<usize> {
    match this {
        Value::RegExp(idx) => Some(*idx),
        _ => None,
    }
}

macro_rules! with_regexp {
    ($interp:expr, $this:expr, $body:expr) => {
        match get_regexp_idx($this) {
            Some(idx) => {
                if let HeapValue::RegExp(ref regexp) = $interp.heap[idx] {
                    $body(regexp)
                } else {
                    Err(Error::TypeError("Not a RegExp".into()))
                }
            }
            None => Err(Error::TypeError("Not a RegExp".into())),
        }
    };
}

// Constructor

pub(super) fn native_regexp_constructor(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let pattern = match args.first() {
        Some(Value::RegExp(idx)) => {
            if let HeapValue::RegExp(ref re) = interp.heap[*idx] {
                re.source.clone()
            } else {
                "".to_string()
            }
        }
        Some(v) => interp.to_string_coerce(v),
        None => "".to_string(),
    };

    let flags = match args.get(1) {
        Some(v) => interp.to_string_coerce(v),
        None => "".to_string(),
    };

    let regexp = JsRegExp::new(&pattern, &flags)
        .map_err(|e| Error::TypeError(format!("Invalid RegExp: {}", e)))?;

    let idx = interp.heap.len();
    interp.heap.push(HeapValue::RegExp(regexp));
    Ok(Value::RegExp(idx))
}

// Instance methods

pub(super) fn native_regexp_test(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    // Fast path: if the input is already a Value::String, borrow it directly
    // to avoid the 24-byte String clone that to_string_coerce would do.
    let input_owned;
    let input: &str = match args.first() {
        Some(Value::String(s)) => s.as_str(),
        Some(v) => {
            input_owned = interp.to_string_coerce(v);
            input_owned.as_str()
        }
        None => return Ok(Value::Boolean(false)),
    };

    // Phase 3.4: Use cached test for non-global, non-sticky regexps to avoid
    // re-running the regex engine on repeated calls with the same input.
    let idx = match get_regexp_idx(this) {
        Some(idx) => idx,
        None => return Err(Error::TypeError("Not a RegExp".into())),
    };

    let result = if let HeapValue::RegExp(ref mut regexp) = interp.heap[idx] {
        if !regexp.global && !regexp.sticky {
            regexp.test_cached(input)
        } else {
            regexp.test(input)
        }
    } else {
        return Err(Error::TypeError("Not a RegExp".into()));
    };

    Ok(Value::Boolean(result))
}

pub(super) fn native_regexp_exec(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    // Fast path: if the input is already a Value::String, borrow it directly
    // to avoid the 24-byte String clone that to_string_coerce would do.
    let input_owned;
    let input: &str = match args.first() {
        Some(Value::String(s)) => s.as_str(),
        Some(v) => {
            input_owned = interp.to_string_coerce(v);
            input_owned.as_str()
        }
        None => return Ok(Value::Null),
    };

    let idx = match get_regexp_idx(this) {
        Some(idx) => idx,
        None => return Err(Error::TypeError("Not a RegExp".into())),
    };

    if let HeapValue::RegExp(ref mut regexp) = interp.heap[idx] {
        if regexp.global || regexp.sticky {
            let start = regexp.last_index as usize;
            if start > input.len() {
                regexp.last_index = 0.0;
                return Ok(Value::Null);
            }
            match regexp.exec_at(input, start) {
                Some((caps, end)) => {
                    if end <= start {
                        regexp.last_index = (start + 1) as f64;
                    } else {
                        regexp.last_index = end as f64;
                    }
                    let mut elements: Vec<Value> = Vec::with_capacity(caps.len());
                    for s in caps {
                        elements.push(Value::String(s));
                    }
                    let arr_idx = interp.heap.len();
                    interp
                        .heap
                        .push(HeapValue::Array(crate::vm::interpreter::JsArray {
                            elements,
                        }));
                    Ok(Value::Array(arr_idx))
                }
                None => {
                    regexp.last_index = 0.0;
                    Ok(Value::Null)
                }
            }
        } else {
            match regexp.exec_with_groups(input) {
                Some((captures, named_groups, match_start)) => {
                    let mut props = rustc_hash::FxHashMap::default();
                    for (i, cap) in captures.iter().enumerate() {
                        props.insert(i.to_string(), Value::String(cap.clone()));
                    }
                    props.insert(wk::LENGTH.to_string(), Value::Float(captures.len() as f64));
                    props.insert("index".to_string(), Value::Float(match_start as f64));
                    props.insert("input".to_string(), Value::String(input.to_string()));
                    if named_groups.is_empty() {
                        props.insert("groups".to_string(), Value::Undefined);
                    } else {
                        let mut group_props = rustc_hash::FxHashMap::default();
                        for (k, v) in named_groups {
                            group_props.insert(k, Value::String(v));
                        }
                        let groups_idx = interp.heap.len();
                        interp
                            .heap
                            .push(HeapValue::Object(crate::vm::interpreter::JsObject {
                                properties: group_props.into(),
                                prototype: None,
                                extensible: true,
                            }));
                        props.insert("groups".to_string(), Value::Object(groups_idx));
                    }
                    let obj_idx = interp.heap.len();
                    interp
                        .heap
                        .push(HeapValue::Object(crate::vm::interpreter::JsObject {
                            properties: props.into(),
                            prototype: None,
                            extensible: true,
                        }));
                    Ok(Value::Object(obj_idx))
                }
                None => Ok(Value::Null),
            }
        }
    } else {
        Err(Error::TypeError("Not a RegExp".into()))
    }
}

pub(super) fn native_regexp_to_string(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    with_regexp!(interp, this, |regexp: &JsRegExp| {
        Ok(Value::String(format!(
            "/{}/{}",
            regexp.source, regexp.flags
        )))
    })
}

// Property access helpers for RegExp properties

pub(super) fn native_regexp_source(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    with_regexp!(interp, this, |regexp: &JsRegExp| {
        Ok(Value::String(regexp.source.clone()))
    })
}

pub(super) fn native_regexp_flags(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    with_regexp!(interp, this, |regexp: &JsRegExp| {
        Ok(Value::String(regexp.flags.clone()))
    })
}

pub(super) fn native_regexp_global(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    with_regexp!(interp, this, |regexp: &JsRegExp| {
        Ok(Value::Boolean(regexp.global))
    })
}

pub(super) fn native_regexp_ignore_case(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    with_regexp!(interp, this, |regexp: &JsRegExp| {
        Ok(Value::Boolean(regexp.ignore_case))
    })
}

pub(super) fn native_regexp_multiline(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    with_regexp!(interp, this, |regexp: &JsRegExp| {
        Ok(Value::Boolean(regexp.multiline))
    })
}

pub(super) fn native_regexp_dot_all(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    with_regexp!(interp, this, |regexp: &JsRegExp| {
        Ok(Value::Boolean(regexp.dot_all))
    })
}

pub(super) fn native_regexp_unicode(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    with_regexp!(interp, this, |regexp: &JsRegExp| {
        Ok(Value::Boolean(regexp.unicode))
    })
}

pub(super) fn native_regexp_sticky(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    with_regexp!(interp, this, |regexp: &JsRegExp| {
        Ok(Value::Boolean(regexp.sticky))
    })
}

pub(super) fn native_regexp_last_index(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    with_regexp!(interp, this, |regexp: &JsRegExp| {
        Ok(Value::Float(regexp.last_index))
    })
}

// String methods that accept RegExp

pub(super) fn native_string_match(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let input = match _this {
        Value::String(s) => s.clone(),
        _ => interp.to_string_coerce(_this),
    };

    let regexp_idx = match args.first() {
        Some(Value::RegExp(idx)) => *idx,
        _ => return Ok(Value::Null),
    };

    if let HeapValue::RegExp(ref regexp) = interp.heap[regexp_idx] {
        if regexp.global {
            let matches = regexp.find_all(&input);
            if matches.is_empty() {
                return Ok(Value::Null);
            }
            let elements: Vec<Value> = matches.into_iter().map(Value::String).collect();
            let arr_idx = interp.heap.len();
            interp
                .heap
                .push(HeapValue::Array(crate::vm::interpreter::JsArray {
                    elements,
                }));
            Ok(Value::Array(arr_idx))
        } else {
            match regexp.exec_with_groups(&input) {
                Some((captures, named_groups, match_start)) => {
                    let mut props = rustc_hash::FxHashMap::default();
                    for (i, cap) in captures.iter().enumerate() {
                        props.insert(i.to_string(), Value::String(cap.clone()));
                    }
                    props.insert(wk::LENGTH.to_string(), Value::Float(captures.len() as f64));
                    props.insert("index".to_string(), Value::Float(match_start as f64));
                    props.insert("input".to_string(), Value::String(input));
                    if named_groups.is_empty() {
                        props.insert("groups".to_string(), Value::Undefined);
                    } else {
                        let mut group_props = rustc_hash::FxHashMap::default();
                        for (k, v) in named_groups {
                            group_props.insert(k, Value::String(v));
                        }
                        let groups_idx = interp.heap.len();
                        interp
                            .heap
                            .push(HeapValue::Object(crate::vm::interpreter::JsObject {
                                properties: group_props.into(),
                                prototype: None,
                                extensible: true,
                            }));
                        props.insert("groups".to_string(), Value::Object(groups_idx));
                    }
                    let obj_idx = interp.heap.len();
                    interp
                        .heap
                        .push(HeapValue::Object(crate::vm::interpreter::JsObject {
                            properties: props.into(),
                            prototype: None,
                            extensible: true,
                        }));
                    Ok(Value::Object(obj_idx))
                }
                None => Ok(Value::Null),
            }
        }
    } else {
        Err(Error::TypeError("Not a RegExp".into()))
    }
}

pub(super) fn native_string_replace(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let input = match _this {
        Value::String(s) => s.clone(),
        _ => interp.to_string_coerce(_this),
    };

    let search = args.first().cloned().unwrap_or(Value::Undefined);
    let replacement = match args.get(1) {
        Some(v) => interp.to_string_coerce(v),
        None => wk::UNDEFINED.to_string(),
    };

    match search {
        Value::RegExp(idx) => {
            let result = if let HeapValue::RegExp(ref regexp) = interp.heap[idx] {
                regexp.replace(&input, &replacement)
            } else {
                return Err(Error::TypeError("Not a RegExp".into()));
            };
            Ok(Value::String(result))
        }
        Value::String(search_str) => {
            Ok(Value::String(input.replacen(&search_str, &replacement, 1)))
        }
        _ => Ok(Value::String(input)),
    }
}

pub(super) fn native_string_search(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let input = match _this {
        Value::String(s) => s.clone(),
        _ => interp.to_string_coerce(_this),
    };

    let search = args.first().cloned().unwrap_or(Value::Undefined);

    match search {
        Value::RegExp(idx) => {
            let result = if let HeapValue::RegExp(ref regexp) = interp.heap[idx] {
                regexp.search(&input)
            } else {
                return Err(Error::TypeError("Not a RegExp".into()));
            };
            Ok(Value::Integer(result))
        }
        Value::String(search_str) => Ok(Value::Integer(
            input.find(&search_str).map(|i| i as i64).unwrap_or(-1),
        )),
        _ => Ok(Value::Integer(-1)),
    }
}
