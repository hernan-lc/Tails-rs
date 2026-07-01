use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::{NativeFn, NativeValue};

pub struct NativeLibrary {
    _library: libloading::Library,
    functions: HashMap<String, NativeFn>,
    constants: HashMap<String, NativeValue>,
}

unsafe impl Send for NativeLibrary {}
unsafe impl Sync for NativeLibrary {}

impl NativeLibrary {
    pub fn load(path: &Path) -> Result<Self, String> {
        let library = unsafe {
            libloading::Library::new(path)
                .map_err(|e| format!("Failed to load native library '{}': {}", path.display(), e))?
        };

        let mut functions = HashMap::new();
        let constants = HashMap::new();

        // Look for the init function
        unsafe {
            type InitFn = fn() -> *mut crate::ModuleHandle;
            let init_fn: libloading::Symbol<InitFn> = library
                .get(b"tails_native_init\0")
                .map_err(|e| format!("Failed to find tails_native_init: {}", e))?;

            let handle = init_fn();
            if handle.is_null() {
                return Err("tails_native_init returned null".to_string());
            }

            let handle = Box::from_raw(handle);
            for (func_name, func_ptr) in &handle.module.functions {
                functions.insert(func_name.clone(), *func_ptr);
            }
        }

        Ok(Self {
            _library: library,
            functions,
            constants,
        })
    }

    pub fn get_function(&self, name: &str) -> Option<NativeFn> {
        self.functions.get(name).copied()
    }

    pub fn get_constant(&self, name: &str) -> Option<NativeValue> {
        self.constants.get(name).copied()
    }

    pub fn get_all_exports(&self) -> HashMap<String, NativeValue> {
        let mut exports = HashMap::new();
        for (name, func) in &self.functions {
            exports.insert(
                name.clone(),
                NativeValue {
                    tag: 10,
                    data: *func as usize as u64,
                },
            );
        }
        for (name, value) in &self.constants {
            exports.insert(name.clone(), *value);
        }
        exports
    }
}

pub fn resolve_native_path(source: &str) -> Option<PathBuf> {
    let name = extract_module_name(source);

    // Try ./dist/<name>.{so,dylib,dll}
    let dist_dir = std::env::current_dir().ok()?.join("dist");
    let lib_path = find_library_in_dir(&dist_dir, &name);

    if let Some(path) = lib_path {
        return Some(path);
    }

    // Try current directory
    if let Some(path) = find_library_in_dir(std::path::Path::new("."), &name) {
        return Some(path);
    }

    // Try the source path directly
    let source_path = Path::new(source);
    if source_path.exists() {
        return Some(source_path.to_path_buf());
    }

    None
}

fn find_library_in_dir(dir: &Path, name: &str) -> Option<PathBuf> {
    let extensions = if cfg!(target_os = "windows") {
        vec!["dll"]
    } else if cfg!(target_os = "macos") {
        vec!["dylib"]
    } else {
        vec!["so"]
    };

    for ext in &extensions {
        let path = dir.join(format!("{}.{}", name, ext));
        if path.exists() {
            return Some(path);
        }
        // Also try lib prefix on Unix
        if *ext != "dll" {
            let path = dir.join(format!("lib{}.{}", name, ext));
            if path.exists() {
                return Some(path);
            }
        }
    }

    None
}

pub fn extract_module_name(source: &str) -> String {
    let path = std::path::Path::new(source);
    let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or(source);

    // Handle .native extension
    let name = file_stem.strip_suffix(".native").unwrap_or(file_stem);

    // Get the last component if it contains /
    if let Some(pos) = name.rfind('/') {
        name[pos + 1..].to_string()
    } else {
        name.to_string()
    }
}

pub fn load_native_module(source: &str) -> Result<NativeLibrary, String> {
    let path = resolve_native_path(source)
        .ok_or_else(|| format!("Cannot find native library for '{}'", source))?;
    NativeLibrary::load(&path)
}
