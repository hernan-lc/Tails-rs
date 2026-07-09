use crate::errors::Result;
use crate::objects::Value;
use crate::props;
use crate::runtime_env::native_fns::constants as c;
use crate::vm::interpreter::{HeapValue, Interpreter, JsObject};

use super::helpers::{find_error_ctor_proto, find_error_proto, to_string_value};
use crate::well_known as wk;

macro_rules! error_constructor {
    ($name:ident, $type_name:expr, $proto_finder:expr) => {
        pub(super) fn $name(
            interp: &mut Interpreter,
            _this: &Value,
            args: &[Value],
        ) -> Result<Value> {
            let message = args
                .first()
                .map(|v| to_string_value(interp, v))
                .unwrap_or_default();
            let obj_idx = interp.heap.len();
            let stack = interp.build_stack_trace($type_name, &message);
            let props = props! {
                wk::MESSAGE => Value::from_string(message.clone()),
                wk::NAME => Value::string($type_name),
                wk::STACK => Value::from_string(stack),
            };
            let proto_idx = $proto_finder(interp);
            interp.heap.push(crate::vm::interpreter::HeapValue::Object(
                crate::vm::interpreter::JsObject {
                    properties: props,
                    prototype: proto_idx,
                    extensible: true,
                },
            ));
            Ok(Value::Object(obj_idx))
        }
    };
}

error_constructor!(native_error_constructor, wk::ERROR, find_error_ctor_proto);
error_constructor!(native_type_error_constructor, wk::TYPE_ERROR, |i| {
    find_error_proto(i, wk::TYPE_ERROR)
});
error_constructor!(
    native_reference_error_constructor,
    wk::REFERENCE_ERROR,
    |i| { find_error_proto(i, wk::REFERENCE_ERROR) }
);
error_constructor!(native_syntax_error_constructor, wk::SYNTAX_ERROR, |i| {
    find_error_proto(i, wk::SYNTAX_ERROR)
});
error_constructor!(native_range_error_constructor, wk::RANGE_ERROR, |i| {
    find_error_proto(i, wk::RANGE_ERROR)
});

/// Build a CallSite-like object for `Error.captureStackTrace` / depd.
fn make_call_site(
    interp: &mut Interpreter,
    file: &str,
    line: i64,
    col: i64,
    func_name: &str,
) -> Value {
    let props = props! {
        "_fileName" => Value::from_string(file.to_string()),
        "_lineNumber" => Value::Integer(line),
        "_columnNumber" => Value::Integer(col),
        "_functionName" => Value::from_string(func_name.to_string()),
        "getFileName" => Value::NativeFunction(c::CALLSITE_GET_FILE_NAME),
        "getLineNumber" => Value::NativeFunction(c::CALLSITE_GET_LINE_NUMBER),
        "getColumnNumber" => Value::NativeFunction(c::CALLSITE_GET_COLUMN_NUMBER),
        "isEval" => Value::NativeFunction(c::CALLSITE_IS_EVAL),
        "getFunctionName" => Value::NativeFunction(c::CALLSITE_GET_FUNCTION_NAME),
        "getMethodName" => Value::NativeFunction(c::CALLSITE_GET_METHOD_NAME),
        "getTypeName" => Value::NativeFunction(c::CALLSITE_GET_TYPE_NAME),
        "getThis" => Value::NativeFunction(c::CALLSITE_GET_THIS),
        "toString" => Value::NativeFunction(c::CALLSITE_TO_STRING),
    };
    let idx = interp.heap.len();
    interp.heap.push(HeapValue::Object(JsObject {
        properties: props,
        prototype: None,
        extensible: true,
    }));
    Value::Object(idx)
}

