use crate::errors::Result;
use rustc_hash::FxHashMap;
use std::path::{Path, PathBuf};

pub struct ModuleLoader {
    modules: FxHashMap<String, Vec<u8>>,
    base_path: PathBuf,
}

impl ModuleLoader {
    pub fn new(base_path: &Path) -> Self {
        Self {
            modules: FxHashMap::default(),
            base_path: base_path.to_path_buf(),
        }
    }

    pub fn load(&mut self, specifier: &str) -> Result<String> {
        if let Some(cached) = self.modules.get(specifier) {
            return String::from_utf8(cached.clone())
                .map_err(|_| crate::errors::Error::RuntimeError("Invalid UTF-8 in module".into()));
        }

        let path = self.resolve(specifier)?;
        let source = std::fs::read_to_string(&path).map_err(|e| {
            crate::errors::Error::RuntimeError(format!("Failed to load module: {}", e))
        })?;

        self.modules
            .insert(specifier.to_string(), source.as_bytes().to_vec());

        Ok(source)
    }

    pub fn resolve(&self, specifier: &str) -> Result<PathBuf> {
        let path = self.base_path.join(specifier);

        if path.exists() {
            return Ok(path);
        }

        let path_with_ext = self.base_path.join(format!("{}.ts", specifier));
        if path_with_ext.exists() {
            return Ok(path_with_ext);
        }

        let path_with_index = self.base_path.join(specifier).join("index.ts");
        if path_with_index.exists() {
            return Ok(path_with_index);
        }

        Err(crate::errors::Error::RuntimeError(format!(
            "Module '{}' not found",
            specifier
        )))
    }
}

impl Default for ModuleLoader {
    fn default() -> Self {
        Self::new(Path::new("."))
    }
}
