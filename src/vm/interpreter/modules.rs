use super::*;
use crate::errors::Result;
use crate::objects::Value;
use std::rc::Rc;

impl Interpreter {
    pub fn execute_module(&mut self, module: &CompiledModule) -> Result<Value> {
        let saved_module = self.current_module.take();
        self.current_module = Some(Rc::new(module.clone()));
        let prev_exports = std::mem::take(&mut self.module_exports);
        let saved_globals = std::mem::take(&mut self.globals);
        // Restore built-in globals into the fresh scope
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
                || key == "fetch"
                || key == "WebSocket"
            {
                self.globals.insert(key.clone(), saved_globals[key].clone());
            }
        }
        let result = self.execute(module);
        let module_globals = std::mem::replace(&mut self.globals, saved_globals.clone());
        let exec_exports = std::mem::replace(&mut self.module_exports, prev_exports);
        for (k, v) in &exec_exports {
            self.module_exports.insert(k.clone(), v.clone());
        }
        // Restore exported values
        for (k, v) in &exec_exports {
            if let Some(mv) = module_globals.get(k) {
                self.globals.insert(k.clone(), mv.clone());
            } else {
                self.globals.insert(k.clone(), v.clone());
            }
        }
        // Also restore named imports that the module registered (via ImportNamed/ImportDefault/ImportAll)
        // These are globals that exist in module_globals but not in the original saved_globals
        for (k, v) in &module_globals {
            if !saved_globals.contains_key(k) {
                // This is a new global added by the module (import or local)
                // Only preserve it if it was an import (i.e., the value exists in a module registry)
                // For simplicity, we check if the value is a function or came from a module
                // We keep it if it's a function (imported functions) or if it was explicitly imported
                // Module-local variables like `const x = 1` should NOT be preserved
                // But imported functions like `validatePlugin` SHOULD be preserved
                // We use a heuristic: if it's a function or if the key was already in module_exports
                if matches!(v, Value::Function(_) | Value::NativeFunction(_))
                    || exec_exports.contains_key(k)
                {
                    self.globals.insert(k.clone(), v.clone());
                }
            }
        }
        self.current_module = saved_module;
        result
    }

    pub(crate) fn load_and_run_module(&mut self, source: &str) -> Result<Option<String>> {
        if source.ends_with(".native") {
            let module_name = super::native_loader::extract_module_name(source);
            if !self.native_loader.has_module(module_name) {
                // Discover and register the module lazily
                super::native_loader::discover_module(module_name, &mut self.native_loader);
            }
            if self.native_loader.has_module(module_name) {
                let exports =
                    self.native_loader
                        .load_module(module_name, &mut self.heap, &mut self.gc)?;
                // Set buffer_proto_idx when buffer module is loaded
                if module_name == "buffer" {
                    if let Some(Value::Object(proto_idx)) = exports.get("prototype") {
                        self.buffer_proto_idx = Some(*proto_idx);
                    }
                }
                let mut props = HashMap::new();
                for (name, val) in &exports {
                    props.insert(name.clone(), val.clone());
                }
                self.module_registry.insert(module_name.to_string(), props);
                return Ok(Some(module_name.to_string()));
            }
            return Ok(None);
        }

        let module_path = match self.resolve_module_path(source) {
            Ok(p) => p,
            Err(_) => {
                // Bare native module name (e.g., "fs", "path", "process")
                let module_name = source;
                if !self.native_loader.has_module(module_name) {
                    super::native_loader::discover_module(module_name, &mut self.native_loader);
                }
                if self.native_loader.has_module(module_name) {
                    let exports = self.native_loader.load_module(
                        module_name,
                        &mut self.heap,
                        &mut self.gc,
                    )?;
                    if module_name == "buffer" {
                        if let Some(Value::Object(proto_idx)) = exports.get("prototype") {
                            self.buffer_proto_idx = Some(*proto_idx);
                        }
                    }
                    let mut props = HashMap::new();
                    for (name, val) in &exports {
                        props.insert(name.clone(), val.clone());
                    }
                    self.module_registry.insert(module_name.to_string(), props);
                    return Ok(Some(module_name.to_string()));
                }
                return Ok(None);
            }
        };
        if self.module_registry.contains_key(&module_path) {
            return Ok(Some(module_path));
        }
        let source_code = match std::fs::read_to_string(&module_path) {
            Ok(s) => s,
            Err(_) => return Ok(None),
        };
        let compiler = crate::compiler::Compiler::new(false);
        let compiled = compiler.compile(&source_code)?;
        let prev_path = self.current_module_path.take();
        self.current_module_path = Some(module_path.clone());
        self.module_registry
            .insert(module_path.clone(), HashMap::new());
        let result = self.execute_module(&compiled);
        let exports = std::mem::take(&mut self.module_exports);
        *self.module_registry.entry(module_path.clone()).or_default() = exports;
        self.current_module_path = prev_path;
        result?;
        Ok(Some(module_path))
    }

    pub(crate) fn resolve_module_path(&self, source: &str) -> Result<String> {
        let base = self.current_module_path.as_deref().unwrap_or(".");
        let base_path = std::path::Path::new(base);
        let parent = base_path.parent().unwrap_or(std::path::Path::new("."));

        if source.ends_with(".native") {
            return Ok(source.to_string());
        }

        let resolved = if source.starts_with("./") || source.starts_with("../") {
            parent.join(source)
        } else {
            std::path::PathBuf::from(source)
        };
        let normalized: std::path::PathBuf = resolved.components().collect();
        if normalized.exists() && normalized.is_file() {
            return Ok(normalized.to_string_lossy().to_string());
        }
        for ext in &[".ts", ".js"] {
            let stem = normalized.with_extension("");
            let candidate = std::path::PathBuf::from(format!("{}{}", stem.to_string_lossy(), ext));
            if candidate.exists() {
                return Ok(candidate.to_string_lossy().to_string());
            }
        }
        if normalized.is_dir() {
            for name in &["index.ts", "index.js"] {
                let idx = normalized.join(name);
                if idx.exists() {
                    return Ok(idx.to_string_lossy().to_string());
                }
            }
        }
        Err(crate::errors::Error::RuntimeError(format!(
            "Module '{}' not found",
            source
        )))
    }
}
