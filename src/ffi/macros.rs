#[macro_export]
macro_rules! empty_tails_value {
    () => {
        $crate::ffi::TailsValue {
            tag: $crate::ffi::TailsValueType::Undefined as u32,
            data: 0,
        }
    };
}

/// Guard that returns `empty_tails_value!()` if either pointer is null.
#[macro_export]
macro_rules! null_guard {
    ($a:expr, $b:expr) => {
        if $a.is_null() || $b.is_null() {
            return $crate::empty_tails_value!();
        }
    };
}

/// Guard that returns `empty_tails_value!()` if the pointer is null.
#[macro_export]
macro_rules! null_guard_single {
    ($a:expr) => {
        if $a.is_null() {
            return $crate::empty_tails_value!();
        }
    };
}

/// Guard that returns nothing (for side-effect functions).
#[macro_export]
macro_rules! null_guard_void {
    ($a:expr, $b:expr) => {
        if $a.is_null() || $b.is_null() {
            return;
        }
    };
}

/// Guard that returns a fallback value if either pointer is null.
#[macro_export]
macro_rules! null_guard_with {
    ($a:expr, $b:expr, $fallback:expr) => {
        if $a.is_null() || $b.is_null() {
            return $fallback;
        }
    };
}

/// Guard for array operations: checks runtime null + array tag.
#[macro_export]
macro_rules! array_guard {
    ($runtime:expr, $array:expr) => {
        if $runtime.is_null() || $array.tag != $crate::ffi::TailsValueType::Array as u32 {
            return $crate::empty_tails_value!();
        }
    };
}

/// Guard for array operations returning i32.
#[macro_export]
macro_rules! array_guard_i32 {
    ($runtime:expr, $array:expr, $fallback:expr) => {
        if $runtime.is_null() || $array.tag != $crate::ffi::TailsValueType::Array as u32 {
            return $fallback;
        }
    };
}
