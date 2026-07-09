use crate::ffi::TailsValue;
use crate::TailsRuntime;

/// The standard signature for native function pointers in Tails-rs.
pub type NativeFunctionPtr =
    extern "C" fn(*mut std::ffi::c_void, TailsValue, *const TailsValue, i32) -> TailsValue;

/// A safe wrapper around a native function pointer.
///
/// Holds the raw function pointer along with a human-readable name
/// for diagnostics and debugging.
#[derive(Debug)]
pub struct SafeNativeFunction {
    ptr: NativeFunctionPtr,
    name: String,
}

impl SafeNativeFunction {
    /// Create a new SafeNativeFunction from a raw pointer.
    ///
    /// # Safety
    /// The function pointer must have the signature:
    /// `extern "C" fn(*mut c_void, TailsValue, *const TailsValue, i32) -> TailsValue`
    pub unsafe fn new(ptr: NativeFunctionPtr, name: &str) -> Self {
        Self {
            ptr,
            name: name.to_string(),
        }
    }

    /// Call the function with the given arguments.
    ///
    /// # Safety
    /// - `runtime` must be a valid pointer to a `TailsRuntime`.
    /// - `args` must contain values matching the native function's expected parameters.
    pub unsafe fn call(
        &self,
        runtime: *mut TailsRuntime,
        this: TailsValue,
        args: &[TailsValue],
    ) -> TailsValue {
        (self.ptr)(
            runtime as *mut std::ffi::c_void,
            this,
            args.as_ptr(),
            args.len() as i32,
        )
    }

    /// Get the name of the function.
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the raw function pointer.
    #[inline]
    pub fn as_ptr(&self) -> NativeFunctionPtr {
        self.ptr
    }
}

/// A generic wrapper for function pointers with type-safe accessors.
///
/// Stores a raw pointer along with a descriptive name.
#[derive(Debug)]
pub struct FunctionPointerWrapper<T> {
    ptr: *const T,
    name: String,
}

impl<T> FunctionPointerWrapper<T> {
    /// Create a new FunctionPointerWrapper.
    ///
    /// # Safety
    /// `ptr` must be a valid, properly aligned pointer to a `T`.
    pub unsafe fn new(ptr: *const T, name: &str) -> Self {
        Self {
            ptr,
            name: name.to_string(),
        }
    }

    /// Get a reference to the pointed-to function/value.
    ///
    /// # Safety
    /// The pointer must be valid and properly aligned.
    pub unsafe fn as_ref(&self) -> &T {
        // SAFETY: caller guarantees pointer validity.
        unsafe { &*self.ptr }
    }

    /// Get the name of the function.
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the raw pointer.
    #[inline]
    pub fn as_ptr(&self) -> *const T {
        self.ptr
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    extern "C" fn test_function(
        _runtime: *mut std::ffi::c_void,
        _this: TailsValue,
        _args: *const TailsValue,
        _argc: i32,
    ) -> TailsValue {
        TailsValue { tag: 0, data: 0 }
    }

    #[test]
    fn test_safe_native_function_creation() {
        let func = unsafe { SafeNativeFunction::new(test_function, "test") };
        assert_eq!(func.name(), "test");
        assert_eq!(func.as_ptr() as usize, test_function as *const () as usize);
    }

    #[test]
    fn test_function_pointer_wrapper() {
        let f: NativeFunctionPtr = test_function;
        let wrapper: FunctionPointerWrapper<NativeFunctionPtr> =
            unsafe { FunctionPointerWrapper::new(&f as *const NativeFunctionPtr, "test") };
        assert_eq!(wrapper.name(), "test");
        assert!(!wrapper.as_ptr().is_null());
    }

    #[test]
    fn test_safe_native_function_name() {
        let func = unsafe { SafeNativeFunction::new(test_function, "my_function") };
        assert_eq!(func.name(), "my_function");
    }

    #[test]
    fn test_function_pointer_wrapper_as_ref() {
        let f: NativeFunctionPtr = test_function;
        let wrapper: FunctionPointerWrapper<NativeFunctionPtr> =
            unsafe { FunctionPointerWrapper::new(&f as *const NativeFunctionPtr, "test") };
        let func_ref = unsafe { wrapper.as_ref() };
        assert_eq!(*func_ref as usize, test_function as *const () as usize);
    }
}
