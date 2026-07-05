use crate::errors::{Error, Result};
use crate::objects::js_date::JsDate;
use crate::objects::Value;
use crate::vm::interpreter::{HeapValue, Interpreter};

use super::helpers::to_f64;

fn get_date_idx(this: &Value) -> Option<usize> {
    match this {
        Value::Date(idx) => Some(*idx),
        _ => None,
    }
}

macro_rules! with_date {
    ($interp:expr, $this:expr, $body:expr) => {
        match get_date_idx($this) {
            Some(idx) => {
                if let HeapValue::Date(ref mut date) = $interp.heap[idx] {
                    $body(date)
                } else {
                    Err(Error::TypeError("Not a Date".into()))
                }
            }
            None => Err(Error::TypeError("Not a Date".into())),
        }
    };
}

macro_rules! date_getter {
    ($name:ident, $method:ident) => {
        pub(super) fn $name(
            interp: &mut Interpreter,
            this: &Value,
            _args: &[Value],
        ) -> Result<Value> {
            with_date!(interp, this, |date: &JsDate| Ok(Value::Float(
                date.$method()
            )))
        }
    };
}

date_getter!(native_date_get_full_year, get_full_year);
date_getter!(native_date_get_month, get_month);
date_getter!(native_date_get_date, get_date);
date_getter!(native_date_get_day, get_day);
date_getter!(native_date_get_hours, get_hours);
date_getter!(native_date_get_minutes, get_minutes);
date_getter!(native_date_get_seconds, get_seconds);
date_getter!(native_date_get_milliseconds, get_milliseconds);
date_getter!(native_date_get_timezone_offset, get_timezone_offset);
date_getter!(native_date_get_utc_full_year, get_utc_full_year);
date_getter!(native_date_get_utc_month, get_utc_month);
date_getter!(native_date_get_utc_date, get_utc_date);
date_getter!(native_date_get_utc_day, get_utc_day);
date_getter!(native_date_get_utc_hours, get_utc_hours);
date_getter!(native_date_get_utc_minutes, get_utc_minutes);
date_getter!(native_date_get_utc_seconds, get_utc_seconds);
date_getter!(native_date_get_utc_milliseconds, get_utc_milliseconds);

// Constructors

pub(super) fn native_date_constructor(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let date = if args.is_empty() {
        JsDate::now()
    } else if args.len() == 1 {
        let arg = &args[0];
        match arg {
            Value::String(s) => JsDate::from_string(s).unwrap_or(JsDate { utc_ms: f64::NAN }),
            Value::Cons(c) => {
                let s = c.flatten();
                JsDate::from_string(&s).unwrap_or(JsDate { utc_ms: f64::NAN })
            }
            Value::Float(f) => JsDate::from_millis(*f),
            Value::Integer(n) => JsDate::from_millis(*n as f64),
            _ => JsDate::now(),
        }
    } else {
        let y = to_f64(&args[0]);
        let m = to_f64(&args[1]);
        let d = args.get(2).map(to_f64).unwrap_or(1.0);
        let h = args.get(3).map(to_f64).unwrap_or(0.0);
        let min = args.get(4).map(to_f64).unwrap_or(0.0);
        let s = args.get(5).map(to_f64).unwrap_or(0.0);
        let ms = args.get(6).map(to_f64).unwrap_or(0.0);
        JsDate::from_components(y, m, d, h, min, s, ms)
    };

    let idx = interp.heap.len();
    interp.heap.push(HeapValue::Date(date));
    Ok(Value::Date(idx))
}

pub(super) fn native_date_now(
    _interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as f64;
    Ok(Value::Float(ms))
}

pub(super) fn native_date_parse(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let s = match args.first() {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Cons(c)) => c.flatten(),
        Some(v) => interp.to_string_coerce(v),
        None => return Ok(Value::Float(f64::NAN)),
    };

    match JsDate::from_string(&s) {
        Some(date) => Ok(Value::Float(date.utc_ms)),
        None => Ok(Value::Float(f64::NAN)),
    }
}

pub(super) fn native_date_utc(
    _interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let y = to_f64(args.first().unwrap_or(&Value::Undefined));
    let m = to_f64(args.get(1).unwrap_or(&Value::Undefined));
    let d = args.get(2).map(to_f64).unwrap_or(1.0);
    let h = args.get(3).map(to_f64).unwrap_or(0.0);
    let min = args.get(4).map(to_f64).unwrap_or(0.0);
    let s = args.get(5).map(to_f64).unwrap_or(0.0);
    let ms = args.get(6).map(to_f64).unwrap_or(0.0);
    let date = JsDate::from_components(y, m, d, h, min, s, ms);
    Ok(Value::Float(date.utc_ms))
}

// Instance methods - getters

pub(super) fn native_date_get_time(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    with_date!(interp, this, |date: &JsDate| Ok(Value::Float(date.utc_ms)))
}

// UTC getters

// Setters

pub(super) fn native_date_set_time(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let ms = to_f64(args.first().unwrap_or(&Value::Undefined));
    with_date!(interp, this, |date: &mut JsDate| Ok(Value::Float(
        date.set_time(ms)
    )))
}

