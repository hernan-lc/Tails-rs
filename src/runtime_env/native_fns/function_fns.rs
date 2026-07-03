use crate::errors::{Error, Result};
use crate::objects::Value;
use crate::vm::interpreter::Interpreter;
use rustc_hash::FxHashMap;

/// Function.prototype.call(thisArg, ...args)
/// Calls a function with a given this value and arguments provided individually.
pub(super) fn native_function_call(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let this_arg = args.first().cloned().unwrap_or(Value::Undefined);
    let call_args = if args.len() > 1 {
        args[1..].to_vec()
    } else {
        Vec::new()
    };
    interp.call_value(this, &this_arg, &call_args)
}

/// Function.prototype.apply(thisArg, argsArray)
/// Calls a function with a given this value and arguments provided as an array.
pub(super) fn native_function_apply(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let this_arg = args.first().cloned().unwrap_or(Value::Undefined);
    let call_args = match args.get(1) {
        Some(Value::Array(arr_idx)) => {
            if let crate::vm::interpreter::HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                arr.elements.clone()
            } else {
                Vec::new()
            }
        }
        Some(Value::Undefined) | None => Vec::new(),
        _ => {
            return Err(Error::TypeError(
                "CreateListFromArrayLike called on non-object".into(),
            ))
        }
    };
    interp.call_value(this, &this_arg, &call_args)
}

/// Function.prototype.bind(thisArg, ...args)
/// Creates a new function that, when called, has its this keyword set to the provided value,
/// with a given sequence of arguments preceding any provided when the new function is called.
pub(super) fn native_function_bind(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let bound_this = args.first().cloned().unwrap_or(Value::Undefined);
    let bound_args = if args.len() > 1 {
        args[1..].to_vec()
    } else {
        Vec::new()
    };

    // Create a new function that captures the bound this and args
    // We'll store the original function and bound values in the closure
    let original_fn = this.clone();

    // Create a special "bound function" by creating a JsFunction with:
    // - bytecode_index = usize::MAX (marks it as special)
    // - closure = [original_fn, bound_this, ...bound_args]
    // - name = "bound " + original name
    let mut closure = vec![original_fn, bound_this];
    closure.extend(bound_args);

    let fn_idx = interp.heap.len();
    interp
        .heap
        .push(crate::vm::interpreter::HeapValue::Function(
            crate::vm::interpreter::JsFunction {
                name: Some("bound".into()),
                params: vec![],
                rest_param: None,
                bytecode_index: usize::MAX,
                closure,
                prototype: None,
                super_class: None,
                properties: FxHashMap::default(),
                owner_module: None,
                module_scope: None,
                is_generator: false,
                source_file: None,
                source_line: None,
                is_arrow: false,
                captured_this: None,
            },
        ));

    Ok(Value::Function(fn_idx))
}
