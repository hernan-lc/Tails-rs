use crate::errors::Result;
use crate::objects::Value;
use crate::vm::interpreter::Interpreter;
use rustc_hash::FxHashMap;

/// native_require(specifier) — CommonJS require() function
///
/// Resolves the module path, checks the cache, reads the source,
/// compiles it, executes it with module/exports/require injected,
/// and returns module.exports.
pub(super) fn native_require(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let specifier = match args.first() {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Cons(c)) => c.flatten(),
        Some(v) => {
            return Err(crate::errors::Error::RuntimeError(format!(
                "require() expected a string argument, got {:?}",
                v
            )))
        }
        None => {
            return Err(crate::errors::Error::RuntimeError(
                "require() requires one argument".into(),
            ))
        }
    };

    // 1. Resolve the module path (fallback to native modules for bare names)
    let module_path = match interp.resolve_module_path(&specifier) {
        Ok(p) => p,
        Err(_) => {
            // Try as a native module (e.g., "path", "fs", "process")
            let module_name = &specifier;
            if !interp.native_loader.has_module(module_name) {
                crate::vm::interpreter::native_loader::discover_module(
                    module_name,
                    &mut interp.native_loader,
                );
            }
            if interp.native_loader.has_module(module_name) {
                if let Some(cached) = interp.require_cache.get(module_name) {
                    return Ok(cached.clone());
                }
                let exports = interp.native_loader.load_module(
                    module_name,
                    &mut interp.heap,
                    &mut interp.gc,
                )?;
                if module_name == "buffer" {
                    if let Some(Value::Object(proto_idx)) = exports.get("prototype") {
                        interp.buffer_proto_idx = Some(*proto_idx);
                    }
                }
                let mut props: FxHashMap<String, Value> = FxHashMap::default();
                for (name, val) in &exports {
                    props.insert(name.clone(), val.clone());
                }
                interp
                    .module_registry
                    .insert(module_name.to_string(), props.clone());
                let result = interp.build_module_object_from_exports(&props);
                interp
                    .require_cache
                    .insert(module_name.to_string(), result.clone());
                return Ok(result);
            }
            return Err(crate::errors::Error::RuntimeError(format!(
                "Cannot find module '{}'",
                specifier
            )));
        }
    };

    // 3. Check cache — return the same Value reference for identity preservation
    if let Some(cached) = interp.require_cache.get(&module_path) {
        return Ok(cached.clone());
    }

    // 3b. Check module_registry for circular dependencies (in-progress module)
    if interp.module_registry.contains_key(&module_path) {
        let partial = interp
            .module_registry
            .get(&module_path)
            .cloned()
            .unwrap_or_default();
        return Ok(interp.build_module_object_from_exports(&partial));
    }

    // 4. Read source
    let source_code = std::fs::read_to_string(&module_path).map_err(|e| {
        crate::errors::Error::RuntimeError(format!("Cannot read module '{}': {}", specifier, e))
    })?;

    // 5. Compile
    let compiler = crate::compiler::Compiler::new(false);
    let compiled = compiler.compile(&source_code)?;

    // 6. Create module and exports objects
    let module_obj = interp.new_object();
    let exports_obj = interp.new_object();
    interp.set_property_str(&module_obj, "exports", exports_obj.clone());

    // 7. Save current state
    let saved_module = interp.current_module.take();
    let saved_path = interp.current_module_path.take();
    let prev_exports = std::mem::take(&mut interp.module_exports);
    let saved_globals = std::mem::take(&mut interp.globals);

    // 8. Restore built-in globals + CJS globals
    for key in saved_globals.keys() {
        if key == "console"
            || key == "Object"
            || key == "JSON"
            || key == "Math"
            || key == "Proxy"
            || key == "Reflect"
            || key == "Error"
            || key == "TypeError"
            || key == "ReferenceError"
            || key == "SyntaxError"
            || key == "RangeError"
            || key == "Array"
            || key == "String"
            || key == "Number"
            || key == "Boolean"
            || key == "parseInt"
            || key == "parseFloat"
            || key == "isNaN"
            || key == "isFinite"
            || key == "setTimeout"
            || key == "setInterval"
            || key == "clearTimeout"
            || key == "clearInterval"
            || key == "Map"
            || key == "Set"
            || key == "WeakMap"
            || key == "WeakSet"
            || key == "Promise"
            || key == "Symbol"
            || key == "BigInt"
            || key == "Date"
            || key == "RegExp"
            || key == "URL"
            || key == "URLSearchParams"
            || key == "Headers"
            || key == "Request"
            || key == "Response"
            || key == "globalThis"
            || key == "fetch"
            || key == "WebSocket"
            || key == "require"
        {
            interp
                .globals
                .insert(key.clone(), saved_globals[key].clone());
        }
    }

    // 9. Set module path and pre-register (for circular deps)
    interp.current_module_path = Some(module_path.clone());
    interp
        .module_registry
        .insert(module_path.clone(), FxHashMap::default());

    // 10. Compute __dirname
    let dirname = std::path::Path::new(&module_path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| ".".to_string());

    // 11. Inject CJS globals: require, module, exports, __filename, __dirname
    interp
        .globals
        .insert("module".to_string(), module_obj.clone());
    interp
        .globals
        .insert("exports".to_string(), exports_obj.clone());
    interp
        .globals
        .insert("__filename".to_string(), Value::String(module_path.clone()));
    interp
        .globals
        .insert("__dirname".to_string(), Value::String(dirname));

    // 12. Execute the module
    let result = interp.execute(&compiled);

    // 13. Read module.exports (may have been reassigned by the module)
    let final_exports = interp
        .get_property_str(&module_obj, "exports")
        .unwrap_or(exports_obj);

    // 14. Store exports in registry
    let export_props = extract_object_properties(interp, &final_exports);
    interp
        .module_registry
        .insert(module_path.clone(), export_props.clone());
    interp.current_module_path = saved_path;
    interp.current_module = saved_module;
    let exec_exports = std::mem::replace(&mut interp.module_exports, prev_exports);

    // Restore parent globals
    interp.globals = saved_globals;

    // 16. Merge sub-module exports into parent module_exports
    for (k, v) in &exec_exports {
        interp.module_exports.insert(k.clone(), v.clone());
    }

    // 17. Check for compilation/execution errors
    result?;

    // If module.exports was reassigned to a non-object (function, string, etc.), return it directly
    let result = match &final_exports {
        Value::Object(_) => interp.build_module_object_from_exports(&export_props),
        other => other.clone(),
    };
    interp
        .require_cache
        .insert(module_path.clone(), result.clone());
    Ok(result)
}

/// Extract all properties from a JS object into a FxHashMap
fn extract_object_properties(interp: &Interpreter, obj: &Value) -> FxHashMap<String, Value> {
    let mut props = FxHashMap::default();
    if let Value::Object(idx) = obj {
        if let crate::vm::interpreter::HeapValue::Object(obj_data) = &interp.heap[*idx] {
            for (k, v) in &obj_data.properties {
                props.insert(k.clone(), v.clone());
            }
        }
    }
    props
}