pub(super) fn native_date_set_utc_full_year(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let y = to_f64(args.first().unwrap_or(&Value::Undefined));
    let m = args.get(1).map(to_f64);
    let d = args.get(2).map(to_f64);
    with_date!(interp, this, |date: &mut JsDate| Ok(Value::Float(
        date.set_utc_full_year(y, m, d)
    )))
}

pub(super) fn native_date_set_utc_month(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let m = to_f64(args.first().unwrap_or(&Value::Undefined));
    let d = args.get(1).map(to_f64);
    with_date!(interp, this, |date: &mut JsDate| Ok(Value::Float(
        date.set_utc_month(m, d)
    )))
}

pub(super) fn native_date_set_utc_date(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let d = to_f64(args.first().unwrap_or(&Value::Undefined));
    with_date!(interp, this, |date: &mut JsDate| Ok(Value::Float(
        date.set_utc_date(d)
    )))
}

pub(super) fn native_date_set_utc_hours(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let h = to_f64(args.first().unwrap_or(&Value::Undefined));
    let min = args.get(1).map(to_f64);
    let s = args.get(2).map(to_f64);
    let ms = args.get(3).map(to_f64);
    with_date!(interp, this, |date: &mut JsDate| Ok(Value::Float(
        date.set_utc_hours(h, min, s, ms)
    )))
}

pub(super) fn native_date_set_utc_minutes(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let min = to_f64(args.first().unwrap_or(&Value::Undefined));
    let s = args.get(1).map(to_f64);
    let ms = args.get(2).map(to_f64);
    with_date!(interp, this, |date: &mut JsDate| Ok(Value::Float(
        date.set_utc_minutes(min, s, ms)
    )))
}

pub(super) fn native_date_set_utc_seconds(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let s = to_f64(args.first().unwrap_or(&Value::Undefined));
    let ms = args.get(1).map(to_f64);
    with_date!(interp, this, |date: &mut JsDate| Ok(Value::Float(
        date.set_utc_seconds(s, ms)
    )))
}

pub(super) fn native_date_set_utc_milliseconds(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let ms = to_f64(args.first().unwrap_or(&Value::Undefined));
    with_date!(interp, this, |date: &mut JsDate| Ok(Value::Float(
        date.set_utc_milliseconds(ms)
    )))
}

// Local setters (delegate to UTC for now)

pub(super) fn native_date_set_full_year(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let y = to_f64(args.first().unwrap_or(&Value::Undefined));
    let m = args.get(1).map(to_f64);
    let d = args.get(2).map(to_f64);
    with_date!(interp, this, |date: &mut JsDate| Ok(Value::Float(
        date.set_full_year(y, m, d)
    )))
}

pub(super) fn native_date_set_month(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let m = to_f64(args.first().unwrap_or(&Value::Undefined));
    let d = args.get(1).map(to_f64);
    with_date!(interp, this, |date: &mut JsDate| Ok(Value::Float(
        date.set_month(m, d)
    )))
}

pub(super) fn native_date_set_date(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let d = to_f64(args.first().unwrap_or(&Value::Undefined));
    with_date!(interp, this, |date: &mut JsDate| Ok(Value::Float(
        date.set_date(d)
    )))
}

pub(super) fn native_date_set_hours(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let h = to_f64(args.first().unwrap_or(&Value::Undefined));
    let min = args.get(1).map(to_f64);
    let s = args.get(2).map(to_f64);
    let ms = args.get(3).map(to_f64);
    with_date!(interp, this, |date: &mut JsDate| Ok(Value::Float(
        date.set_hours(h, min, s, ms)
    )))
}

pub(super) fn native_date_set_minutes(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let min = to_f64(args.first().unwrap_or(&Value::Undefined));
    let s = args.get(1).map(to_f64);
    let ms = args.get(2).map(to_f64);
    with_date!(interp, this, |date: &mut JsDate| Ok(Value::Float(
        date.set_minutes(min, s, ms)
    )))
}

pub(super) fn native_date_set_seconds(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let s = to_f64(args.first().unwrap_or(&Value::Undefined));
    let ms = args.get(1).map(to_f64);
    with_date!(interp, this, |date: &mut JsDate| Ok(Value::Float(
        date.set_seconds(s, ms)
    )))
}

pub(super) fn native_date_set_milliseconds(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let ms = to_f64(args.first().unwrap_or(&Value::Undefined));
    with_date!(interp, this, |date: &mut JsDate| Ok(Value::Float(
        date.set_milliseconds(ms)
    )))
}

// String conversion methods

macro_rules! date_to_string_fn {
    ($name:ident, $method:ident) => {
        pub(super) fn $name(
            interp: &mut Interpreter,
            this: &Value,
            _args: &[Value],
        ) -> Result<Value> {
            with_date!(interp, this, |date: &JsDate| {
                Ok(Value::String(date.$method()))
            })
        }
    };
}

date_to_string_fn!(native_date_to_string, to_utc_string);
date_to_string_fn!(native_date_to_iso_string, to_iso_string);
date_to_string_fn!(native_date_to_utc_string, to_utc_string);
date_to_string_fn!(native_date_to_date_string, to_date_string);
date_to_string_fn!(native_date_to_time_string, to_time_string);
date_to_string_fn!(native_date_to_json, to_iso_string);

pub(super) fn native_date_value_of(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    with_date!(interp, this, |date: &JsDate| Ok(Value::Float(date.utc_ms)))
}
