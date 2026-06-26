use super::*;
use crate::errors::Result;
use crate::objects::Value;

impl Interpreter {
    pub fn execute_module(&mut self, module: &CompiledModule) -> Result<Value> {
        eprintln!("execute_module: starting, insns={}", module.instructions.len());
        let saved_module = self.current_module.take();
        self.current_module = Some(module.clone());
        let prev_exports = std::mem::take(&mut self.module_exports);
        let pre_keys: std::collections::HashSet<String> = self.globals.keys().cloned().collect();
        let result = self.execute(module);
        eprintln!("execute_module: execute returned, result={:?}", result);
        let post_keys: std::collections::HashSet<String> = self.globals.keys().cloned().collect();
        let export_keys: std::collections::HashSet<String> = self.module_exports.keys().cloned().collect();
        for key in post_keys.difference(&pre_keys) {
            if !export_keys.contains(key) {
                self.globals.remove(key);
            }
        }
        let exec_exports = std::mem::replace(&mut self.module_exports, prev_exports);
        for (k, v) in exec_exports {
            self.module_exports.insert(k, v);
        }
        self.current_module = saved_module;
        result
    }

    fn resolve_local_from_stack(&self, _name: &str) -> Option<usize> {
        None
    }

    fn load_and_run_module(&mut self, source: &str) -> Result<Option<String>> {
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
        self.module_registry.insert(module_path.clone(), HashMap::new());
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
        Err(crate::errors::Error::RuntimeError(format!("Module '{}' not found", source)))
    }
}
