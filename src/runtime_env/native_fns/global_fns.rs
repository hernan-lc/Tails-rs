use crate::errors::Result;
use crate::objects::Value;
use crate::props;
use crate::runtime_env::native_fns::constants as c;
use crate::vm::interpreter::{HeapValue, Interpreter, JsFunction, JsObject, PropertyStorage};
use std::cell::RefCell;
use std::rc::Rc;

use super::helpers::{to_f64, to_i64, to_string_value};


pub(super) fn native_parse_int(
    _interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let s = match args.first() {
        Some(Value::String(s)) => s.to_string(),
        Some(v) => to_string_value(_interp, v),
        None => return Ok(Value::Float(f64::NAN)),
    };
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Ok(Value::Float(f64::NAN));
    }

    let negative = trimmed.starts_with('-');
    let digits = trimmed.trim_start_matches(['-', '+']);

    let provided_radix = args.get(1).map(to_i64).unwrap_or(0);

    let (radix, num_str) = if provided_radix == 0 {
        if digits.starts_with("0x") || digits.starts_with("0X") {
            (16u32, &digits[2..])
        } else if digits.starts_with("0")
            && digits.len() > 1
            && digits.as_bytes()[1].is_ascii_digit()
        {
            (8u32, digits)
        } else {
            (10u32, digits)
        }
    } else if provided_radix == 16 {
        let stripped = digits
            .strip_prefix("0x")
            .or_else(|| digits.strip_prefix("0X"))
            .unwrap_or(digits);
        (16u32, stripped)
    } else {
        (provided_radix as u32, digits)
    };

    if !(2..=36).contains(&radix) {
        return Ok(Value::Float(f64::NAN));
    }

    let mut result: i64 = 0;
    let mut found_digit = false;
    for ch in num_str.chars() {
        let lower = ch.to_ascii_lowercase();
        let digit = match lower {
            '0'..='9' => lower as u32 - '0' as u32,
            'a'..='z' => lower as u32 - 'a' as u32 + 10,
            _ => break,
        };
        if digit >= radix {
            break;
        }
        found_digit = true;
        result = result
            .checked_mul(radix as i64)
            .and_then(|r| r.checked_add(digit as i64))
            .unwrap_or(i64::MAX);
    }

    if !found_digit {
        return Ok(Value::Float(f64::NAN));
    }

    if negative {
        Ok(Value::Float(-result as f64))
    } else {
        Ok(Value::Float(result as f64))
    }
}

