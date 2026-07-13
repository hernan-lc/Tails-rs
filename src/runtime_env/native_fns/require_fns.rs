use crate::errors::Result;
use crate::objects::Value;
use crate::vm::interpreter::{Interpreter, PropertyStorage};
use crate::well_known as wk;
use rustc_hash::FxHashMap;

/// native_require(specifier) — CommonJS require() function
///
/// Resolves the module path, checks the cache, reads the source,
/// compiles it, executes it with module/exports/require injected,
/// and returns module.exports.
pub(crate) fn native_require(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let specifier = match args.first() {
        Some(Value::String(s)) => s.to_string(),
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

    // Strip Node's `node:` builtin prefix (e.g. `node:events` → `events`).
    let bare_specifier = specifier
        .strip_prefix("node:")
        .unwrap_or(specifier.as_str());

    // Shim `get-intrinsic`: the real package's GetIntrinsic hits several
    // unfinished edges (complex regex + large free-var functions causing stack
    // underflows). A native walk of known globals is enough for express /
    // side-channel / call-bound.
    if bare_specifier == "get-intrinsic" {
        if let Some(cached) = interp.require_cache.get("get-intrinsic") {
            return Ok(cached.clone());
        }
        let fn_val =
            Value::NativeFunction(crate::runtime_env::native_fns::constants::GET_INTRINSIC);
        // call-bound does `require('get-intrinsic')` and calls the export as a
        // function — module.exports = GetIntrinsic (function, not {default}).
        interp
            .require_cache
            .insert("get-intrinsic".into(), fn_val.clone());
        return Ok(fn_val);
    }

    // Shim `debug`: the real package relies on rest-params + function accessors
    // + complex closures that still trip stack underflows when invoked. Express
    // only needs a no-op logger at load/init time.
    if bare_specifier == "debug" {
        if let Some(cached) = interp.require_cache.get("debug") {
            return Ok(cached.clone());
        }
        let fn_val = Value::NativeFunction(crate::runtime_env::native_fns::constants::DEBUG_NOOP);
        interp.require_cache.insert("debug".into(), fn_val.clone());
        return Ok(fn_val);
    }

    // Shim `safe-regex2`: find-my-way / fastify use it only for a load-time
    // catastrophic-backtracking safety assertion on their built-in route
    // regexes (which are simple and safe). The real package depends on `ret`,
    // whose regex tokenizer trips a runtime edge here; returning `true` keeps
    // the assertion satisfied without changing fastify's routing behavior.
    if bare_specifier == "safe-regex2" {
        if let Some(cached) = interp.require_cache.get("safe-regex2") {
            return Ok(cached.clone());
        }
        let fn_val = Value::NativeFunction(crate::runtime_env::native_fns::constants::SAFE_REGEX_TRUE);
        interp
            .require_cache
            .insert("safe-regex2".into(), fn_val.clone());
        return Ok(fn_val);
    }

    // 1. Resolve the module path (fallback to native modules for bare names)
    let module_path = match interp.resolve_module_path_with_context(bare_specifier, true) {
        Ok(p) => p,
        Err(_) => {
            // Try as a native module (e.g., "path", "fs", "process", "tty")
            let module_name = bare_specifier;
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
                // `assert` is both callable and a method bag, mirroring Node.
                // Return the callable NativeFunction; its methods resolve via
                // the `ASSERT` arm in `get_property`.
                if module_name == wk::MOD_ASSERT {
                    let result = Value::NativeFunction(
                        crate::runtime_env::native_fns::constants::ASSERT,
                    );
                    interp
                        .require_cache
                        .insert(module_name.to_string(), result.clone());
                    return Ok(result);
                }
                let exports = interp.native_loader.load_module(
                    module_name,
                    &mut interp.heap,
                    &mut interp.gc,
                )?;
                if module_name == wk::MOD_BUFFER {
                    if let Some(Value::Object(proto_idx)) = exports.get(wk::PROTOTYPE) {
                        interp.buffer_proto_idx = Some(*proto_idx);
                    }
                    // Buffer constructor object needs Object.prototype for hasOwnProperty
                    if let Some(Value::Object(ctor_idx)) = exports.get("Buffer") {
                        if let crate::vm::interpreter::HeapValue::Object(obj) =
                            &mut interp.heap[*ctor_idx]
                        {
                            if obj.prototype.is_none() {
                                obj.prototype = interp.object_proto_idx;
                            }
                        }
                    }
                }
                let mut props: FxHashMap<String, Value> = FxHashMap::default();
                for (name, val) in &exports {
                    props.insert(name.to_string(), val.clone());
                }
                interp
                    .module_registry
                    .insert(module_name.to_string(), props.clone());
                // Build the export object *without* tagging `__module_path` with
                // the caller's path. `build_module_object_from_exports` would
                // stamp the current CJS file path onto the native module, and
                // live-binding lookup in `get_property` would then resolve
                // every property against the *caller's* registry entry
                // (returning undefined for `util.deprecate`, `path.join`, etc.).
                let heap_idx = interp.heap.len();
                interp.heap.push(crate::vm::interpreter::HeapValue::Object(
                    crate::vm::interpreter::JsObject {
                        properties: PropertyStorage::Map(props),
                        prototype: interp.object_proto_idx,
                        extensible: true,
                    },
                ));
                let result = Value::Object(heap_idx);
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

    // 2. Canonicalize the resolved path so that different relative
    // specifiers mapping to the same physical file (e.g.
    // `./compile/codegen` vs `../compile/validate/../codegen` vs a bare
    // `ajv/dist/compile/codegen`) share a single cache entry. Without this,
    // CommonJS module caching breaks: the same module loads multiple times
    // under different `..`-laden keys, and the circular-dependency
    // pre-registration in `module_registry` can hand out an incomplete
    // (empty) exports snapshot — which is how `ajv-formats` saw
    // `codegen.operators` as `undefined`.
    let module_path = match std::fs::canonicalize(&module_path) {
        Ok(abs) => abs.to_string_lossy().to_string(),
        Err(_) => module_path,
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

    // 4b. JSON modules — Node returns the parsed object as module.exports
    if module_path.ends_with(".json") {
        let json: serde_json::Value = serde_json::from_str(&source_code).map_err(|e| {
            crate::errors::Error::SyntaxError(format!("JSON parse error in '{}': {}", specifier, e))
        })?;
        let value = super::helpers::from_json_value(interp, json);
        interp
            .require_cache
            .insert(module_path.clone(), value.clone());
        return Ok(value);
    }

    // 5. Compile
    let compiler = crate::compiler::Compiler::new(false);
    let compiled = compiler.compile(&source_code)?;

    // 6. Create module and exports objects
    let module_obj = interp.new_object();
    let exports_obj = interp.new_object();
    interp.set_property_str(&module_obj, "exports", exports_obj.clone());

    // 7. Save current state. We save/restore only the CJS injection globals
    // (module, exports, require, __filename, __dirname) so nested requires
    // don't clobber them. Other globals (like parent's `const X = ...`)
    // are left untouched so they persist across requires.
    let saved_module = interp.current_module.take();
    let saved_path = interp.current_module_path.take();
    let prev_exports = std::mem::take(&mut interp.module_exports);
    let saved_module_globals = interp.module_globals.take();
    let saved_module_globals_rc = interp.module_globals_rc.take();
    let saved_cjs_globals = {
        let mut g = FxHashMap::default();
        for key in &["module", "exports", "require", "__filename", "__dirname"] {
            if let Some(v) = interp.globals.get(*key) {
                g.insert(key.to_string(), v.clone());
            }
        }
        g
    };

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
    interp.globals.insert(
        "__filename".to_string(),
        Value::from_string(module_path.clone()),
    );
    interp
        .globals
        .insert("__dirname".to_string(), Value::from_string(dirname));

    // 12. Execute the module on an isolated operand stack so nested
    // `require()` / if-else / BlockExit cannot corrupt the caller's stack
    // (which may hold intermediate assignment slots like `module`, `"exports"`).
    //
    // Also isolate exception handlers and pending exceptions: parent try/catch
    // PCs are meaningless inside the child module bytecode. Leaking them made
    // host TypeErrors (e.g. `process.binding` missing) jump to the wrong PC,
    // leave `pending_exception` set, and rethrow after `require()` returned
    // (breaks safer-buffer under an outer try, and express's require tree).
    let module_scope_rc: std::rc::Rc<std::cell::RefCell<rustc_hash::FxHashMap<String, Value>>> =
        std::rc::Rc::new(std::cell::RefCell::new(interp.globals.clone()));
    interp.module_globals = Some(module_scope_rc.clone());
    interp.module_globals_rc = Some(module_scope_rc);
    let saved_stack = std::mem::take(&mut interp.stack);
    let saved_block_scopes = std::mem::take(&mut interp.block_scope_stack);
    let saved_exception_handlers = std::mem::take(&mut interp.exception_handlers);
    let saved_pending_exception = interp.pending_exception.take();
    let result = interp.execute(&compiled);
    interp.stack = saved_stack;
    interp.block_scope_stack = saved_block_scopes;
    interp.exception_handlers = saved_exception_handlers;
    interp.pending_exception = saved_pending_exception;

    // 13. Read module.exports (may have been reassigned by the module)
    let final_exports = interp
        .get_property_str(&module_obj, "exports")
        .unwrap_or(exports_obj);

    // 14. Store exports in registry
    let export_props = extract_object_properties(interp, &final_exports);
    let mut export_map: FxHashMap<String, Value> = export_props
        .iter()
        .map(|(k, v)| (k.to_string(), v.clone()))
        .collect();
    // If module.exports was reassigned to a non-object (e.g. a function
    // carrying properties like `stringify.configure`), ESM `import x from
    // 'cjs'` should resolve to that function directly. Stash it as the CJS
    // default so the ESM loader can return it, and also surface its own
    // properties (e.g. `configure`) as named exports after the function's
    // own props are read.
    if !matches!(final_exports, Value::Object(_)) {
        interp
            .cjs_default_exports
            .insert(module_path.clone(), final_exports.clone());
    } else {
        // Surface function/object property exports so `import { configure }`
        // and `cjsModule.configure` both resolve.
        let own_props = extract_object_properties(interp, &final_exports);
        for (k, v) in own_props.iter() {
            export_map.entry(k.to_string()).or_insert_with(|| v.clone());
        }
    }
    interp
        .module_registry
        .insert(module_path.clone(), export_map.clone());
    interp.current_module_path = saved_path;
    interp.current_module = saved_module;
    let exec_exports = std::mem::replace(&mut interp.module_exports, prev_exports);

    // Restore only the CJS injection globals so nested requires don't clobber
    // the parent's module/exports bindings. Other globals (parent's top-level
    // `const X = require(...)` assignments) are left untouched.
    for (key, value) in &saved_cjs_globals {
        interp.globals.insert(key.clone(), value.clone());
    }
    interp.module_globals = saved_module_globals;
    interp.module_globals_rc = saved_module_globals_rc;

    // 16. Merge sub-module exports into parent module_exports
    for (k, v) in &exec_exports {
        interp.module_exports.insert(k.clone(), v.clone());
    }

    // 17. Check for compilation/execution errors
    result?;

    // If module.exports was reassigned to a non-object (function, string, etc.), return it directly
    // Temporarily set current_module_path so build_module_object_from_exports tags the
    // result with the correct module path (not the parent's).
    let saved_mp = interp.current_module_path.take();
    interp.current_module_path = Some(module_path.clone());
    let result = match &final_exports {
        Value::Object(_) => interp.build_module_object_from_exports(&export_map),
        other => other.clone(),
    };
    interp.current_module_path = saved_mp;
    interp
        .require_cache
        .insert(module_path.clone(), result.clone());
    Ok(result)
}

/// `module.createRequire(filename)` — returns a `require` function bound to
/// the given (ignored) path. We just hand back the runtime's built-in
/// `require` (index `c::REQUIRE`), which resolves relative to the caller's
/// CJS context.
pub(super) fn native_module_create_require(
    _interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    Ok(Value::NativeFunction(
        crate::runtime_env::native_fns::constants::REQUIRE,
    ))
}

/// Extract all properties from a JS object into a PropertyStorage
fn extract_object_properties(interp: &Interpreter, obj: &Value) -> PropertyStorage {
    let mut props = PropertyStorage::new();
    if let Value::Object(idx) = obj {
        if let crate::vm::interpreter::HeapValue::Object(obj_data) = &interp.heap[*idx] {
            for (k, v) in &obj_data.properties {
                props.insert(k.to_string(), v.clone());
            }
        }
    }
    props
}
