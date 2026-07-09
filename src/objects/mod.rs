pub mod display;
pub mod eq;
pub mod hash;
pub mod js_array;
pub mod js_collections;
pub mod js_date;
pub mod js_date_calendar;
pub mod js_promise;
pub mod js_proxy;
pub mod safe_typed_array;
pub mod strings;

use std::sync::Arc;

/// Shared JS string payload. Cloning is O(1) (atomic refcount).
pub type JsStr = Arc<str>;

#[derive(Debug, Clone)]
pub enum Value {
    Undefined,
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(JsStr),
    Cons(strings::ConsString),
    BigInt(i128),
    Symbol(u64),
    Function(usize),
    NativeFunction(usize),
    Object(usize),
    Array(usize),
    Promise(usize),
    Proxy(usize),
    Generator(usize),
    TypedArray(usize),
    Map(usize),
    Set(usize),
    WeakMap(usize),
    WeakSet(usize),
    Date(usize),
    RegExp(usize),
    Buffer(usize),
    NativeObject(NativeObjectId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NativeObjectId(pub u32);

impl Eq for Value {}

impl Value {
    /// Build a shared string value (preferred constructor).
    #[inline]
    pub fn string(s: impl AsRef<str>) -> Self {
        Value::String(Arc::from(s.as_ref()))
    }

    /// Move a `String` into a shared string value without copying bytes twice.
    #[inline]
    pub fn from_string(s: String) -> Self {
        Value::String(Arc::from(s))
    }

    /// Flatten this value into a `String`. For `Value::String` this copies
    /// the shared bytes; for `Value::Cons` it walks the rope tree.
    pub fn flatten(&self) -> String {
        match self {
            Value::String(s) => s.to_string(),
            Value::Cons(c) => c.flatten(),
            _ => self.to_string(),
        }
    }

    /// Write the flattened representation of this value into `buf`.
    pub fn flatten_into(&self, buf: &mut String) {
        match self {
            Value::String(s) => buf.push_str(s),
            Value::Cons(c) => c.flatten_into(buf),
            _ => buf.push_str(&self.to_string()),
        }
    }

    /// O(1) string length for string-like `Value`. Returns `None` for
    /// non-string values.
    pub fn str_len(&self) -> Option<usize> {
        match self {
            Value::String(s) => Some(s.len()),
            Value::Cons(c) => Some(c.total_len),
            _ => None,
        }
    }

    #[inline]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s.as_ref()),
            _ => None,
        }
    }
}

pub use strings::{
    ConsString, SYMBOL_ASYNC_ITERATOR, SYMBOL_HAS_INSTANCE, SYMBOL_ITERATOR, SYMBOL_SPECIES,
    SYMBOL_TO_PRIMITIVE, SYMBOL_TO_STRING_TAG, SYMBOL_UNSCOPABLES, USER_SYMBOL_START,
};

#[cfg(test)]
mod size_probe {
    #[test]
    fn print_value_sizes() {
        use super::*;
        use std::mem::size_of;
        eprintln!("Value={}", size_of::<Value>());
        eprintln!("ConsString={}", size_of::<ConsString>());
        eprintln!("JsStr={}", size_of::<JsStr>());
        eprintln!("i128={}", size_of::<i128>());
        assert!(size_of::<Value>() > 0);
    }
}
