use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::{take_module_handle, NativeFn, NativeValue};

pub struct NativeLibrary {
    _library: libloading::Library,
    functions: HashMap<String, NativeFn>,
    constants: HashMap<String, NativeValue>,
}

// SAFETY: NativeLibrary is only used from the single runtime thread that owns
// the interpreter. Function pointers remain valid while `_library` is kept alive.
unsafe impl Send for NativeLibrary {}
unsafe impl Sync for NativeLibrary {}

impl NativeLibrary {
    pub fn load(path: &Path) -> Result<Self, String> {
        // Safety: `Library::new` is the OS dynamic linker; path is a real filesystem path.
        let library = unsafe {
            libloading::Library::new(path)
                .map_err(|e| format!("Failed to load native library '{}': {}", path.display(), e))?
        };

        let mut functions = HashMap::new();
        let constants = HashMap::new();

        // Try module-specific init first (tails_native_init_<name>), then fallback to generic
        type InitFn = fn() -> *mut crate::ModuleHandle;

        // Safety: InitFn must match the exported init symbol; symbols are generated
        // by tails_module macros with a known signature.
        let init_result = unsafe {
            let module_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            let base_name = module_stem.strip_prefix("lib").unwrap_or(module_stem);

            let init_name = format!("tails_native_init_{}\0", base_name.replace('-', "_"));
            if let Ok(init_fn) = library.get::<InitFn>(init_name.as_bytes()) {
                Some(*init_fn)
            } else {
                let init_name2 = format!("tails_native_init_{}\0", base_name);
                if let Ok(init_fn) = library.get::<InitFn>(init_name2.as_bytes()) {
                    Some(*init_fn)
                } else {
                    library
                        .get::<InitFn>(b"tails_native_init\0")
                        .ok()
                        .map(|s| *s)
                }
            }
        };

        let init_fn =
            init_result.ok_or_else(|| format!("No init function found in '{}'", path.display()))?;

        let handle = init_fn();
        if handle.is_null() {
            return Err("init function returned null".to_string());
        }

        // Safety: handle from tails_native_init, non-null, Box-allocated.
        let handle = unsafe { take_module_handle(handle) };
        for (func_name, func_ptr) in &handle.module.functions {
            functions.insert(func_name.clone(), *func_ptr);
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
