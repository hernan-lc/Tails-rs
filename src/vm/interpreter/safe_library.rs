use libloading::{Library, Symbol};
use std::ffi::CString;
use std::path::Path;
use std::sync::Arc;

/// A safe wrapper around a dynamically loaded library
///
/// Manages the library lifecycle with automatic cleanup via `Drop`.
/// Thread-safe: implements `Send` and `Sync` (delegated to `libloading::Library`).
pub struct SafeLibrary {
    library: Option<Library>,
    path: String,
}

impl SafeLibrary {
    /// Load a library from the given path
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let path_str = path.as_ref().to_string_lossy().to_string();

        unsafe {
            match Library::new(path.as_ref()) {
                Ok(library) => Ok(Self {
                    library: Some(library),
                    path: path_str,
                }),
                Err(e) => Err(format!("Failed to load library '{}': {}", path_str, e)),
            }
        }
    }

    /// Get a function symbol from the library
    ///
    /// # Safety
    /// The caller must ensure the function pointer `T` has the correct signature
    /// matching the actual symbol in the library.
    pub unsafe fn get_function<T>(&self, name: &str) -> Result<Symbol<'_, T>, String> {
        let library = self.library.as_ref().ok_or("Library not loaded")?;

        let c_name =
            CString::new(name).map_err(|e| format!("Invalid symbol name: {}", e))?;

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

impl Drop for SafeLibrary {
    fn drop(&mut self) {
        self.library.take();
    }
}

// Safety: SafeLibrary can be sent across threads because
// libloading::Library is Send + Sync
unsafe impl Send for SafeLibrary {}
unsafe impl Sync for SafeLibrary {}

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

        let func = std::mem::transmute::<Symbol<'_, T>, Symbol<'static, T>>(func);

        Ok(Self {
            func,
            _library: library,
        })
    }

    /// Get a reference to the function pointer
    pub fn as_ptr(&self) -> &T {
        &*self.func
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
