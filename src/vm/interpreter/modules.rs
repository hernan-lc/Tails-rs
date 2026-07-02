use super::safe_library::SafeLibrary;
use super::*;
use crate::errors::Result;
use crate::objects::Value;
use crate::runtime_env::native_fns::NATIVE_TABLE;
use rustc_hash::FxHashMap;
use std::path::Path;
use std::rc::Rc;

impl Interpreter {
    fn load_native_library(&mut self, lib_path: &Path) -> Option<FxHashMap<String, Value>> {
        let library = SafeLibrary::new(lib_path).ok()?;
        let mut props = FxHashMap::default();

        type InitFn = fn() -> *mut tails_abi::ModuleHandle;

        // Try module-specific init first, then fallback to generic
        let module_stem = lib_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let base_name = module_stem.strip_prefix("lib").unwrap_or(module_stem);

        let init_fn = unsafe {
            // Try several naming conventions to find the init function. The
            // `tails_native_init_*` macro emits a symbol that includes the
            // full module name (e.g. `tails_native_init_tails_process`), so
            // we also try prepending the `tails_` prefix.
            let candidates = [
                format!("tails_native_init_{}", base_name.replace('-', "_")),
                format!("tails_native_init_{}", base_name),
                format!("tails_native_init_tails_{}", base_name.replace('-', "_")),
                format!("tails_native_init_tails_{}", base_name),
                "tails_native_init".to_string(),
            ];
            let mut found = None;
            for name in &candidates {
                if let Ok(f) = library.get_function::<InitFn>(name) {
                    found = Some(f);
                    break;
                }
            }
            found
        };

        let init_fn = init_fn?;
        let handle = init_fn();
        if handle.is_null() {
            return None;
        }
        // Safety: handle comes from tails_native_init and is checked for null
        let handle = unsafe { Box::from_raw(handle) };

        for (func_name, func_ptr) in &handle.module.functions {
            let raw_fn = *func_ptr as usize;
            let idx = self.dynamic_native_fns.len();
            self.dynamic_native_fns.push(raw_fn);

            props.insert(
                func_name.clone(),
                Value::NativeFunction(NATIVE_TABLE.len() + idx),
            );
        }

        // Register class methods from DTS metadata
        let mut class_methods: HashMap<String, FxHashMap<String, Value>> = HashMap::new();
        for func_name in props.keys() {
            if let Some(method_name) = func_name.strip_prefix("counter_") {
                class_methods
                    .entry("Counter".to_string())
                    .or_default()
                    .insert(method_name.to_string(), props[func_name].clone());
            }
        }
        for (class_name, methods) in class_methods {
            self.native_class_registry.insert(class_name, methods);
        }

        // Safety: The library must remain loaded for the function pointers in props
        // to stay valid. Dropping SafeLibrary would unload the underlying Library,
        // invalidating every pointer we stored in `props`. Using `mem::forget`
        // intentionally skips the Drop impl so the native library stays in memory
        // for the lifetime of the process.
        std::mem::forget(library);

        Some(props)
    }

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
                || key == "globalThis"
                || key == "globalThis"
                || key == "globalThis"
                || key == "fetch"
                || key == "WebSocket"
                || key == "require"
                || key == "Buffer"
                || key == "process"
            {
                self.globals.insert(key.clone(), saved_globals[key].clone());
            }
        }

        // Inject __filename, __dirname, module, exports globals
        if let Some(ref path) = self.current_module_path {
            self.globals
                .insert("__filename".to_string(), Value::String(path.clone()));
            let dir = std::path::Path::new(path)
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| ".".to_string());
            self.globals
                .insert("__dirname".to_string(), Value::String(dir));

            // inject import.meta with url property (file:// URL)
            let file_url = if path.starts_with('/') {
                format!("file://{}", path)
            } else {
                format!(
                    "file://{}/{}",
                    std::env::current_dir()
                        .map(|d| d.to_string_lossy().to_string())
                        .unwrap_or_default(),
                    path
                )
            };
            let mut meta_props = FxHashMap::default();
            meta_props.insert("url".to_string(), Value::String(file_url));
            let meta_obj_idx = self.gc.allocate(
                &mut self.heap,
                HeapValue::Object(crate::vm::interpreter::JsObject {
                    properties: meta_props,
                    prototype: None,
                    extensible: true,
                }),
            );
            let mut import_props = FxHashMap::default();
            import_props.insert("meta".to_string(), Value::Object(meta_obj_idx));
            let import_obj_idx = self.gc.allocate(
                &mut self.heap,
                HeapValue::Object(crate::vm::interpreter::JsObject {
                    properties: import_props,
                    prototype: None,
                    extensible: true,
                }),
            );
            self.globals
                .insert("import".to_string(), Value::Object(import_obj_idx));

            // Also inject module and exports for CJS compatibility
            let module_obj = self.new_object();
            let exports_obj = self.new_object();
            self.set_property_str(&module_obj, "exports", exports_obj.clone());
            self.globals.insert("module".to_string(), module_obj);
            self.globals.insert("exports".to_string(), exports_obj);
        }

        let result = {
            self.module_globals_rc = None;
            self.execute(module)
        };
        self.module_globals_rc = Some(std::rc::Rc::new(self.globals.clone()));
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
                // Resolve the path relative to the importing file's directory
                let resolved_path = if source.starts_with("./") || source.starts_with("../") {
                    // Relative path - resolve from current module's directory
                    let base = self.current_module_path.as_deref().unwrap_or(".");
                    let base_path = std::path::Path::new(base);
                    let parent = base_path.parent().unwrap_or(std::path::Path::new("."));
                    parent.join(source)
                } else if source.starts_with('/') {
                    // Absolute path
                    std::path::PathBuf::from(source)
                } else {
                    // Bare specifier like "my-module.native"
                    // Check node_modules first
                    let base = self.current_module_path.as_deref().unwrap_or(".");
                    let base_path = std::path::Path::new(base);
                    let parent = base_path.parent().unwrap_or(std::path::Path::new("."));

                    if let Some(resolved) = self.resolve_from_node_modules(source, parent) {
                        std::path::PathBuf::from(resolved)
                    } else {
                        // Fall back to ./dist/
                        let dist_dir = std::env::current_dir().unwrap_or_default().join("dist");
                        // Try with lib prefix and platform extension
                        let ext = if cfg!(target_os = "windows") {
                            "dll"
                        } else if cfg!(target_os = "macos") {
                            "dylib"
                        } else {
                            "so"
                        };
                        let with_ext = dist_dir.join(format!("{}.{}", module_name, ext));
                        if with_ext.exists() {
                            with_ext
                        } else {
                            let with_lib = dist_dir.join(format!("lib{}.{}", module_name, ext));
                            if with_lib.exists() {
                                with_lib
                            } else {
                                dist_dir.join(format!("{}.{}", module_name, ext))
                            }
                        }
                    }
                };

                // Try to load from the resolved path directly
                if resolved_path.exists() {
                    if let Some(props) = self.load_native_library(&resolved_path) {
                        self.module_registry.insert(module_name.to_string(), props);
                        return Ok(Some(module_name.to_string()));
                    }
                }

                // If the resolved path doesn't exist or failed to load,
                // try to find the library in the directory containing the resolved path
                if let Some(parent_dir) = resolved_path.parent() {
                    if let Some(lib_path) =
                        super::native_loader::find_library_in_dir(parent_dir, module_name)
                    {
                        if let Some(props) = self.load_native_library(&lib_path) {
                            self.module_registry.insert(module_name.to_string(), props);
                            return Ok(Some(module_name.to_string()));
                        }
                    }
                }

                // As a last resort, look in the project's ./dist/ directory.
                // This makes relative imports like `import x from "./x.native"`
                // work even when the script is in a temp directory, provided
                // the user has run `tails build` and the .so is in dist/.
                let dist_dir = std::env::current_dir().unwrap_or_default().join("dist");
                if let Some(lib_path) =
                    super::native_loader::find_library_in_dir(&dist_dir, module_name)
                {
                    if let Some(props) = self.load_native_library(&lib_path) {
                        self.module_registry.insert(module_name.to_string(), props);
                        return Ok(Some(module_name.to_string()));
                    }
                }

                // Fall back to static registration
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
                let mut props = FxHashMap::default();
                for (name, val) in &exports {
                    props.insert(name.clone(), val.clone());
                }
                self.module_registry.insert(module_name.to_string(), props);
                return Ok(Some(module_name.to_string()));
            }
            return Err(Error::RuntimeError(format!(
                "Cannot find module '{}'",
                source
            )));
        }

        let module_path = match self.resolve_module_path(source) {
            Ok(p) => p,
            Err(_) => {
                // Bare native module name (e.g., "fs", "path", "process")
                let module_name = source;
                if !self.native_loader.has_module(module_name) {
                    // Try to load from ./dist/ first via the proper
                    // `load_native_library` path so the function pointers
                    // get registered in `dynamic_native_fns`.
                    let dist_dir = std::env::current_dir().unwrap_or_default().join("dist");
                    if let Some(lib_path) =
                        super::native_loader::find_library_in_dir(&dist_dir, module_name)
                    {
                        if let Some(props) = self.load_native_library(&lib_path) {
                            self.module_registry.insert(module_name.to_string(), props);
                            return Ok(Some(module_name.to_string()));
                        }
                    }
                    // Fall back to static registration
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
                    let mut props = FxHashMap::default();
                    for (name, val) in &exports {
                        props.insert(name.clone(), val.clone());
                    }
                    self.module_registry.insert(module_name.to_string(), props);
                    return Ok(Some(module_name.to_string()));
                }
                return Err(Error::RuntimeError(format!(
                    "Cannot find module '{}'",
                    source
                )));
            }
        };
        if self.module_registry.contains_key(&module_path) {
            return Ok(Some(module_path));
        }
        let source_code = std::fs::read_to_string(&module_path)
            .map_err(|e| Error::RuntimeError(format!("Cannot read module '{}': {}", source, e)))?;
        let compiler = crate::compiler::Compiler::new(false);
        let compiled = compiler.compile(&source_code)?;
        let prev_path = self.current_module_path.take();
        self.current_module_path = Some(module_path.clone());
        self.module_registry
            .insert(module_path.clone(), FxHashMap::default());
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

        let is_bare = !source.starts_with('.') && !source.starts_with('/');

        // For bare specifiers (e.g. "valibot"), skip local file checks and go straight to node_modules
        if !is_bare {
            let resolved = if source.starts_with("./") || source.starts_with("../") {
                parent.join(source)
            } else {
                std::path::PathBuf::from(source)
            };
            let normalized: std::path::PathBuf = resolved.components().collect();
            if normalized.exists() && normalized.is_file() {
                return Ok(normalized.to_string_lossy().to_string());
            }
            for ext in &[".ts", ".js", ".cjs"] {
                let stem = normalized.with_extension("");
                let candidate =
                    std::path::PathBuf::from(format!("{}{}", stem.to_string_lossy(), ext));
                if candidate.exists() {
                    return Ok(candidate.to_string_lossy().to_string());
                }
            }
            if normalized.is_dir() {
                for name in &["index.ts", "index.js", "index.cjs"] {
                    let idx = normalized.join(name);
                    if idx.exists() {
                        return Ok(idx.to_string_lossy().to_string());
                    }
                }
            }
        }

        // node_modules resolution for bare specifiers (no ./ or ../)
        if is_bare {
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
    fn resolve_package_entry(&self, pkg_dir: &std::path::Path, subpath: &str) -> Option<String> {
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

    /// Try resolving a path with .ts, .js, .cjs, /index.ts, /index.js fallbacks
    fn resolve_with_fallbacks(&self, path: &std::path::Path) -> Option<String> {
        if path.exists() && path.is_file() {
            return Some(path.to_string_lossy().to_string());
        }
        for ext in &[".ts", ".js", ".mjs", ".cjs"] {
            let with_ext = path.with_extension("");
            let candidate =
                std::path::PathBuf::from(format!("{}{}", with_ext.to_string_lossy(), ext));
            if candidate.exists() {
                return Some(candidate.to_string_lossy().to_string());
            }
        }
        if path.is_dir() {
            for name in &["index.ts", "index.js", "index.mjs", "index.cjs"] {
                let idx = path.join(name);
                if idx.exists() {
                    return Some(idx.to_string_lossy().to_string());
                }
            }
        }
        None
    }

    pub(crate) fn build_module_object_from_exports(
        &mut self,
        exports: &FxHashMap<String, Value>,
    ) -> Value {
        let heap_idx = self.heap.len();
        let mut props = FxHashMap::default();
        for (k, v) in exports {
            props.insert(k.clone(), v.clone());
        }
        self.heap.push(HeapValue::Object(JsObject {
            properties: props,
            prototype: None,
            extensible: true,
        }));
        Value::Object(heap_idx)
    }

    pub(crate) fn build_error_promise(&mut self, message: String) -> Value {
        let reason_idx = self.heap.len();
        let mut props = FxHashMap::default();
        props.insert("message".into(), Value::String(message));
        self.heap.push(HeapValue::Object(JsObject {
            properties: props,
            prototype: None,
            extensible: true,
        }));
        let promise_idx = self.heap.len();
        self.heap.push(HeapValue::Promise(
            crate::objects::js_promise::JsPromise::rejected(Value::Object(reason_idx)),
        ));
        Value::Promise(promise_idx)
    }

    pub(crate) fn build_module_promise(&mut self, exports: FxHashMap<String, Value>) -> Value {
        let module_obj = self.build_module_object_from_exports(&exports);
        let promise_idx = self.heap.len();
        self.heap.push(HeapValue::Promise(
            crate::objects::js_promise::JsPromise::fulfilled(module_obj),
        ));
        Value::Promise(promise_idx)
    }

    pub(crate) fn exec_import_module(&mut self, source: &str) -> Result<Option<Value>> {
        match self.load_and_run_module(source) {
            Ok(Some(module_path)) => {
                let exports = self
                    .module_registry
                    .get(&module_path)
                    .cloned()
                    .unwrap_or_default();
                Ok(Some(self.build_module_object_from_exports(&exports)))
            }
            Ok(None) => Ok(Some(Value::Undefined)),
            Err(e) => Ok(Some(self.build_error_promise(e.message().to_string()))),
        }
    }

    pub(crate) fn exec_import_named(
        &mut self,
        source: &str,
        imported_name: &str,
        local_name: &str,
    ) -> Result<Value> {
        match self.load_and_run_module(source)? {
            Some(module_path) => {
                let exports = self
                    .module_registry
                    .get(&module_path)
                    .cloned()
                    .unwrap_or_default();
                let val = exports
                    .get(imported_name)
                    .cloned()
                    .unwrap_or(Value::Undefined);
                self.globals.insert(local_name.to_string(), val);
                Ok(Value::Undefined)
            }
            None => Err(Error::RuntimeError(format!(
                "Cannot find module '{}'",
                source
            ))),
        }
    }

    pub(crate) fn exec_import_default(&mut self, source: &str, local_name: &str) -> Result<Value> {
        match self.load_and_run_module(source)? {
            Some(module_path) => {
                let exports = self
                    .module_registry
                    .get(&module_path)
                    .cloned()
                    .unwrap_or_default();
                let val = if let Some(v) = exports.get("default") {
                    v.clone()
                } else if !exports.is_empty() {
                    self.build_module_object_from_exports(&exports)
                } else {
                    Value::Undefined
                };
                self.globals.insert(local_name.to_string(), val);
                Ok(Value::Undefined)
            }
            None => Err(Error::RuntimeError(format!(
                "Cannot find module '{}'",
                source
            ))),
        }
    }

    pub(crate) fn exec_import_all(&mut self, source: &str, local_name: &str) -> Result<Value> {
        match self.load_and_run_module(source)? {
            Some(module_path) => {
                let exports = self
                    .module_registry
                    .get(&module_path)
                    .cloned()
                    .unwrap_or_default();
                let module_obj = self.build_module_object_from_exports(&exports);
                self.globals.insert(local_name.to_string(), module_obj);
                Ok(Value::Undefined)
            }
            None => Err(Error::RuntimeError(format!(
                "Cannot find module '{}'",
                source
            ))),
        }
    }

    pub(crate) fn exec_native_import(&mut self, source: &str, local_name: &str) -> Result<Value> {
        match self.load_and_run_module(source)? {
            Some(module_path) => {
                let exports = self
                    .module_registry
                    .get(&module_path)
                    .cloned()
                    .unwrap_or_default();
                let val = exports
                    .get("default")
                    .cloned()
                    .unwrap_or_else(|| self.build_module_object_from_exports(&exports));
                self.globals.insert(local_name.to_string(), val);
                Ok(Value::Undefined)
            }
            None => Err(Error::RuntimeError(format!(
                "Cannot find module '{}'",
                source
            ))),
        }
    }

    pub(crate) fn exec_export_named(&mut self, names: &[String]) -> Result<()> {
        for name in names {
            if let Some(val) = self.globals.get(name) {
                self.module_exports.insert(name.clone(), val.clone());
            }
        }
        Ok(())
    }

    pub(crate) fn exec_export_default(&mut self) -> Result<()> {
        let val = self.stack.last().cloned().unwrap_or(Value::Undefined);
        self.module_exports.insert("default".to_string(), val);
        Ok(())
    }

    pub(crate) fn exec_store_module_export(&mut self, name: &str) -> Result<()> {
        if let Some(val) = self.globals.get(name) {
            self.module_exports.insert(name.to_string(), val.clone());
        }
        Ok(())
    }

    pub(crate) fn exec_reexport_all(&mut self, source: &str) -> Result<()> {
        match self.load_and_run_module(source)? {
            Some(module_path) => {
                let exports = self
                    .module_registry
                    .get(&module_path)
                    .cloned()
                    .unwrap_or_default();
                for (k, v) in &exports {
                    if k != "default" {
                        self.module_exports.insert(k.clone(), v.clone());
                    }
                }
                Ok(())
            }
            None => Err(Error::RuntimeError(format!(
                "Cannot find module '{}'",
                source
            ))),
        }
    }
}
