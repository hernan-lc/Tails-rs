use crate::errors::{Error, Result};
use crate::objects::Value;
use crate::vm::interpreter::Interpreter;
use crate::well_known as wk;

use super::helpers::to_f64;

pub(super) fn native_number_constructor(
    _interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let value = args.first().cloned().unwrap_or(Value::Undefined);
    match value {
        Value::Integer(n) => Ok(Value::Float(n as f64)),
        Value::Float(_) => Ok(value),
        Value::String(s) => match s.trim().parse::<f64>() {
            Ok(n) => Ok(Value::Float(n)),
            Err(_) => Ok(Value::Float(f64::NAN)),
        },
        Value::Boolean(b) => Ok(Value::Float(if b { 1.0 } else { 0.0 })),
        Value::BigInt(n) => Ok(Value::Float(n as f64)),
        Value::Null => Ok(Value::Float(0.0)),
        Value::Undefined => Ok(Value::Float(f64::NAN)),
        _ => Ok(Value::Float(f64::NAN)),
    }
}

pub(super) fn native_boolean_constructor(
    _interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let value = args.first().cloned().unwrap_or(Value::Undefined);
    Ok(Value::Boolean(truthy(&value)))
}

fn truthy(v: &Value) -> bool {
    match v {
        Value::Undefined | Value::Null => false,
        Value::Boolean(b) => *b,
        Value::Integer(n) => *n != 0,
        Value::Float(n) => *n != 0.0 && !n.is_nan(),
        Value::String(s) => !s.is_empty(),
        Value::Cons(c) => !c.flatten().is_empty(),
        Value::BigInt(n) => *n != 0,
        _ => true,
    }
}

pub(super) fn native_bigint_constructor(
    _interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let value = args.first().cloned().unwrap_or(Value::Undefined);
    match value {
        Value::BigInt(_) => Ok(value),
        Value::Integer(n) => Ok(Value::BigInt(n as i128)),
        Value::Float(n) => Ok(Value::BigInt(n as i128)),
        Value::String(s) => {
            let parsed: i128 = s
                .parse()
                .map_err(|_| Error::TypeError(format!("Cannot convert '{}' to BigInt", s)))?;
            Ok(Value::BigInt(parsed))
        }
        Value::Boolean(b) => Ok(Value::BigInt(if b { 1 } else { 0 })),
        _ => Err(Error::TypeError("Cannot convert value to BigInt".into())),
    }
}

pub(super) fn native_number_to_fixed(
    _interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let n = match this {
        Value::Float(f) => *f,
        Value::Integer(i) => *i as f64,
        _ => return Ok(Value::from_string(wk::NAN.to_string())),
    };
    let digits = args.first().map(|v| to_f64(v) as u32).unwrap_or(0);
    if n.is_nan() {
        return Ok(Value::from_string(wk::NAN.to_string()));
    }
    if n.is_infinite() {
        return Ok(Value::from_string(if n.is_sign_positive() {
                wk::INFINITY.to_string()
            } else {
                format!("-{}", wk::INFINITY)
            }
            .to_string(),));
    }
    let formatted = format!("{:.*}", digits as usize, n);
    Ok(Value::from_string(formatted.into()))
}

pub(super) fn native_number_to_string(
    _interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let radix = args.first().map(|v| to_f64(v) as u32).unwrap_or(10);
    match this {
        Value::Float(f) => {
            if *f == 0.0 && f.is_sign_negative() {
                return Ok(Value::from_string("-0".to_string()));
            }
            if radix == 10 {
                Ok(Value::from_string(format_number(*f).into()))
            } else {
                Ok(Value::from_string(format_int(*f as i64, radix).into()))
            }
        }
        Value::Integer(i) => {
            if radix == 10 {
                Ok(Value::from_string(i.to_string()))
            } else {
                Ok(Value::from_string(format_int(*i, radix).into()))
            }
        }
        _ => Ok(Value::from_string(wk::NAN.to_string())),
    }
}

pub(super) fn native_number_value_of(
    _interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    Ok(this.clone())
}

pub(super) fn native_boolean_to_string(
    _interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    match this {
        Value::Boolean(b) => Ok(Value::from_string(b.to_string())),
        _ => Ok(Value::from_string(wk::TRUE.to_string())),
    }
}

pub(super) fn native_boolean_value_of(
    _interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    Ok(this.clone())
}

