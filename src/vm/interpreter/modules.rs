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
            {
                self.globals.insert(key.clone(), saved_globals[key].clone());
            }
        }
        let result = self.execute(module);
        let module_globals = std::mem::replace(&mut self.globals, saved_globals);
        let exec_exports = std::mem::replace(&mut self.module_exports, prev_exports);
        for (k, v) in &exec_exports {
            self.module_exports.insert(k.clone(), v.clone());
        }
        for (k, v) in &exec_exports {
            if let Some(mv) = module_globals.get(k) {
                self.globals.insert(k.clone(), mv.clone());
            } else {
                self.globals.insert(k.clone(), v.clone());
            }
        }
        self.current_module = saved_module;
        result
    }

    pub(crate) fn load_and_run_module(&mut self, source: &str) -> Result<Option<String>> {
        if source.ends_with(".native") {
            let module_name = super::native_loader::extract_module_name(source);
            if self.native_loader.has_module(module_name) {
                let exports =
                    self.native_loader
                        .load_module(module_name, &mut self.heap, &mut self.gc)?;
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
            Err(_) => return Ok(None),
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
        if resolved.exists() && resolved.is_file() {
            return Ok(resolved.to_string_lossy().to_string());
        }
        for ext in &[".ts", ".js"] {
            let stem = resolved.with_extension("");
            let candidate = std::path::PathBuf::from(format!("{}{}", stem.to_string_lossy(), ext));
            if candidate.exists() {
                return Ok(candidate.to_string_lossy().to_string());
            }
        }
        if resolved.is_dir() {
            for name in &["index.ts", "index.js"] {
                let idx = resolved.join(name);
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
