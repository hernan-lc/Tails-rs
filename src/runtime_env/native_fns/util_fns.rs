use crate::errors::Result;
use crate::objects::Value;
use crate::vm::interpreter::{HeapValue, Interpreter, JsObject};

use super::helpers::{to_f64, to_string_value};

pub(super) fn native_util_format(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let fmt = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    let mut result = String::new();
    let mut arg_idx = 1;
    let mut chars = fmt.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '%' {
            match chars.next() {
                Some('s') => {
                    let arg = args.get(arg_idx).cloned().unwrap_or(Value::Undefined);
                    result.push_str(&to_string_value(interp, &arg));
                    arg_idx += 1;
                }
                Some('d') | Some('i') => {
                    let arg = args.get(arg_idx).cloned().unwrap_or(Value::Undefined);
                    let n = to_f64(&arg);
                    if n.is_nan() {
                        result.push_str("NaN");
                    } else {
                        result.push_str(&(n as i64).to_string());
                    }
                    arg_idx += 1;
                }
                Some('f') => {
                    let arg = args.get(arg_idx).cloned().unwrap_or(Value::Undefined);
                    let n = to_f64(&arg);
                    result.push_str(&format!("{:.6}", n));
                    arg_idx += 1;
                }
                Some('o') | Some('O') => {
                    let arg = args.get(arg_idx).cloned().unwrap_or(Value::Undefined);
                    result.push_str(&inspect_value(interp, &arg, 2, 0));
                    arg_idx += 1;
                }
                Some('j') => {
                    let arg = args.get(arg_idx).cloned().unwrap_or(Value::Undefined);
                    result.push_str(
                        &crate::runtime_env::native_fns::json_fns::native_json_stringify(
                            interp,
                            &Value::Undefined,
                            &[arg],
                        )
                        .map(|v| to_string_value(interp, &v))
                        .unwrap_or_default(),
                    );
                    arg_idx += 1;
                }
                Some('%') => result.push('%'),
                Some(c) => {
                    result.push('%');
                    result.push(c);
                }
                None => result.push('%'),
            }
        } else {
            result.push(ch);
        }
    }
    Ok(Value::String(result))
}

pub(super) fn native_util_inspect(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let value = args.first().cloned().unwrap_or(Value::Undefined);
    let depth = args
        .get(1)
        .and_then(|v| match v {
            Value::Object(idx) => {
                if let HeapValue::Object(obj) = &interp.heap[*idx] {
                    obj.properties
                        .get("depth")
                        .map(|d| to_f64(d) as usize)
                } else {
                    None
                }
            }
            _ => None,
        })
        .unwrap_or(2);
    Ok(Value::String(inspect_value(interp, &value, depth, 0)))
}

pub(super) fn native_util_promisify(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let original = args.first().cloned().unwrap_or(Value::Undefined);
    let name = format!("promisified {}", to_string_value(interp, &original));
    let wrapper_idx = interp.gc.allocate(
        &mut interp.heap,
        HeapValue::Object(JsObject {
            properties: crate::props! {
                "_original" => original,
                "name" => Value::String(name),
            },
            prototype: None,
            extensible: true,
        }),
    );
    Ok(Value::Object(wrapper_idx))
}

pub(super) fn native_util_callbackify(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let original = args.first().cloned().unwrap_or(Value::Undefined);
    let name = format!("callbackified {}", to_string_value(interp, &original));
    let wrapper_idx = interp.gc.allocate(
        &mut interp.heap,
        HeapValue::Object(JsObject {
            properties: crate::props! {
                "_original" => original,
                "name" => Value::String(name),
            },
            prototype: None,
            extensible: true,
        }),
    );
    Ok(Value::Object(wrapper_idx))
}

fn inspect_value(interp: &Interpreter, value: &Value, depth: usize, indent: usize) -> String {
    if depth == 0 {
        return "[Object]".to_string();
    }
    match value {
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
        Value::String(s) => format!("'{}'", s),
        Value::BigInt(n) => format!("{}n", n),
        Value::Symbol(id) => format!("Symbol({})", id),
        Value::Array(idx) => {
            if let HeapValue::Array(arr) = &interp.heap[*idx] {
                let prefix = " ".repeat(indent);
                let inner = " ".repeat(indent + 2);
                if arr.elements.is_empty() {
                    return "[]".to_string();
                }
                let items: Vec<String> = arr
                    .elements
                    .iter()
                    .map(|v| format!("{}{}", inner, inspect_value(interp, v, depth - 1, indent + 2)))
                    .collect();
                format!("[\n{}\n{}]", items.join(",\n"), prefix)
            } else {
                "[]".to_string()
            }
        }
        Value::Object(idx) => {
            if let HeapValue::Object(obj) = &interp.heap[*idx] {
                let prefix = " ".repeat(indent);
                let inner = " ".repeat(indent + 2);
                if obj.properties.is_empty() {
                    return "{}".to_string();
                }
                let items: Vec<String> = obj
                    .properties
                    .iter()
                    .map(|(k, v)| {
                        format!(
                            "{}{}: {}",
                            inner,
                            k,
                            inspect_value(interp, v, depth - 1, indent + 2)
                        )
                    })
                    .collect();
                format!("{{\n{}\n{}}}", items.join(",\n"), prefix)
            } else {
                "{}".to_string()
            }
        }
        Value::Function(idx) => {
            if let HeapValue::Function(f) = &interp.heap[*idx] {
                f.name
                    .clone()
                    .unwrap_or_else(|| "[Function]".to_string())
            } else {
                "[Function]".to_string()
            }
        }
        Value::NativeFunction(idx) => format!("[Function: native({})]", idx),
        Value::Promise(idx) => format!("Promise {{ <{}> }}", idx),
        _ => "[Object]".to_string(),
    }
}
