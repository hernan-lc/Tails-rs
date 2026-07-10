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
        let mut arr = Vec::with_capacity(self.len());
        for item in self {
            let nv = item.to_native_value()?;
            let ov = OwnedValue::from_native_value(nv)?;
            arr.push(ov);
        }
        let sv = OwnedValue::Array(Box::new(arr));
        Ok(crate::store_handle(sv))
    }
}

impl<T: FromNativeValue> FromNativeValue for Vec<T> {
    fn from_native_value(val: NativeValue) -> Result<Self, String> {
        match val.tag {
            crate::TAG_ARRAY | crate::TAG_OBJECT => {
                if val.data != 0 {
                    if let Some(simd_json::OwnedValue::Array(arr)) = crate::get_handle(val.data) {
                        let mut res = Vec::with_capacity(arr.len());
                        for item in arr.iter() {
                            let nv = item.to_native_value()?;
                            res.push(T::from_native_value(nv)?);
                        }
                        Ok(res)
                    } else {
                        Ok(Vec::new())
                    }
                } else {
                    Ok(Vec::new())
                }
            }
            _ => Ok(Vec::new()),
        }
    }
}

impl<T: ToNativeValue> ToNativeValue for HashMap<String, T> {
    fn to_native_value(&self) -> Result<NativeValue, String> {
        let mut obj = simd_json::value::owned::Object::new();
        for (k, v) in self {
            let nv = v.to_native_value()?;
            let ov = OwnedValue::from_native_value(nv)?;
            obj.insert(k.clone(), ov);
        }
        let sv = OwnedValue::Object(Box::new(obj));
        Ok(crate::store_handle(sv))
    }
}

