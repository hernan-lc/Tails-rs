use crate::errors::Result;
use crate::objects::Value;
use crate::vm::interpreter::Interpreter;

use super::helpers::to_display_string;
use colored::*;
use std::cell::RefCell;
use std::collections::HashMap;

const MAX_DEPTH: usize = 4;
const INDENT: &str = "  ";

thread_local! {
    static USE_COLORS: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
    static USE_TIMESTAMPS: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static GROUP_DEPTH: std::cell::Cell<i32> = const { std::cell::Cell::new(0) };
    static TIMERS: RefCell<HashMap<String, u128>> = RefCell::new(HashMap::new());
}

pub fn set_colors(enabled: bool) {
    USE_COLORS.with(|c| c.set(enabled));
}

pub fn set_timestamps(enabled: bool) {
    USE_TIMESTAMPS.with(|c| c.set(enabled));
}

pub fn get_use_colors() -> bool {
    USE_COLORS.with(|c| c.get())
}

fn get_indent() -> String {
    GROUP_DEPTH.with(|d| "  ".repeat(d.get() as usize))
}

fn get_timestamp() -> String {
    USE_TIMESTAMPS.with(|ts| {
        if ts.get() {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default();
            let secs = now.as_secs();
            let hours = (secs / 3600) % 24;
            let minutes = (secs / 60) % 60;
            let seconds = secs % 60;
            format!("[{:02}:{:02}:{:02}] ", hours, minutes, seconds)
        } else {
            String::new()
        }
    })
}

