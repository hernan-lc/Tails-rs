use crate::errors::Result;
use crate::objects::Value;
use crate::vm::interpreter::Interpreter;

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
