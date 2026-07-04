#[macro_export]
macro_rules! empty_tails_value {
    () => {
        $crate::ffi::TailsValue {
            tag: $crate::ffi::TailsValueType::Undefined as u32,
            data: 0,
        }
    };
}