fn collect_object_properties<'a>(
    interp: &'a Interpreter,
    obj_idx: usize,
    visited: &mut std::collections::HashSet<usize>,
) -> Vec<(String, &'a Value)> {
    if !visited.insert(obj_idx) {
        return Vec::new();
    }

    let mut all_props: Vec<(String, &'a Value)> = Vec::new();

    if let crate::vm::interpreter::HeapValue::Object(obj) = &interp.heap[obj_idx] {
        for (k, v) in &obj.properties {
            if k == "constructor" {
                continue;
            }
            if k.starts_with("__getter_") {
                let prop_name = &k["__getter_".len()..];
                all_props.push((prop_name.to_string(), v));
                continue;
            }
            if k.starts_with("__setter_") || k.starts_with("__method_") {
                continue;
            }
            all_props.push((k.clone(), v));
        }
        if let Some(proto_idx) = obj.prototype {
            let proto_props = collect_object_properties(interp, proto_idx, visited);
            for (k, v) in proto_props {
                if !all_props.iter().any(|(existing_k, _)| existing_k == &k) {
                    all_props.push((k, v));
                }
            }
        }
    }

    all_props
}

fn pretty_format(interp: &Interpreter, v: &Value, depth: usize, use_colors: bool) -> String {
    if depth >= MAX_DEPTH {
        return match v {
            Value::Object(_) => "[Object]".to_string(),
            Value::Array(_) => "[Array]".to_string(),
            _ => to_display_string(interp, v),
        };
    }

    match v {
        Value::Object(obj_idx) => {
            let mut visited = std::collections::HashSet::new();
            let all_props = collect_object_properties(interp, *obj_idx, &mut visited);

            if all_props.is_empty() {
                return "{}".to_string();
            }

            let mut props = all_props;
            props.sort_by(|a, b| a.0.cmp(&b.0));

            let pad = INDENT.repeat(depth + 1);
            let closing_pad = INDENT.repeat(depth);

            let mut lines: Vec<String> = Vec::new();
            for (key, val) in &props {
                let getter_key = format!("__getter_{}", key);
                let setter_key = format!("__setter_{}", key);
                let is_getter = if let crate::vm::interpreter::HeapValue::Object(obj) =
                    &interp.heap[*obj_idx]
                {
                    obj.properties.contains_key(&getter_key)
                        && !obj.properties.contains_key(&setter_key)
                } else {
                    false
                };
                let is_setter = if let crate::vm::interpreter::HeapValue::Object(obj) =
                    &interp.heap[*obj_idx]
                {
                    !obj.properties.contains_key(&getter_key)
                        && obj.properties.contains_key(&setter_key)
                } else {
                    false
                };
                let val_str = if is_getter || is_setter {
                    if use_colors {
                        "[Getter]".dimmed().to_string()
                    } else {
                        "[Getter]".to_string()
                    }
                } else {
                    pretty_format(interp, val, depth + 1, use_colors)
                };
                if use_colors {
                    lines.push(format!("{}{}: {}", pad, key.bold(), val_str));
                } else {
                    lines.push(format!("{}{}: {}", pad, key, val_str));
                }
            }

            format!("{{\n{}\n{}}}", lines.join(",\n"), closing_pad)
        }
        Value::Array(arr_idx) => {
            if let crate::vm::interpreter::HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                if arr.elements.is_empty() {
                    return "[]".to_string();
                }

                let pad = INDENT.repeat(depth + 1);
                let closing_pad = INDENT.repeat(depth);

                let mut lines: Vec<String> = Vec::new();
                for elem in &arr.elements {
                    let val_str = pretty_format(interp, elem, depth + 1, use_colors);
                    lines.push(format!("{}{}", pad, val_str));
                }

                format!("[\n{}\n{}]", lines.join(",\n"), closing_pad)
            } else {
                "[Array]".to_string()
            }
        }
        Value::Function(idx) => {
            if let crate::vm::interpreter::HeapValue::Function(f) = &interp.heap[*idx] {
                let name = f.name.as_deref().unwrap_or("anonymous");
                if f.prototype.is_some() && f.super_class.is_some() {
                    if use_colors {
                        format!("[class {}]", name.cyan())
                    } else {
                        format!("[class {}]", name)
                    }
                } else if use_colors {
                    format!("[Function: {}]", name.cyan())
                } else {
                    format!("[Function: {}]", name)
                }
            } else {
                "[Function]".to_string()
            }
        }
        Value::NativeFunction(_) => {
            if use_colors {
                "[NativeFunction]".cyan().to_string()
            } else {
                "[NativeFunction]".to_string()
            }
        }
        Value::String(s) => {
            if use_colors {
                format!("\"{}\"", s.green())
            } else {
                format!("\"{}\"", s)
            }
        }
        Value::Integer(n) => {
            if use_colors {
                n.to_string().magenta().to_string()
            } else {
                n.to_string()
            }
        }
        Value::Float(n) => {
            let val = if *n == (*n as i64) as f64 {
                (*n as i64).to_string()
            } else {
                n.to_string()
            };
            if use_colors {
                val.magenta().to_string()
            } else {
                val
            }
        }
        Value::Boolean(b) => {
            let val = b.to_string();
            if use_colors {
                val.yellow().to_string()
            } else {
                val
            }
        }
        Value::Null => {
            if use_colors {
                "null".red().bold().to_string()
            } else {
                "null".to_string()
            }
        }
        Value::Undefined => {
            if use_colors {
                "undefined".dimmed().to_string()
            } else {
                "undefined".to_string()
            }
        }
        Value::Map(idx) => {
            if let crate::vm::interpreter::HeapValue::Map(map) = &interp.heap[*idx] {
                if map.entries.is_empty() {
                    return "Map(0) {}".to_string();
                }
                let pad = INDENT.repeat(depth + 1);
                let closing_pad = INDENT.repeat(depth);
                let mut lines: Vec<String> = Vec::new();
                for (k, val) in &map.entries {
                    let k_str = pretty_format(interp, k, depth + 1, use_colors);
                    let v_str = pretty_format(interp, val, depth + 1, use_colors);
                    lines.push(format!("{}{} => {}", pad, k_str, v_str));
                }
                format!(
                    "Map({}) {{\n{}\n{}}}",
                    map.entries.len(),
                    lines.join(",\n"),
                    closing_pad
                )
            } else {
                "Map".to_string()
            }
        }
        Value::Set(idx) => {
            if let crate::vm::interpreter::HeapValue::Set(set) = &interp.heap[*idx] {
                if set.values.is_empty() {
                    return "Set(0) {}".to_string();
                }
                let pad = INDENT.repeat(depth + 1);
                let closing_pad = INDENT.repeat(depth);
                let mut lines: Vec<String> = Vec::new();
                for val in &set.values {
                    let val_str = pretty_format(interp, val, depth + 1, use_colors);
                    lines.push(format!("{}{}", pad, val_str));
                }
                format!(
                    "Set({}) {{\n{}\n{}}}",
                    set.values.len(),
                    lines.join(",\n"),
                    closing_pad
                )
            } else {
                "Set".to_string()
            }
        }
        Value::Date(idx) => {
            if let crate::vm::interpreter::HeapValue::Date(d) = &interp.heap[*idx] {
                if use_colors {
                    format!("Date({})", d.to_utc_string().blue())
                } else {
                    format!("Date({})", d.to_utc_string())
                }
            } else {
                "Date".to_string()
            }
        }
        Value::RegExp(idx) => {
            if let crate::vm::interpreter::HeapValue::RegExp(r) = &interp.heap[*idx] {
                if use_colors {
                    format!("/{}/{}", r.source.red(), r.flags)
                } else {
                    format!("/{}/{}", r.source, r.flags)
                }
            } else {
                "RegExp".to_string()
            }
        }
        _ => to_display_string(interp, v),
    }
}