pub(super) fn native_number_to_exponential(
    _interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let n = match this {
        Value::Float(f) => *f,
        Value::Integer(i) => *i as f64,
        _ => return Ok(Value::from_string(wk::NAN.to_string())),
    };
    let digits = match args.first() {
        Some(v) => to_f64(v) as usize,
        None => 20,
    };
    if n.is_nan() {
        return Ok(Value::from_string(wk::NAN.to_string()));
    }
    if n.is_infinite() {
        return Ok(Value::from_string(if n.is_sign_positive() {
                wk::INFINITY.to_string()
            } else {
                format!("-{}", wk::INFINITY)
            }
            .to_string(),));
    }
    let formatted = format!("{:.*e}", digits, n);
    let fixed = formatted.replacen(
        'e',
        if formatted.contains("e+") || formatted.contains("e-") {
            "e"
        } else {
            "e+"
        },
        1,
    );
    // Strip trailing zeros after decimal point (but keep at least one digit)
    let result = if args.is_empty() {
        if let Some(dot_pos) = fixed.find('.') {
            if let Some(e_pos) = fixed[dot_pos..].find('e') {
                let mut end = dot_pos + e_pos;
                while end > dot_pos + 1 && fixed.as_bytes()[end - 1] == b'0' {
                    end -= 1;
                }
                format!("{}{}", &fixed[..end], &fixed[dot_pos + e_pos..])
            } else {
                fixed
            }
        } else {
            fixed
        }
    } else {
        fixed
    };
    Ok(Value::from_string(result.into()))
}

pub(super) fn native_number_to_precision(
    _interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let n = match this {
        Value::Float(f) => *f,
        Value::Integer(i) => *i as f64,
        _ => return Ok(Value::from_string(wk::NAN.to_string())),
    };
    let precision = args.first().map(|v| to_f64(v) as usize).unwrap_or(0);
    if n.is_nan() {
        return Ok(Value::from_string(wk::NAN.to_string()));
    }
    if n.is_infinite() {
        return Ok(Value::from_string(if n.is_sign_positive() {
                wk::INFINITY.to_string()
            } else {
                format!("-{}", wk::INFINITY)
            }
            .to_string(),));
    }
    if precision == 0 {
        return Ok(Value::from_string(format_number(n).into()));
    }
    let abs_n = n.abs();
    let int_part = abs_n.floor();
    let int_digits = if int_part == 0.0 {
        0
    } else {
        (int_part.log10().floor() as usize) + 1
    };
    let decimal_places = precision.saturating_sub(int_digits);
    let factor = 10.0_f64.powi(decimal_places as i32);
    let rounded = (n * factor).round() / factor;
    let formatted = format!("{:.*}", decimal_places, rounded);
    Ok(Value::from_string(formatted.into()))
}

pub(super) fn native_number_is_integer(
    _interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let n = to_f64(args.first().unwrap_or(&Value::Undefined));
    Ok(Value::Boolean(n.fract() == 0.0 && n.is_finite()))
}

pub(super) fn native_number_is_safe_integer(
    _interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let n = to_f64(args.first().unwrap_or(&Value::Undefined));
    let max_safe = (2_i64.pow(53) - 1) as f64;
    Ok(Value::Boolean(
        n.fract() == 0.0 && n.is_finite() && n.abs() <= max_safe,
    ))
}

pub(super) fn native_number_parse_float(
    _interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let s = match args.first().unwrap_or(&Value::Undefined) {
        Value::String(s) => s.to_string(),
        other => format!("{}", to_f64(other)),
    };
    match s.trim().parse::<f64>() {
        Ok(n) => Ok(Value::Float(n)),
        Err(_) => Ok(Value::Float(f64::NAN)),
    }
}

fn format_number(n: f64) -> String {
    if n == 0.0 && n.is_sign_negative() {
        return "-0".to_string();
    }
    let s = format!("{}", n);
    if s.contains('.') || s.contains('e') || s.contains('E') {
        return s;
    }
    s
}

fn format_int(n: i64, radix: u32) -> String {
    if radix == 10 {
        return n.to_string();
    }
    if !(2..=36).contains(&radix) {
        return wk::NAN.to_string();
    }
    if n == 0 {
        return "0".to_string();
    }
    let negative = n < 0;
    let mut num = n.unsigned_abs();
    let mut digits = Vec::new();
    let chars = "0123456789abcdefghijklmnopqrstuvwxyz";
    while num > 0 {
        digits.push(chars.chars().nth((num % radix as u64) as usize).unwrap());
        num /= radix as u64;
    }
    digits.reverse();
    let result: String = digits.iter().collect();
    if negative {
        format!("-{}", result)
    } else {
        result
    }
}