impl<T: FromNativeValue> FromNativeValue for HashMap<String, T> {
    fn from_native_value(val: NativeValue) -> Result<Self, String> {
        match val.tag {
            crate::TAG_OBJECT | crate::TAG_ARRAY => {
                if val.data != 0 {
                    if let Some(simd_json::OwnedValue::Object(obj)) = crate::get_handle(val.data) {
                        let mut res = HashMap::with_capacity(obj.len());
                        for (k, v) in obj.iter() {
                            let nv = v.to_native_value()?;
                            res.insert(k.clone(), T::from_native_value(nv)?);
                        }
                        Ok(res)
                    } else {
                        Ok(HashMap::new())
                    }
                } else {
                    Ok(HashMap::new())
                }
            }
            _ => Ok(HashMap::new()),
        }
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

use simd_json::OwnedValue;

// ============================================================================
// simd_json::OwnedValue — handle-based zero-copy passing
// ============================================================================

impl ToNativeValue for OwnedValue {
    fn to_native_value(&self) -> Result<NativeValue, String> {
        match self {
            OwnedValue::Static(node) => match node {
                simd_json::StaticNode::Null => Ok(crate::null()),
                simd_json::StaticNode::Bool(b) => Ok(crate::boolean(*b)),
                simd_json::StaticNode::I64(i) => Ok(crate::integer(*i)),
                simd_json::StaticNode::U64(u) => Ok(crate::integer(*u as i64)),
                simd_json::StaticNode::F64(f) => Ok(crate::number(*f)),
            },
            OwnedValue::String(s) => Ok(crate::string(s)),
            OwnedValue::Array(_) | OwnedValue::Object(_) => Ok(crate::store_handle(self.clone())),
        }
    }
}

impl FromNativeValue for OwnedValue {
    fn from_native_value(val: NativeValue) -> Result<Self, String> {
        match val.tag {
            crate::TAG_NULL => Ok(OwnedValue::Static(simd_json::StaticNode::Null)),
            crate::TAG_UNDEFINED => Ok(OwnedValue::Static(simd_json::StaticNode::Null)),
            crate::TAG_BOOLEAN => Ok(OwnedValue::Static(simd_json::StaticNode::Bool(
                crate::get_boolean(val),
            ))),
            crate::TAG_NUMBER => {
                let n = crate::get_number(val);
                if n.fract() == 0.0 && n.abs() <= i64::MAX as f64 {
                    Ok(OwnedValue::Static(simd_json::StaticNode::I64(n as i64)))
                } else {
                    Ok(OwnedValue::Static(simd_json::StaticNode::F64(n)))
                }
            }
            crate::TAG_STRING => {
                let s = crate::get_string(val);
                // Try JSON parsing to handle serialized arrays/objects from FFI
                let mut s_bytes = s.into_bytes();
                if let Ok(parsed) = simd_json::from_slice::<OwnedValue>(&mut s_bytes) {
                    match parsed {
                        OwnedValue::Array(_) | OwnedValue::Object(_) => Ok(parsed),
                        _ => Ok(OwnedValue::String(
                            String::from_utf8_lossy(&s_bytes).to_string(),
                        )),
                    }
                } else {
                    Ok(OwnedValue::String(
                        String::from_utf8_lossy(&s_bytes).to_string(),
                    ))
                }
            }
            crate::TAG_OBJECT | crate::TAG_ARRAY => {
                if val.data != 0 {
                    match crate::get_handle(val.data) {
                        Some(v) => Ok(v),
                        None => {
                            eprintln!(
                                "[abi-debug] handle {} not found, registry size={}",
                                val.data,
                                crate::resolve_shared_registry().lock().unwrap().len()
                            );
                            Ok(OwnedValue::Static(simd_json::StaticNode::Null))
                        }
                    }
                } else {
                    if val.tag == crate::TAG_ARRAY {
                        Ok(OwnedValue::Array(Box::new(Vec::new())))
                    } else {
                        Ok(OwnedValue::Object(Box::new(
                            simd_json::value::owned::Object::new(),
                        )))
                    }
                }
            }
            _ => Ok(OwnedValue::Static(simd_json::StaticNode::Null)),
        }
    }
}

// ============================================================================
// serde_json::Value — handle-based zero-copy passing
// ============================================================================

fn simd_to_serde(v: &OwnedValue) -> serde_json::Value {
    match v {
        OwnedValue::Static(node) => match node {
            simd_json::StaticNode::Null => serde_json::Value::Null,
            simd_json::StaticNode::Bool(b) => serde_json::Value::Bool(*b),
            simd_json::StaticNode::I64(i) => serde_json::Value::Number((*i).into()),
            simd_json::StaticNode::U64(u) => serde_json::Value::Number((*u).into()),
            simd_json::StaticNode::F64(f) => {
                serde_json::Value::Number(serde_json::Number::from_f64(*f).unwrap_or(0.into()))
            }
        },
        OwnedValue::String(s) => serde_json::Value::String(s.clone()),
        OwnedValue::Array(arr) => serde_json::Value::Array(arr.iter().map(simd_to_serde).collect()),
        OwnedValue::Object(obj) => {
            let mut map = serde_json::Map::new();
            for (k, v) in obj.iter() {
                map.insert(k.clone(), simd_to_serde(v));
            }
            serde_json::Value::Object(map)
        }
    }
}

fn serde_to_simd(v: &serde_json::Value) -> OwnedValue {
    match v {
        serde_json::Value::Null => OwnedValue::Static(simd_json::StaticNode::Null),
        serde_json::Value::Bool(b) => OwnedValue::Static(simd_json::StaticNode::Bool(*b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                OwnedValue::Static(simd_json::StaticNode::I64(i))
            } else if let Some(u) = n.as_u64() {
                OwnedValue::Static(simd_json::StaticNode::U64(u))
            } else if let Some(f) = n.as_f64() {
                OwnedValue::Static(simd_json::StaticNode::F64(f))
            } else {
                OwnedValue::Static(simd_json::StaticNode::Null)
            }
        }
        serde_json::Value::String(s) => OwnedValue::String(s.clone()),
        serde_json::Value::Array(arr) => {
            OwnedValue::Array(Box::new(arr.iter().map(serde_to_simd).collect()))
        }
        serde_json::Value::Object(obj) => {
            let mut map = simd_json::value::owned::Object::new();
            for (k, v) in obj {
                map.insert(k.clone(), serde_to_simd(v));
            }
            OwnedValue::Object(Box::new(map))
        }
    }
}

impl ToNativeValue for serde_json::Value {
    fn to_native_value(&self) -> Result<NativeValue, String> {
        let owned = serde_to_simd(self);
        owned.to_native_value()
    }
}

impl FromNativeValue for serde_json::Value {
    fn from_native_value(val: NativeValue) -> Result<Self, String> {
        let owned = OwnedValue::from_native_value(val)?;
        Ok(simd_to_serde(&owned))
    }
}