fn colorize_value(interp: &Interpreter, v: &Value) -> String {
    let use_colors = get_use_colors();
    pretty_format(interp, v, 0, use_colors)
}

pub(super) fn native_console_log(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let indent = get_indent();
    let timestamp = get_timestamp();
    let parts: Vec<String> = args.iter().map(|a| colorize_value(interp, a)).collect();
    println!("{}{}{}", timestamp, indent, parts.join(" "));
    Ok(Value::Undefined)
}

pub(super) fn native_console_warn(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let indent = get_indent();
    let timestamp = get_timestamp();
    let use_colors = get_use_colors();
    let parts: Vec<String> = args.iter().map(|a| colorize_value(interp, a)).collect();
    let msg = parts.join(" ");
    if use_colors {
        eprintln!("{}{}{}", timestamp, indent, msg.yellow());
    } else {
        eprintln!("{}{}{}", timestamp, indent, msg);
    }
    Ok(Value::Undefined)
}

pub(super) fn native_console_error(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let indent = get_indent();
    let timestamp = get_timestamp();
    let use_colors = get_use_colors();
    let parts: Vec<String> = args.iter().map(|a| colorize_value(interp, a)).collect();
    let msg = parts.join(" ");
    if use_colors {
        eprintln!("{}{}{}", timestamp, indent, msg.red());
    } else {
        eprintln!("{}{}{}", timestamp, indent, msg);
    }
    Ok(Value::Undefined)
}

pub(super) fn native_console_info(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let indent = get_indent();
    let timestamp = get_timestamp();
    let use_colors = get_use_colors();
    let parts: Vec<String> = args.iter().map(|a| colorize_value(interp, a)).collect();
    let msg = parts.join(" ");
    if use_colors {
        println!("{}{}{}", timestamp, indent, msg.blue());
    } else {
        println!("{}{}{}", timestamp, indent, msg);
    }
    Ok(Value::Undefined)
}

