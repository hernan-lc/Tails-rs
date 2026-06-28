use crate::errors::Result;
use crate::objects::Value;
use crate::vm::interpreter::{HeapValue, Interpreter, JsObject};

use super::helpers::{to_f64, to_string_value};

pub(super) fn native_datetime_format_constructor(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let _locale = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_else(|| "en-US".to_string());

    let mut options = std::collections::HashMap::new();
    if let Some(Value::Object(opts_idx)) = args.get(1) {
        if let HeapValue::Object(obj) = &interp.heap[*opts_idx] {
            for (k, v) in &obj.properties {
                options.insert(k.clone(), to_string_value(interp, v));
            }
        }
    }

    let date_style = options
        .get("dateStyle")
        .cloned()
        .unwrap_or_else(|| "medium".to_string());
    let time_style = options
        .get("timeStyle")
        .cloned()
        .unwrap_or_else(|| "short".to_string());
    let weekday = options.get("weekday").cloned();
    let year = options.get("year").cloned();
    let month = options.get("month").cloned();
    let day = options.get("day").cloned();
    let hour = options.get("hour").cloned();
    let minute = options.get("minute").cloned();
    let second = options.get("second").cloned();

    let mut formatter_props = std::collections::HashMap::new();
    formatter_props.insert("type".into(), Value::String("datetime".into()));
    formatter_props.insert("dateStyle".into(), Value::String(date_style));
    formatter_props.insert("timeStyle".into(), Value::String(time_style));
    if let Some(w) = weekday {
        formatter_props.insert("weekday".into(), Value::String(w));
    }
    if let Some(y) = year {
        formatter_props.insert("year".into(), Value::String(y));
    }
    if let Some(m) = month {
        formatter_props.insert("month".into(), Value::String(m));
    }
    if let Some(d) = day {
        formatter_props.insert("day".into(), Value::String(d));
    }
    if let Some(h) = hour {
        formatter_props.insert("hour".into(), Value::String(h));
    }
    if let Some(mi) = minute {
        formatter_props.insert("minute".into(), Value::String(mi));
    }
    if let Some(s) = second {
        formatter_props.insert("second".into(), Value::String(s));
    }

    formatter_props.insert("format".into(), Value::NativeFunction(262));
    formatter_props.insert("formatToParts".into(), Value::NativeFunction(263));

    let formatter_idx = interp.gc.allocate(
        &mut interp.heap,
        HeapValue::Object(JsObject {
            properties: formatter_props,
            prototype: None,
            extensible: true,
        }),
    );
    Ok(Value::Object(formatter_idx))
}