pub(super) fn native_parse_float(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let s = match args.first() {
        Some(Value::String(s)) => s.to_string(),
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

pub(super) fn native_is_nan(
    _interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let n = args.first().map(to_f64).unwrap_or(f64::NAN);
    Ok(Value::Boolean(n.is_nan()))
}

pub(super) fn native_is_finite(
    _interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let n = args.first().map(to_f64).unwrap_or(f64::NAN);
    Ok(Value::Boolean(n.is_finite()))
}

pub(super) fn native_set_timeout(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let callback = args.first().cloned().unwrap_or(Value::Undefined);
    let delay = args.get(1).map(to_f64).unwrap_or(0.0);
    let id = interp.async_runtime.enqueue_macrotask(callback, delay);
    Ok(Value::Float(id as f64))
}

pub(super) fn native_set_interval(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let callback = args.first().cloned().unwrap_or(Value::Undefined);
    let delay = args.get(1).map(to_f64).unwrap_or(0.0);
    let id = interp.async_runtime.enqueue_interval(callback, delay);
    Ok(Value::Float(id as f64))
}

pub(super) fn native_clear_timeout(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    if let Some(Value::Float(id)) = args.first() {
        interp.async_runtime.cancel_timer(*id as u32);
    }
    Ok(Value::Undefined)
}

pub(super) fn native_clear_interval(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    if let Some(Value::Float(id)) = args.first() {
        interp.async_runtime.cancel_timer(*id as u32);
    }
    Ok(Value::Undefined)
}

pub(super) fn native_number_parse_int(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    native_parse_int(interp, this, args)
}

pub(super) fn native_number_parse_float(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    native_parse_float(interp, this, args)
}

pub(super) fn native_number_is_nan(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    native_is_nan(interp, this, args)
}

pub(super) fn native_number_is_finite(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    native_is_finite(interp, this, args)
}

/// decodeURIComponent — percent-decode (throws on malformed sequences in real ES;
/// we best-effort decode).
pub(super) fn native_decode_uri_component(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let s = match args.first() {
        Some(v) => to_string_value(interp, v),
        None => "undefined".into(),
    };
    match urlencoding::decode(&s) {
        Ok(decoded) => Ok(Value::from_string(decoded.into_owned())),
        Err(_) => Ok(Value::from_string(s)),
    }
}

/// encodeURIComponent
pub(super) fn native_encode_uri_component(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let s = match args.first() {
        Some(v) => to_string_value(interp, v),
        None => "undefined".into(),
    };
    Ok(Value::from_string(urlencoding::encode(&s).into_owned()))
}

/// decodeURI — same as decodeURIComponent for our purposes (full ES differs on reserved chars).
pub(super) fn native_decode_uri(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    native_decode_uri_component(interp, this, args)
}

/// encodeURI
pub(super) fn native_encode_uri(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    native_encode_uri_component(interp, this, args)
}

/// No-op `debug` package factory: `require('debug')(ns)` returns a logger.
pub(super) fn native_debug_noop(
    _interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    Ok(Value::NativeFunction(
        crate::runtime_env::native_fns::constants::DEBUG_LOGGER_NOOP,
    ))
}

/// Instance logger returned by `require('debug')(namespace)`.
pub(super) fn native_debug_logger_noop(
    _interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    Ok(Value::Undefined)
}

/// Native `get-intrinsic` shim: resolve `%Name%` / `Name.prototype.method`
/// against the interpreter globals and standard prototypes.
pub(super) fn native_get_intrinsic(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let name = match args.first() {
        Some(Value::String(s)) => s.to_string(),
        Some(Value::Cons(c)) => c.flatten(),
        _ => {
            return Err(crate::errors::Error::TypeError(
                "intrinsic name must be a non-empty string".into(),
            ))
        }
    };
    let allow_missing = matches!(args.get(1), Some(Value::Boolean(true)));
    let trimmed = name.trim_matches('%');
    let parts: Vec<&str> = trimmed.split('.').filter(|p| !p.is_empty()).collect();
    if parts.is_empty() {
        if allow_missing {
            return Ok(Value::Undefined);
        }
        return Err(crate::errors::Error::SyntaxError(format!(
            "intrinsic {} does not exist!",
            name
        )));
    }
    let mut cur = match parts[0] {
        "globalThis" | "global" => interp
            .globals
            .get("globalThis")
            .cloned()
            .unwrap_or(Value::Undefined),
        other => interp
            .globals
            .get(other)
            .cloned()
            .unwrap_or(Value::Undefined),
    };
    if matches!(cur, Value::Undefined) {
        if allow_missing {
            return Ok(Value::Undefined);
        }
        return Err(crate::errors::Error::TypeError(format!(
            "intrinsic {} exists, but is not available",
            name
        )));
    }
    for part in parts.iter().skip(1) {
        cur = interp.get_property(&cur, &Value::from_string((*part).to_string()))?;
        if matches!(cur, Value::Undefined) {
            if allow_missing {
                return Ok(Value::Undefined);
            }
            return Err(crate::errors::Error::TypeError(format!(
                "intrinsic {} exists, but is not available",
                name
            )));
        }
    }
    Ok(cur)
}

/// Minimal `eval` — compiles and runs a string expression/statement list.
/// Used by get-intrinsic which only needs the global binding at load time;
/// full ES scope rules are not replicated.
pub(super) fn native_eval(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let source = match args.first() {
        Some(Value::String(s)) => s.to_string(),
        Some(Value::Cons(c)) => c.flatten(),
        Some(v) => return Ok(v.clone()), // non-string: return as-is (ES)
        None => return Ok(Value::Undefined),
    };
    let compiler = crate::compiler::Compiler::new(false);
    let compiled = compiler.compile(&source)?;
    interp.execute(&compiled)
}

/// `diagnostics_channel.channel(name)` — returns a channel object. Since the
/// runtime has no subscribers, `hasSubscribers` is `false` and `publish` is a
/// no-op, which is all fastify requires for its initialization channel.
pub(super) fn native_diagnostics_channel_channel(
    interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    Ok(make_channel_object(interp))
}

/// `diagnostics_channel.tracingChannel(name)` — returns the standard set of
/// tracing sub-channels (`start`, `end`, `asyncStart`, `asyncEnd`, `error`),
/// each a no-op channel object.
pub(super) fn native_diagnostics_channel_tracing_channel(
    interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let start = make_channel_object(interp);
    let end = make_channel_object(interp);
    let async_start = make_channel_object(interp);
    let async_end = make_channel_object(interp);
    let error = make_channel_object(interp);
    let channel_idx = interp.gc.allocate(
        &mut interp.heap,
        HeapValue::Object(JsObject {
            properties: props! {
                "start" => start,
                "end" => end,
                "asyncStart" => async_start,
                "asyncEnd" => async_end,
                "error" => error,
            },
            prototype: None,
            extensible: true,
        }),
    );
    Ok(Value::Object(channel_idx))
}

/// `AsyncResource` constructor (node:async_hooks). fastify only uses it as
/// `new AsyncResource(name, obj).bind(fn)` to bind a callback; we return an
/// object whose `bind` simply returns the passed function unchanged, which is
/// all the runtime needs for content-type-parser's callback handling.
pub(super) fn native_async_resource_constructor(
    interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let obj_idx = interp.gc.allocate(
        &mut interp.heap,
        HeapValue::Object(JsObject {
            properties: props! {
                "bind" => Value::NativeFunction(c::ASYNC_HOOKS_BIND),
                "runInAsyncScope" => Value::NativeFunction(c::ASYNC_HOOKS_BIND),
            },
            prototype: None,
            extensible: true,
        }),
    );
    Ok(Value::Object(obj_idx))
}

/// `AsyncResource#bind(fn)` — returns `fn` unchanged (no async-context wrapping).
pub(super) fn native_async_resource_bind(
    _interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    Ok(args.first().cloned().unwrap_or(Value::Undefined))
}

/// `safe-regex2` shim — always reports a regex as safe (see require_fns.rs).
pub(super) fn native_safe_regex_true(
    _interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    Ok(Value::Boolean(true))
}
/// fastify uses it only for high-resolution timestamps; a monotonic
/// wall-clock reading (seconds since Unix epoch) is sufficient.
pub(super) fn native_perf_hooks_now(
    _interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0);
    Ok(Value::Float(secs * 1000.0))
}

/// Build a no-op channel object: `publish` is a function that does nothing,
/// `hasSubscribers` is `false`, and `subscribe`/`unsubscribe` are no-ops.
fn make_channel_object(interp: &mut Interpreter) -> Value {
    let publish_idx = interp.gc.allocate(
        &mut interp.heap,
        HeapValue::Function(JsFunction {
            name: Some("publish".to_string()),
            params: vec!["message".to_string()],
            rest_param: None,
            bytecode_index: usize::MAX,
            local_count: 0,
            closure: Rc::new(RefCell::new(Vec::new())),
            prototype: None,
            super_class: None,
            properties: PropertyStorage::new(),
            owner_module: interp.current_module.clone(),
            module_scope: None,
            is_generator: false,
            source_file: interp.current_module_path.clone(),
            source_line: None,
            is_arrow: false,
            captured_this: None,
            capture_slots: Vec::new(),
        }),
    );
    let channel_idx = interp.gc.allocate(
        &mut interp.heap,
        HeapValue::Object(JsObject {
            properties: props! {
                "publish" => Value::Function(publish_idx),
                "hasSubscribers" => Value::Boolean(false),
                "subscribe" => Value::NativeFunction(
                    crate::runtime_env::native_fns::constants::DEBUG_LOGGER_NOOP,
                ),
                "unsubscribe" => Value::NativeFunction(
                    crate::runtime_env::native_fns::constants::DEBUG_LOGGER_NOOP,
                ),
            },
            prototype: None,
            extensible: true,
        }),
    );
    Value::Object(channel_idx)
}