pub(super) fn native_console_table(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    if args.is_empty() {
        println!("[]");
        return Ok(Value::Undefined);
    }

    let indent = get_indent();
    let timestamp = get_timestamp();
    let use_colors = get_use_colors();

    match &args[0] {
        Value::Array(arr_idx) => {
            if let crate::vm::interpreter::HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                if arr.elements.is_empty() {
                    println!("{}{}(empty table)", timestamp, indent);
                    return Ok(Value::Undefined);
                }

                let mut all_keys: Vec<String> = Vec::new();
                for elem in &arr.elements {
                    if let Value::Object(obj_idx) = elem {
                        if let crate::vm::interpreter::HeapValue::Object(obj) =
                            &interp.heap[*obj_idx]
                        {
                            for key in obj.properties.keys() {
                                if key.starts_with("__getter_")
                                    || key.starts_with("__setter_")
                                    || key.starts_with("__method_")
                                {
                                    continue;
                                }
                                if !all_keys.contains(key) {
                                    all_keys.push(key.clone());
                                }
                            }
                        }
                    }
                }
                all_keys.sort();

                if all_keys.is_empty() {
                    let parts: Vec<String> = arr
                        .elements
                        .iter()
                        .enumerate()
                        .map(|(i, e)| format!("{}: {}", i, colorize_value(interp, e)))
                        .collect();
                    println!("{}{}[{}]", timestamp, indent, parts.join(", "));
                    return Ok(Value::Undefined);
                }

                let index_width = format!("{}", arr.elements.len() - 1).len().max(5);
                let mut col_widths: Vec<usize> = all_keys.iter().map(|k| k.len()).collect();

                for elem in &arr.elements {
                    if let Value::Object(obj_idx) = elem {
                        if let crate::vm::interpreter::HeapValue::Object(obj) =
                            &interp.heap[*obj_idx]
                        {
                            for (i, key) in all_keys.iter().enumerate() {
                                if let Some(val) = obj.properties.get(key) {
                                    let val_str = to_display_string(interp, val);
                                    col_widths[i] = col_widths[i].max(val_str.len());
                                }
                            }
                        }
                    }
                }

                let header_idx = "(index)".to_string();
                let mut header = format!("{:width$}", header_idx, width = index_width);
                for key in &all_keys {
                    if use_colors {
                        header.push_str(&format!(" | {}", key.bold()));
                    } else {
                        header.push_str(&format!(" | {}", key));
                    }
                }
                let separator = "-".repeat(header.len());
                println!("{}{}{}", timestamp, indent, header);
                println!("{}{}{}", timestamp, indent, separator);

                for (row_idx, elem) in arr.elements.iter().enumerate() {
                    let mut row = format!("{:width$}", row_idx, width = index_width);
                    if let Value::Object(obj_idx) = elem {
                        if let crate::vm::interpreter::HeapValue::Object(obj) =
                            &interp.heap[*obj_idx]
                        {
                            for (i, key) in all_keys.iter().enumerate() {
                                let val_str = if let Some(val) = obj.properties.get(key) {
                                    colorize_value(interp, val)
                                } else {
                                    "undefined".to_string()
                                };
                                row.push_str(&format!(
                                    " | {:width$}",
                                    val_str,
                                    width = col_widths[i]
                                ));
                            }
                        }
                    } else {
                        let val_str = colorize_value(interp, elem);
                        row.push_str(&format!(" | {}", val_str));
                    }
                    println!("{}{}{}", timestamp, indent, row);
                }
            }
        }
        Value::Object(obj_idx) => {
            if let crate::vm::interpreter::HeapValue::Object(obj) = &interp.heap[*obj_idx] {
                let mut props: Vec<(&String, &Value)> = obj
                    .properties
                    .iter()
                    .filter(|(k, _)| {
                        !k.starts_with("__getter_")
                            && !k.starts_with("__setter_")
                            && !k.starts_with("__method_")
                    })
                    .collect();
                props.sort_by(|a, b| a.0.cmp(b.0));

                if props.is_empty() {
                    println!("{}{{}}", timestamp);
                    return Ok(Value::Undefined);
                }

                let key_width = props.iter().map(|(k, _)| k.len()).max().unwrap_or(5);
                let val_width = props
                    .iter()
                    .map(|(_, v)| to_display_string(interp, v).len())
                    .max()
                    .unwrap_or(5);

                let header = if use_colors {
                    format!(
                        "{:width_key$} | {:width_val$}",
                        "Key".bold(),
                        "Value".bold(),
                        width_key = key_width,
                        width_val = val_width
                    )
                } else {
                    format!(
                        "{:width_key$} | {:width_val$}",
                        "Key",
                        "Value",
                        width_key = key_width,
                        width_val = val_width
                    )
                };
                let separator = "-".repeat(header.len());
                println!("{}{}{}", timestamp, indent, header);
                println!("{}{}{}", timestamp, indent, separator);

                for (key, val) in &props {
                    let val_str = colorize_value(interp, val);
                    println!(
                        "{}{}{:width_key$} | {:width_val$}",
                        timestamp,
                        indent,
                        key,
                        val_str,
                        width_key = key_width,
                        width_val = val_width
                    );
                }
            }
        }
        _ => {
            let parts: Vec<String> = args.iter().map(|a| colorize_value(interp, a)).collect();
            println!("{}{}{}", timestamp, indent, parts.join(" "));
        }
    }

    Ok(Value::Undefined)
}