pub(super) fn native_datetime_format_format(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    // Get the formatter's options from `this`
    let (date_style, time_style, has_explicit_parts) = if let Value::Object(obj_idx) = _this {
        if let HeapValue::Object(obj) = &interp.heap[*obj_idx] {
            let ds = obj
                .properties
                .get("dateStyle")
                .and_then(|v| {
                    if let Value::String(s) = v {
                        Some(s.clone())
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "medium".to_string());
            let ts = obj
                .properties
                .get("timeStyle")
                .and_then(|v| {
                    if let Value::String(s) = v {
                        Some(s.clone())
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "short".to_string());
            let has_parts = obj.properties.contains_key("weekday")
                || obj.properties.contains_key("year")
                || obj.properties.contains_key("month")
                || obj.properties.contains_key("day")
                || obj.properties.contains_key("hour");
            (ds, ts, has_parts)
        } else {
            ("medium".to_string(), "short".to_string(), false)
        }
    } else {
        ("medium".to_string(), "short".to_string(), false)
    };

    // Get date from argument or current time
    let millis = if let Some(val) = args.first() {
        match val {
            Value::Float(f) => *f as i64,
            Value::Integer(n) => *n,
            Value::Object(obj_idx) => {
                // Try to get .getTime() from a Date object
                if let HeapValue::Object(obj) = &interp.heap[*obj_idx] {
                    if let Some(Value::Float(f)) = obj.properties.get("time") {
                        *f as i64
                    } else {
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis() as i64
                    }
                } else {
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as i64
                }
            }
            _ => std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64,
        }
    } else {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64
    };

    let formatted = format_datetime(
        millis,
        &date_style,
        &time_style,
        has_explicit_parts,
        interp,
        _this,
    );
    Ok(Value::String(formatted))
}

fn format_datetime(
    millis: i64,
    date_style: &str,
    time_style: &str,
    has_explicit_parts: bool,
    interp: &Interpreter,
    this: &Value,
) -> String {
    let seconds = millis / 1000;
    let _ms = millis % 1000;

    // Simple UTC formatting
    let total_secs = seconds;
    let secs_in_day = ((total_secs % 86400) + 86400) % 86400;
    let hours = secs_in_day / 3600;
    let minutes = (secs_in_day % 3600) / 60;
    let secs = secs_in_day % 60;

    // Days since epoch for date calculation
    let days = total_secs / 86400;
    let mut year = 1970;
    let mut remaining_days = days;
    loop {
        let days_in_year = if is_leap(year) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }

    let month_days = if is_leap(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 0;
    for (i, &md) in month_days.iter().enumerate() {
        if remaining_days < md {
            month = i;
            break;
        }
        remaining_days -= md;
    }
    let day = remaining_days + 1;

    let month_name = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ][month];

    let month_full = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ][month];

    let weekday_names = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    let day_of_week = ((days % 7) + 7) % 7;
    let weekday = weekday_names[day_of_week as usize];

    // If explicit parts are set, use them
    if has_explicit_parts {
        if let Value::Object(obj_idx) = this {
            if let HeapValue::Object(obj) = &interp.heap[*obj_idx] {
                let mut parts = Vec::new();

                if let Some(Value::String(w)) = obj.properties.get("weekday") {
                    match w.as_str() {
                        "short" => parts.push(weekday.to_string()),
                        "long" | "narrow" => {
                            let full_names = [
                                "Sunday",
                                "Monday",
                                "Tuesday",
                                "Wednesday",
                                "Thursday",
                                "Friday",
                                "Saturday",
                            ];
                            parts.push(full_names[day_of_week as usize].to_string());
                        }
                        _ => {}
                    }
                }
                if let Some(Value::String(y)) = obj.properties.get("year") {
                    match y.as_str() {
                        "numeric" | "2-digit" => parts.push(format!("{:04}", year)),
                        _ => {}
                    }
                }
                if let Some(Value::String(m)) = obj.properties.get("month") {
                    match m.as_str() {
                        "numeric" => parts.push(format!("{}", month + 1)),
                        "2-digit" => parts.push(format!("{:02}", month + 1)),
                        "short" => parts.push(month_name.to_string()),
                        "long" => parts.push(month_full.to_string()),
                        _ => {}
                    }
                }
                if let Some(Value::String(d)) = obj.properties.get("day") {
                    match d.as_str() {
                        "numeric" => parts.push(format!("{}", day)),
                        "2-digit" => parts.push(format!("{:02}", day)),
                        _ => {}
                    }
                }
                if let Some(Value::String(h)) = obj.properties.get("hour") {
                    match h.as_str() {
                        "numeric" | "2-digit" => {
                            let ampm = if hours >= 12 { "PM" } else { "AM" };
                            let h12 = if hours == 0 {
                                12
                            } else if hours > 12 {
                                hours - 12
                            } else {
                                hours
                            };
                            parts.push(format!("{}:{:02}", h12, minutes));
                            parts.push(ampm.to_string());
                        }
                        _ => {}
                    }
                }
                return parts.join(" ");
            }
        }
    }

    // Default formatting based on dateStyle/timeStyle
    match date_style {
        "full" => {
            let full_month = month_full;
            let result = format!(
                "{}, {} {} {}, {:02}:{:02}:{:02} UTC",
                weekday, full_month, day, year, hours, minutes, secs
            );
            result
        }
        "long" => {
            format!(
                "{} {} {}, {:02}:{:02}:{:02} UTC",
                month_full, day, year, hours, minutes, secs
            )
        }
        "medium" | "short" => {
            format!(
                "{} {}, {} {:02}:{:02} UTC",
                month_name, day, year, hours, minutes
            )
        }
        "none" | "" => match time_style {
            "none" | "" => String::new(),
            _ => format!("{:02}:{:02}", hours, minutes),
        },
        _ => {
            format!(
                "{}/{:02}/{:02} {:02}:{:02}:{:02}",
                year,
                month + 1,
                day,
                hours,
                minutes,
                secs
            )
        }
    }
}

fn is_leap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

pub(super) fn native_datetime_format_format_to_parts(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    // Simplified: return an array of {type, value} objects
    let formatted = native_datetime_format_format(interp, _this, args)?;
    if let Value::String(s) = formatted {
        let parts: Vec<Value> = s
            .chars()
            .map(|c| {
                let mut props = std::collections::HashMap::new();
                props.insert("type".into(), Value::String("literal".into()));
                props.insert("value".into(), Value::String(c.to_string()));
                let idx = interp.heap.len();
                interp.heap.push(HeapValue::Object(JsObject {
                    properties: props,
                    prototype: None,
                    extensible: true,
                }));
                Value::Object(idx)
            })
            .collect();
        let arr_idx = interp.heap.len();
        interp
            .heap
            .push(HeapValue::Array(crate::vm::interpreter::JsArray {
                elements: parts,
            }));
        Ok(Value::Array(arr_idx))
    } else {
        let arr_idx = interp.heap.len();
        interp
            .heap
            .push(HeapValue::Array(crate::vm::interpreter::JsArray {
                elements: Vec::new(),
            }));
        Ok(Value::Array(arr_idx))
    }
}

pub(super) fn native_number_format_constructor(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let _locale = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_else(|| "en-US".to_string());

    let mut options = std::collections::HashMap::new();
    if let Some(Value::Object(opts_idx)) = args.get(1) {
        if let HeapValue::Object(obj) = &interp.heap[*opts_idx] {
            for (k, v) in &obj.properties {
                options.insert(k.clone(), to_string_value(interp, v));
            }
        }
    }

    let style = options
        .get("style")
        .cloned()
        .unwrap_or_else(|| "decimal".to_string());
    let currency = options.get("currency").cloned();
    let minimum_fraction_digits = options
        .get("minimumFractionDigits")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(0);
    let maximum_fraction_digits = options
        .get("maximumFractionDigits")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(3);

    let mut formatter_props = std::collections::HashMap::new();
    formatter_props.insert("type".into(), Value::String("number".into()));
    formatter_props.insert("style".into(), Value::String(style));
    if let Some(c) = currency {
        formatter_props.insert("currency".into(), Value::String(c));
    }
    formatter_props.insert(
        "minimumFractionDigits".into(),
        Value::Integer(minimum_fraction_digits as i64),
    );
    formatter_props.insert(
        "maximumFractionDigits".into(),
        Value::Integer(maximum_fraction_digits as i64),
    );

    formatter_props.insert("format".into(), Value::NativeFunction(264));

    let formatter_idx = interp.gc.allocate(
        &mut interp.heap,
        HeapValue::Object(JsObject {
            properties: formatter_props,
            prototype: None,
            extensible: true,
        }),
    );
    Ok(Value::Object(formatter_idx))
}

pub(super) fn native_number_format_format(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let number = args.first().map(to_f64).unwrap_or(0.0);

    let (style, currency, min_frac, max_frac) = if let Value::Object(obj_idx) = _this {
        if let HeapValue::Object(obj) = &interp.heap[*obj_idx] {
            let s = obj
                .properties
                .get("style")
                .and_then(|v| {
                    if let Value::String(s) = v {
                        Some(s.clone())
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "decimal".to_string());
            let c = obj.properties.get("currency").and_then(|v| {
                if let Value::String(s) = v {
                    Some(s.clone())
                } else {
                    None
                }
            });
            let min_f = obj
                .properties
                .get("minimumFractionDigits")
                .and_then(|v| {
                    if let Value::Integer(n) = v {
                        Some(*n as usize)
                    } else {
                        None
                    }
                })
                .unwrap_or(0);
            let max_f = obj
                .properties
                .get("maximumFractionDigits")
                .and_then(|v| {
                    if let Value::Integer(n) = v {
                        Some(*n as usize)
                    } else {
                        None
                    }
                })
                .unwrap_or(3);
            (s, c, min_f, max_f)
        } else {
            ("decimal".to_string(), None, 0, 3)
        }
    } else {
        ("decimal".to_string(), None, 0, 3)
    };

    let formatted = match style.as_str() {
        "currency" => {
            let sym = currency.unwrap_or_else(|| "USD".to_string());
            let symbol = match sym.as_str() {
                "USD" => "$",
                "EUR" => "\u{20AC}",
                "GBP" => "\u{00A3}",
                "JPY" => "\u{00A5}",
                _ => &sym,
            };
            format!("{}{}", symbol, format_number(number, min_frac, max_frac))
        }
        "percent" => {
            let pct = number * 100.0;
            format!("{}%", format_number(pct, min_frac, max_frac))
        }
        _ => format_number(number, min_frac, max_frac),
    };

    Ok(Value::String(formatted))
}

fn format_number(n: f64, min_frac: usize, max_frac: usize) -> String {
    if n.is_nan() {
        return "NaN".to_string();
    }
    if n.is_infinite() {
        return if n > 0.0 { "Infinity" } else { "-Infinity" }.to_string();
    }

    // Determine precision
    let _int_part = n.trunc();

    // Format with the right number of decimal places
    let formatted = if max_frac == 0 && min_frac == 0 {
        // Integer format
        format!("{:.0}", n.round())
    } else {
        // Use the max fraction digits for display, then trim trailing zeros
        let precision = max_frac;
        let s = format!("{:.prec$}", n, prec = precision);
        // Trim trailing zeros after decimal point
        if s.contains('.') {
            let trimmed = s.trim_end_matches('0').trim_end_matches('.');
            trimmed.to_string()
        } else {
            s
        }
    };

    // Add thousand separators
    add_thousand_separators(&formatted)
}

fn add_thousand_separators(s: &str) -> String {
    if let Some(rest) = s.strip_prefix('-') {
        return format!("-{}", add_thousand_separators(rest));
    }

    let parts: Vec<&str> = s.split('.').collect();
    let int_part = parts[0];
    let frac_part = if parts.len() > 1 {
        Some(parts[1])
    } else {
        None
    };

    let mut result = String::new();
    let chars: Vec<char> = int_part.chars().collect();
    for (i, c) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i).is_multiple_of(3) {
            result.push(',');
        }
        result.push(*c);
    }

    if let Some(frac) = frac_part {
        result.push('.');
        result.push_str(frac);
    }

    result
}
