use crate::errors::Result;
use crate::objects::Value;
use crate::vm::interpreter::{HeapValue, Interpreter, JsObject};
use crate::well_known as wk;

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
                        result.push_str(wk::NAN);
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
    Ok(Value::from_string(result))
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
                    obj.properties.get("depth").map(|d| to_f64(d) as usize)
                } else {
                    None
                }
            }
            _ => None,
        })
        .unwrap_or(2);
    Ok(Value::from_string(inspect_value(interp, &value, depth, 0)))
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
                wk::NAME => Value::from_string(name),
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
                wk::NAME => Value::from_string(name),
            },
            prototype: None,
            extensible: true,
        }),
    );
    Ok(Value::Object(wrapper_idx))
}

/// `util.inherits(ctor, superCtor)` — classical inheritance helper.
pub(super) fn native_util_inherits(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let ctor = args.first().cloned().unwrap_or(Value::Undefined);
    let super_ctor = args.get(1).cloned().unwrap_or(Value::Undefined);
    if matches!(ctor, Value::Undefined | Value::Null)
        || matches!(super_ctor, Value::Undefined | Value::Null)
    {
        return Err(crate::errors::Error::TypeError(
            "util.inherits requires two arguments".into(),
        ));
    }
    // superCtor.prototype
    let super_proto = interp.get_property(&super_ctor, &Value::from_string("prototype".into()))?;
    // Object.create(superCtor.prototype)
    let child_proto = crate::runtime_env::native_fns::object_fns::native_object_create(
        interp,
        &Value::Undefined,
        &[super_proto],
    )?;
    // child_proto.constructor = ctor
    interp.set_property_str(&child_proto, "constructor", ctor.clone());
    // ctor.prototype = child_proto
    interp.set_property_str(&ctor, "prototype", child_proto);
    // ctor.super_ = superCtor
    interp.set_property_str(&ctor, "super_", super_ctor);
    Ok(Value::Undefined)
}

/// `util.deprecate(fn, msg[, code])` — Node returns a wrapper that emits a
/// one-time warning. We return the original function unchanged (no warning
/// emission yet), which is enough for libraries that only need the identity.
pub(super) fn native_util_deprecate(
    _interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    Ok(args.first().cloned().unwrap_or(Value::Undefined))
}

fn inspect_value(interp: &Interpreter, value: &Value, depth: usize, indent: usize) -> String {
    if depth == 0 {
        return "[Object]".to_string();
    }
    match value {
        Value::Undefined => wk::UNDEFINED.to_string(),
        Value::Null => wk::NULL.to_string(),
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
        Value::Cons(c) => format!("'{}'", c.flatten()),
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
                    .map(|v| {
                        format!(
                            "{}{}",
                            inner,
                            inspect_value(interp, v, depth - 1, indent + 2)
                        )
                    })
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
                let name = f.name.as_deref().unwrap_or("anonymous");
                let tag = if name == "anonymous" {
                    "[Function (anonymous)]".to_string()
                } else {
                    format!("[Function: {}]", name)
                };
                // Own properties (cloned so we can recurse without holding a borrow).
                let items_src: Vec<(String, Value)> = f
                    .properties
                    .keys()
                    .filter(|k| {
                        !k.starts_with("__getter_")
                            && !k.starts_with("__setter_")
                            && !k.starts_with("__method_")
                            && *k != "__[[Prototype]]__"
                            && *k != "constructor"
                    })
                    .filter_map(|k| f.properties.get(k).map(|v| (k.to_string(), v.clone())))
                    .collect();
                if items_src.is_empty() {
                    tag
                } else {
                    let prefix = " ".repeat(indent);
                    let inner = " ".repeat(indent + 2);
                    let items: Vec<String> = items_src
                        .iter()
                        .map(|(k, v)| {
                            format!(
                                "{}{}: {}",
                                inner,
                                k,
                                inspect_value(interp, v, depth.saturating_sub(1), indent + 2)
                            )
                        })
                        .collect();
                    format!("{} {{\n{}\n{}}}", tag, items.join(",\n"), prefix)
                }
            } else {
                "[Function]".to_string()
            }
        }
        Value::NativeFunction(idx) => format!("[Function: native({})]", idx),
        Value::Promise(idx) => format!("Promise {{ <{}> }}", idx),
        _ => "[Object]".to_string(),
    }
}