pub(super) fn native_console_dir(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    if args.is_empty() {
        return Ok(Value::Undefined);
    }

    let indent = get_indent();
    let timestamp = get_timestamp();
    let use_colors = get_use_colors();
    let formatted = pretty_format(interp, &args[0], 0, use_colors);
    println!("{}{}{}", timestamp, indent, formatted);

    Ok(Value::Undefined)
}

pub(super) fn native_console_group(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let indent = get_indent();
    let timestamp = get_timestamp();
    let parts: Vec<String> = args.iter().map(|a| colorize_value(interp, a)).collect();

    if !parts.is_empty() {
        if get_use_colors() {
            println!("{}{}{}", timestamp, indent, parts.join(" ").bold());
        } else {
            println!("{}{}{}", timestamp, indent, parts.join(" "));
        }
    }

    GROUP_DEPTH.with(|d| d.set(d.get() + 1));

    Ok(Value::Undefined)
}

pub(super) fn native_console_group_end(
    _interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    GROUP_DEPTH.with(|d| {
        let val = d.get();
        if val > 0 {
            d.set(val - 1);
        }
    });
    Ok(Value::Undefined)
}

pub(super) fn native_console_group_collapsed(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let indent = get_indent();
    let timestamp = get_timestamp();
    let parts: Vec<String> = args.iter().map(|a| colorize_value(interp, a)).collect();

    if !parts.is_empty() {
        if get_use_colors() {
            println!("{}{}▶ {}", timestamp, indent, parts.join(" ").bold());
        } else {
            println!("{}{}▶ {}", timestamp, indent, parts.join(" "));
        }
    }

    GROUP_DEPTH.with(|d| d.set(d.get() + 1));

    Ok(Value::Undefined)
}

pub(super) fn native_console_time(
    _interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let label = if args.is_empty() {
        "default".to_string()
    } else {
        to_display_string(_interp, &args[0])
    };

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    TIMERS.with(|t| {
        t.borrow_mut().insert(label, now);
    });

    Ok(Value::Undefined)
}

pub(super) fn native_console_time_end(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let label = if args.is_empty() {
        "default".to_string()
    } else {
        to_display_string(interp, &args[0])
    };

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    let elapsed = TIMERS.with(|t| t.borrow().get(&label).map(|start| now - start).unwrap_or(0));

    let indent = get_indent();
    let timestamp = get_timestamp();
    let use_colors = get_use_colors();

    if use_colors {
        println!(
            "{}{}{}: {}ms",
            timestamp,
            indent,
            label.bold(),
            elapsed.to_string().cyan()
        );
    } else {
        println!("{}{}{}: {}ms", timestamp, indent, label, elapsed);
    }

    Ok(Value::Undefined)
}

pub(super) fn native_console_assert(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    if args.is_empty() {
        return Ok(Value::Undefined);
    }

    let condition = match &args[0] {
        Value::Boolean(b) => *b,
        Value::Integer(n) => *n != 0,
        Value::Float(n) => !n.is_nan() && *n != 0.0,
        Value::Null | Value::Undefined => false,
        Value::String(s) => !s.is_empty(),
        _ => true,
    };

    if !condition {
        let indent = get_indent();
        let timestamp = get_timestamp();
        let use_colors = get_use_colors();
        let parts: Vec<String> = if args.len() > 1 {
            args[1..]
                .iter()
                .map(|a| colorize_value(interp, a))
                .collect()
        } else {
            vec!["Assertion failed".to_string()]
        };
        let msg = parts.join(" ");
        if use_colors {
            eprintln!("{}{}{}", timestamp, indent, msg.red());
        } else {
            eprintln!("{}{}{}", timestamp, indent, msg);
        }
    }

    Ok(Value::Undefined)
}

pub(super) fn native_console_clear(
    _interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    print!("\x1B[2J\x1B[1;1H");
    Ok(Value::Undefined)
}
