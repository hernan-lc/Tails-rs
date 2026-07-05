use crate::errors::{Error, Result};
use crate::objects::Value;
use crate::vm::interpreter::{Interpreter, PropertyStorage};
use std::cell::RefCell;
use std::rc::Rc;

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
    let closure_rc = Rc::new(RefCell::new(closure));

    let fn_idx = interp.heap.len();
    interp
        .heap
        .push(crate::vm::interpreter::HeapValue::Function(
            crate::vm::interpreter::JsFunction {
                name: Some("bound".into()),
                params: vec![],
                rest_param: None,
                bytecode_index: usize::MAX,
                closure: closure_rc,
                prototype: None,
                super_class: None,
                properties: PropertyStorage::new(),
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

/// Function constructor: new Function(...paramBodies, body)
///
/// The last argument is always the function body. All preceding arguments
/// are parameter names (as strings). This compiles the body at runtime and
/// returns a callable function value.
pub(super) fn native_function_constructor(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    if args.is_empty() {
        return Err(Error::TypeError(
            "Function constructor requires at least one argument".into(),
        ));
    }

    let body_idx = args.len() - 1;
    let body = match &args[body_idx] {
        Value::String(s) => s.clone(),
        Value::Cons(c) => c.flatten(),
        other => {
            return Err(Error::TypeError(format!(
                "Function body must be a string, got {:?}",
                other
            )))
        }
    };

    let mut param_names: Vec<String> = Vec::new();
    for arg in &args[..body_idx] {
        match arg {
            Value::String(s) => param_names.push(s.clone()),
            Value::Cons(c) => param_names.push(c.flatten()),
            other => {
                return Err(Error::TypeError(format!(
                    "Function parameter name must be a string, got {:?}",
                    other
                )))
            }
        }
    }

    // Build a source string: function __tails_anon__(...params) { body }
    let params_str = param_names.join(", ");
    // Ensure body ends with a semicolon so the parser doesn't choke on missing ASI
    let body_trimmed = body.trim_end();
    let body_with_semi = if body_trimmed.ends_with(';') || body_trimmed.ends_with('}') {
        body_trimmed.to_string()
    } else {
        format!("{};", body_trimmed)
    };
    let source = format!(
        "function __tails_anon__({}) {{ {} }}",
        params_str, body_with_semi
    );

    // Compile the source
    let compiler = crate::compiler::Compiler::new(false);
    let compiled = compiler.compile(&source)?;

    // Save and restore interpreter state around execution
    let saved_module = interp.current_module.take();
    let saved_path = interp.current_module_path.take();
    let saved_mg = interp.module_globals.take();
    let saved_eh = std::mem::take(&mut interp.exception_handlers);

    // Execute the compiled module (defines __tails_anon__ as a global)
    let result = interp.execute(&compiled);

    interp.current_module = saved_module;
    interp.current_module_path = saved_path;
    interp.module_globals = saved_mg;
    interp.exception_handlers = saved_eh;

    // Check for compilation/execution errors
    result?;

    // Retrieve the function from globals
    let func = interp.globals.remove("__tails_anon__").ok_or_else(|| {
        Error::RuntimeError("Function constructor: compiled body did not produce a function".into())
    })?;

    Ok(func)
}
