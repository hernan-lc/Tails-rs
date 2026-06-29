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

        // node_modules resolution for bare specifiers (no ./ or ../)
        if !source.starts_with('.') && !source.starts_with('/') {
            if let Some(resolved) = self.resolve_from_node_modules(source, parent) {
                return Ok(resolved);
            }
        }

        Err(crate::errors::Error::RuntimeError(format!(
            "Module '{}' not found",
            source
        )))
    }

    /// Walk up the directory tree looking for node_modules/<pkg> and resolve its entry point
    fn resolve_from_node_modules(
        &self,
        specifier: &str,
        start_dir: &std::path::Path,
    ) -> Option<String> {
        // Parse scoped package names: @scope/pkg/subpath -> (@scope/pkg, subpath)
        let (pkg_name, subpath) = if specifier.starts_with('@') {
            let mut parts = specifier.splitn(3, '/');
            let scope = parts.next()?;
            let name = parts.next()?;
            let sub = parts.next().unwrap_or("");
            (format!("{}/{}", scope, name), sub)
        } else {
            let mut parts = specifier.splitn(2, '/');
            let name = parts.next()?;
            let sub = parts.next().unwrap_or("");
            (name.to_string(), sub)
        };

        let mut current = Some(start_dir);
        while let Some(dir) = current {
            let pkg_dir = dir.join("node_modules").join(&pkg_name);
            if pkg_dir.is_dir() {
                return self.resolve_package_entry(&pkg_dir, subpath);
            }
            current = dir.parent();
        }
        None
    }

    /// Resolve a package's entry point from its package.json
    fn resolve_package_entry(
        &self,
        pkg_dir: &std::path::Path,
        subpath: &str,
    ) -> Option<String> {
        // If there's a subpath, resolve that directly
        if !subpath.is_empty() {
            let sub = pkg_dir.join(subpath);
            return self.resolve_with_fallbacks(&sub);
        }

        // Read package.json
        let pkg_json_path = pkg_dir.join("package.json");
        let pkg_json = std::fs::read_to_string(&pkg_json_path).ok()?;
        let pkg: serde_json::Value = serde_json::from_str(&pkg_json).ok()?;

        // Priority: exports > module > main > index.js
        // Try exports["."]["import"] > exports["."]["default"] > exports["."]
        if let Some(exports) = pkg.get("exports") {
            if let Some(resolved) = self.resolve_exports_field(exports, pkg_dir) {
                return Some(resolved);
            }
        }

        // Try "module" field (ESM)
        if let Some(module) = pkg.get("module").and_then(|v| v.as_str()) {
            let path = pkg_dir.join(module);
            if let Some(r) = self.resolve_with_fallbacks(&path) {
                return Some(r);
            }
        }

        // Try "main" field
        if let Some(main) = pkg.get("main").and_then(|v| v.as_str()) {
            let path = pkg_dir.join(main);
            if let Some(r) = self.resolve_with_fallbacks(&path) {
                return Some(r);
            }
        }

        // Fallback: index.js
        self.resolve_with_fallbacks(&pkg_dir.join("index.js"))
    }

    /// Try the "exports" field with condition resolution
    fn resolve_exports_field(
        &self,
        exports: &serde_json::Value,
        pkg_dir: &std::path::Path,
    ) -> Option<String> {
        match exports {
            // "exports": "./index.js"
            serde_json::Value::String(s) => {
                let path = pkg_dir.join(s);
                return self.resolve_with_fallbacks(&path);
            }
            // "exports": { ".": "...", "./sub": "..." }
            serde_json::Value::Object(map) => {
                // Look for "." entry (the main entry)
                if let Some(dot_entry) = map.get(".") {
                    return self.resolve_condition(dot_entry, pkg_dir);
                }
                // If no "." key, maybe it's a condition map directly (e.g. {"import":"...", "default":"..."})
                // Try "import" then "default"
                if let Some(import_val) = map.get("import") {
                    if let Some(r) = self.resolve_condition(import_val, pkg_dir) {
                        return Some(r);
                    }
                }
                if let Some(default_val) = map.get("default") {
                    if let Some(r) = self.resolve_condition(default_val, pkg_dir) {
                        return Some(r);
                    }
                }
            }
            _ => {}
        }
        None
    }

    /// Resolve a single condition value (string or nested condition map)
    fn resolve_condition(
        &self,
        value: &serde_json::Value,
        pkg_dir: &std::path::Path,
    ) -> Option<String> {
        match value {
            serde_json::Value::String(s) => {
                let path = pkg_dir.join(s);
                self.resolve_with_fallbacks(&path)
            }
            serde_json::Value::Object(map) => {
                // Nested conditions: try "import" > "default" > "require" > "node"
                for condition in &["import", "default", "require", "node"] {
                    if let Some(val) = map.get(*condition) {
                        if let Some(r) = self.resolve_condition(val, pkg_dir) {
                            return Some(r);
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// Try resolving a path with .ts, .js, /index.ts, /index.js fallbacks
    fn resolve_with_fallbacks(&self, path: &std::path::Path) -> Option<String> {
        if path.exists() && path.is_file() {
            return Some(path.to_string_lossy().to_string());
        }
        // Try extensions
        for ext in &[".ts", ".js", ".mjs"] {
            let with_ext = path.with_extension("");
            let candidate = std::path::PathBuf::from(format!("{}{}", with_ext.to_string_lossy(), ext));
            if candidate.exists() {
                return Some(candidate.to_string_lossy().to_string());
            }
        }
        // Try directory index
        if path.is_dir() {
            for name in &["index.ts", "index.js", "index.mjs"] {
                let idx = path.join(name);
                if idx.exists() {
                    return Some(idx.to_string_lossy().to_string());
                }
            }
        }
        None
    }
}