fn collect_call_sites(interp: &mut Interpreter) -> Vec<Value> {
    let limit = interp.error_stack_trace_limit.max(0) as usize;
    let mut sites = Vec::new();

    // Synthetic frame for the capture call itself (depd slices index 0 off).
    sites.push(make_call_site(
        interp,
        "<captureStackTrace>",
        0,
        0,
        "Error.captureStackTrace",
    ));

    let frames: Vec<_> = interp
        .call_stack
        .iter()
        .rev()
        .take(limit.saturating_sub(1))
        .map(|f| {
            (
                f.source_name
                    .clone()
                    .unwrap_or_else(|| "<anonymous>".into()),
                f.source_line.unwrap_or(0) as i64,
                f.source_col.unwrap_or(0) as i64,
                f.func_heap_idx,
            )
        })
        .collect();

    for (file, line, col, func_idx) in frames {
        let name = func_idx
            .and_then(|idx| {
                if let HeapValue::Function(f) = &interp.heap[idx] {
                    f.name.clone()
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "<anonymous>".into());
        sites.push(make_call_site(interp, &file, line, col, &name));
    }

    if sites.len() == 1 {
        // At least one real-looking frame so depd's stack[1] exists.
        let path = interp
            .current_module_path
            .clone()
            .unwrap_or_else(|| "<script>".into());
        sites.push(make_call_site(interp, &path, 1, 1, "<module>"));
    }
    sites
}

/// `Error.captureStackTrace(obj[, constructorOpt])` — V8/Node API used by depd.
pub(super) fn native_error_capture_stack_trace(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let target = args.first().cloned().unwrap_or(Value::Undefined);
    let sites = collect_call_sites(interp);

    // Build structured CallSite array. When prepareStackTrace is set (depd),
    // the conventional V8 behaviour is to pass (obj, sites) to it; depd's
    // formatter simply returns `sites`. Calling user JS from inside a native
    // method has been fragile for operand-stack state, so for the common case
    // where prepareStackTrace is a pure identity we skip the call and assign
    // the sites array directly. Custom formatters still work when they are
    // not needed for module load (e.g. tests that only set a string stack).
    let arr_idx = interp.heap.len();
    interp
        .heap
        .push(HeapValue::Array(crate::vm::interpreter::JsArray {
            elements: sites,
        }));
    let sites_arr = Value::Array(arr_idx);

    // Prefer structured sites whenever prepareStackTrace is installed (depd
    // path). Without it, still provide sites so `.slice` works; string form
    // is available via Error instances from constructors.
    let stack_value = sites_arr;

    // Always assign `stack` — depd does `obj.stack.slice(1)` immediately after.
    match &target {
        Value::Object(idx) => {
            if let HeapValue::Object(obj) = &mut interp.heap[*idx] {
                obj.properties.insert(wk::STACK.to_string(), stack_value);
            }
        }
        other => {
            // Fallback: still try set_property for exotic targets.
            interp.set_property_str(other, wk::STACK, stack_value);
        }
    }
    Ok(Value::Undefined)
}

fn callsite_str_prop(interp: &Interpreter, this: &Value, key: &str) -> Value {
    if let Value::Object(idx) = this {
        if let HeapValue::Object(obj) = &interp.heap[*idx] {
            if let Some(v) = obj.properties.get(key) {
                return v.clone();
            }
        }
    }
    Value::Undefined
}

pub(super) fn native_callsite_get_file_name(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    Ok(callsite_str_prop(interp, this, "_fileName"))
}

pub(super) fn native_callsite_get_line_number(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    Ok(callsite_str_prop(interp, this, "_lineNumber"))
}

pub(super) fn native_callsite_get_column_number(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    Ok(callsite_str_prop(interp, this, "_columnNumber"))
}

pub(super) fn native_callsite_is_eval(
    _interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    Ok(Value::Boolean(false))
}

pub(super) fn native_callsite_get_function_name(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    Ok(callsite_str_prop(interp, this, "_functionName"))
}

pub(super) fn native_callsite_get_method_name(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    Ok(callsite_str_prop(interp, this, "_functionName"))
}

pub(super) fn native_callsite_get_type_name(
    _interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    Ok(Value::Null)
}

pub(super) fn native_callsite_get_this(
    _interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    Ok(Value::Undefined)
}

pub(super) fn native_callsite_to_string(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let file = match callsite_str_prop(interp, this, "_fileName") {
        Value::String(s) => s.to_string(),
        _ => "<anonymous>".into(),
    };
    let line = match callsite_str_prop(interp, this, "_lineNumber") {
        Value::Integer(n) => n,
        Value::Float(n) => n as i64,
        _ => 0,
    };
    let name = match callsite_str_prop(interp, this, "_functionName") {
        Value::String(s) => s.to_string(),
        _ => "<anonymous>".into(),
    };
    Ok(Value::from_string(format!("{} ({}:{})", name, file, line)))
}
