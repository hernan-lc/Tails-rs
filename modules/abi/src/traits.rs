use std::collections::HashMap;

use crate::{NativeValue, TAG_NULL, TAG_UNDEFINED};

pub trait ToNativeValue {
    fn to_native_value(&self) -> Result<NativeValue, String>;
}

pub trait FromNativeValue: Sized {
    fn from_native_value(val: NativeValue) -> Result<Self, String>;
}

impl ToNativeValue for NativeValue {
    fn to_native_value(&self) -> Result<NativeValue, String> {
        Ok(*self)
    }
}

impl FromNativeValue for NativeValue {
    fn from_native_value(val: NativeValue) -> Result<Self, String> {
        Ok(val)
    }
}

impl ToNativeValue for String {
    fn to_native_value(&self) -> Result<NativeValue, String> {
        Ok(crate::string(self))
    }
}

impl FromNativeValue for String {
    fn from_native_value(val: NativeValue) -> Result<Self, String> {
        Ok(crate::get_string(val))
    }
}

impl ToNativeValue for &str {
    fn to_native_value(&self) -> Result<NativeValue, String> {
        Ok(crate::string(self))
    }
}

impl ToNativeValue for f64 {
    fn to_native_value(&self) -> Result<NativeValue, String> {
        Ok(crate::number(*self))
    }
}

impl FromNativeValue for f64 {
    fn from_native_value(val: NativeValue) -> Result<Self, String> {
        Ok(crate::get_number(val))
    }
}

impl ToNativeValue for f32 {
    fn to_native_value(&self) -> Result<NativeValue, String> {
        Ok(crate::number(*self as f64))
    }
}

impl FromNativeValue for f32 {
    fn from_native_value(val: NativeValue) -> Result<Self, String> {
        Ok(crate::get_number(val) as f32)
    }
}

impl ToNativeValue for i64 {
    fn to_native_value(&self) -> Result<NativeValue, String> {
        Ok(crate::integer(*self))
    }
}

impl FromNativeValue for i64 {
    fn from_native_value(val: NativeValue) -> Result<Self, String> {
        Ok(crate::get_integer(val))
    }
}

impl ToNativeValue for i32 {
    fn to_native_value(&self) -> Result<NativeValue, String> {
        Ok(crate::integer(*self as i64))
    }
}

impl FromNativeValue for i32 {
    fn from_native_value(val: NativeValue) -> Result<Self, String> {
        Ok(crate::get_integer(val) as i32)
    }
}

impl ToNativeValue for u64 {
    fn to_native_value(&self) -> Result<NativeValue, String> {
        Ok(crate::integer(*self as i64))
    }
}

impl FromNativeValue for u64 {
    fn from_native_value(val: NativeValue) -> Result<Self, String> {
        Ok(crate::get_integer(val) as u64)
    }
}

impl ToNativeValue for u32 {
    fn to_native_value(&self) -> Result<NativeValue, String> {
        Ok(crate::integer(*self as i64))
    }
}

impl FromNativeValue for u32 {
    fn from_native_value(val: NativeValue) -> Result<Self, String> {
        Ok(crate::get_integer(val) as u32)
    }
}

impl ToNativeValue for i16 {
    fn to_native_value(&self) -> Result<NativeValue, String> {
        Ok(crate::integer(*self as i64))
    }
}

impl FromNativeValue for i16 {
    fn from_native_value(val: NativeValue) -> Result<Self, String> {
        Ok(crate::get_integer(val) as i16)
    }
}

impl ToNativeValue for u16 {
    fn to_native_value(&self) -> Result<NativeValue, String> {
        Ok(crate::integer(*self as i64))
    }
}

impl FromNativeValue for u16 {
    fn from_native_value(val: NativeValue) -> Result<Self, String> {
        Ok(crate::get_integer(val) as u16)
    }
}

impl ToNativeValue for i8 {
    fn to_native_value(&self) -> Result<NativeValue, String> {
        Ok(crate::integer(*self as i64))
    }
}

impl FromNativeValue for i8 {
    fn from_native_value(val: NativeValue) -> Result<Self, String> {
        Ok(crate::get_integer(val) as i8)
    }
}

impl ToNativeValue for u8 {
    fn to_native_value(&self) -> Result<NativeValue, String> {
        Ok(crate::integer(*self as i64))
    }
}

impl FromNativeValue for u8 {
    fn from_native_value(val: NativeValue) -> Result<Self, String> {
        Ok(crate::get_integer(val) as u8)
    }
}

impl ToNativeValue for bool {
    fn to_native_value(&self) -> Result<NativeValue, String> {
        Ok(crate::boolean(*self))
    }
}

impl FromNativeValue for bool {
    fn from_native_value(val: NativeValue) -> Result<Self, String> {
        Ok(crate::get_boolean(val))
    }
}

impl<T: ToNativeValue> ToNativeValue for Option<T> {
    fn to_native_value(&self) -> Result<NativeValue, String> {
        match self {
            Some(v) => v.to_native_value(),
            None => Ok(crate::null()),
        }
    }
}

impl<T: FromNativeValue> FromNativeValue for Option<T> {
    fn from_native_value(val: NativeValue) -> Result<Self, String> {
        if val.tag == TAG_NULL || val.tag == TAG_UNDEFINED {
            Ok(None)
        } else {
            Ok(Some(T::from_native_value(val)?))
        }
    }
}

impl<T: ToNativeValue> ToNativeValue for Vec<T> {
    fn to_native_value(&self) -> Result<NativeValue, String> {
        Ok(crate::array_new())
    }
}

impl<T: FromNativeValue> FromNativeValue for Vec<T> {
    fn from_native_value(_val: NativeValue) -> Result<Self, String> {
        Ok(Vec::new())
    }
}

impl<T: ToNativeValue> ToNativeValue for HashMap<String, T> {
    fn to_native_value(&self) -> Result<NativeValue, String> {
        Ok(crate::object_new())
    }
}

impl<T: FromNativeValue> FromNativeValue for HashMap<String, T> {
    fn from_native_value(_val: NativeValue) -> Result<Self, String> {
        Ok(HashMap::new())
    }
}

impl ToNativeValue for () {
    fn to_native_value(&self) -> Result<NativeValue, String> {
        Ok(crate::undefined())
    }
}

impl FromNativeValue for () {
    fn from_native_value(_val: NativeValue) -> Result<Self, String> {
        Ok(())
    }
}

impl<T: ToNativeValue, E: std::fmt::Display> ToNativeValue for Result<T, E> {
    fn to_native_value(&self) -> Result<NativeValue, String> {
        match self {
            Ok(v) => v.to_native_value(),
            Err(e) => Err(e.to_string()),
        }
    }
}
