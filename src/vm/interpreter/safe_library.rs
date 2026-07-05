use libloading::{Library, Symbol};
use std::ffi::CString;
use std::path::Path;
use std::sync::Arc;

/// A safe wrapper around a dynamically loaded library
///
/// Manages the library lifecycle with automatic cleanup.
/// Thread-safe: implements `Send` and `Sync` (delegated to `libloading::Library`).
pub struct SafeLibrary {
    library: Option<Library>,
    path: String,
}

impl SafeLibrary {
    /// Load a library from the given path
    #[must_use = "use the returned Result to check for loading errors"]
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let path_str = path.as_ref().to_string_lossy().to_string();

        let library = unsafe { Library::new(path.as_ref()) }
            .map_err(|e| format!("Failed to load library '{}': {}", path_str, e))?;

        Ok(Self {
            library: Some(library),
            path: path_str,
        })
    }

    /// Get a function symbol from the library
    ///
    /// # Safety
    /// The caller must ensure the function pointer `T` has the correct signature
    /// matching the actual symbol in the library.
    #[must_use = "use the returned Result to check for symbol lookup errors"]
    pub unsafe fn get_function<T>(&self, name: &str) -> Result<Symbol<'_, T>, String> {
        let library = self.library.as_ref().ok_or("Library not loaded")?;

        let c_name = CString::new(name).map_err(|e| format!("Invalid symbol name: {}", e))?;

        match library.get::<T>(c_name.as_bytes_with_nul()) {
            Ok(symbol) => Ok(symbol),
            Err(e) => Err(format!("Symbol '{}' not found: {}", name, e)),
        }
    }

    /// Get the path of the loaded library
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Check if the library is loaded
    pub fn is_loaded(&self) -> bool {
        self.library.is_some()
    }
}

/// A safe wrapper around a function pointer from a dynamic library
///
/// Keeps the library alive via `Arc` so the function pointer remains valid.
pub struct SafeFunction<T: 'static> {
    func: Symbol<'static, T>,
    _library: Arc<SafeLibrary>,
}

impl<T: 'static> SafeFunction<T> {
    /// Create a new SafeFunction from a library symbol
    ///
    /// # Safety
    /// The caller must ensure the function pointer `T` has the correct signature
    /// matching the actual symbol in the library.
    pub unsafe fn new(library: Arc<SafeLibrary>, name: &str) -> Result<Self, String> {
        let func = library.get_function::<T>(name)?;

        // SAFETY: The Arc<SafeLibrary> stored in this struct keeps the library loaded,
        // so the symbol pointer remains valid for 'static.
        let func = std::mem::transmute::<Symbol<'_, T>, Symbol<'static, T>>(func);

        Ok(Self {
            func,
            _library: library,
        })
    }

    /// Get a reference to the function pointer
    pub fn as_ptr(&self) -> &T {
        &self.func
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_safe_library_nonexistent() {
        let result = SafeLibrary::new("/nonexistent/library.so");
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_library_is_loaded() {
        let result = SafeLibrary::new("/nonexistent/library.so");
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_function_creation() {
        let library = SafeLibrary::new("/nonexistent/library.so");
        assert!(library.is_err());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_safe_library_load_and_get_function() {
        let lib_path = "libc.so.6";
        let lib = SafeLibrary::new(lib_path).expect("failed to load libc");
        assert!(lib.is_loaded());
        assert_eq!(lib.path(), lib_path);

        unsafe {
            let abs_fn: Symbol<'_, unsafe extern "C" fn(i32) -> i32> =
                lib.get_function("abs").expect("failed to get abs");
            let result = abs_fn(-42);
            assert_eq!(result, 42);
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_safe_function_wrapper() {
        let lib_path = "libc.so.6";
        let lib = Arc::new(SafeLibrary::new(lib_path).expect("failed to load libc"));

        unsafe {
            let abs_fn = SafeFunction::<unsafe extern "C" fn(i32) -> i32>::new(lib, "abs")
                .expect("failed to create SafeFunction");
            let result: i32 = (*abs_fn.as_ptr())(-10);
            assert_eq!(result, 10);
        }
    }
}
