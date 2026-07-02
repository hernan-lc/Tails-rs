pub mod js_array;
pub mod js_collections;
pub mod js_date;
pub mod js_promise;
pub mod js_proxy;
pub mod safe_typed_array;
use std::fmt;

/// Well-known symbol IDs (small numbers to distinguish from user symbols)
pub const SYMBOL_ITERATOR: u64 = 1;
pub const SYMBOL_TO_STRING_TAG: u64 = 2;
pub const SYMBOL_HAS_INSTANCE: u64 = 3;
pub const SYMBOL_TO_PRIMITIVE: u64 = 4;
pub const SYMBOL_SPECIES: u64 = 5;
pub const SYMBOL_UNSCOPABLES: u64 = 6;
pub const SYMBOL_ASYNC_ITERATOR: u64 = 7;
/// Starting ID for user-created symbols
pub const USER_SYMBOL_START: u64 = 1000;

#[derive(Debug, Clone)]
pub enum Value {
    Undefined,
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
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

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Undefined, Value::Undefined) => true,
            (Value::Null, Value::Null) => true,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Integer(a), Value::Float(b)) => *a as f64 == *b,
            (Value::Float(a), Value::Integer(b)) => *a == *b as f64,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::BigInt(a), Value::BigInt(b)) => a == b,
            (Value::Symbol(a), Value::Symbol(b)) => a == b,
            (Value::Function(a), Value::Function(b)) => a == b,
            (Value::NativeFunction(a), Value::NativeFunction(b)) => a == b,
            (Value::Object(a), Value::Object(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Promise(a), Value::Promise(b)) => a == b,
            (Value::Proxy(a), Value::Proxy(b)) => a == b,
            (Value::Generator(a), Value::Generator(b)) => a == b,
            (Value::TypedArray(a), Value::TypedArray(b)) => a == b,
            (Value::Map(a), Value::Map(b)) => a == b,
            (Value::Set(a), Value::Set(b)) => a == b,
            (Value::WeakMap(a), Value::WeakMap(b)) => a == b,
            (Value::WeakSet(a), Value::WeakSet(b)) => a == b,
            (Value::Date(a), Value::Date(b)) => a == b,
            (Value::RegExp(a), Value::RegExp(b)) => a == b,
            (Value::Buffer(a), Value::Buffer(b)) => a == b,
            (Value::NativeObject(a), Value::NativeObject(b)) => a == b,
            _ => false,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Undefined => write!(f, "undefined"),
            Value::Null => write!(f, "null"),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Integer(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::String(s) => write!(f, "{}", s),
            Value::BigInt(i) => write!(f, "{}n", i),
            Value::Symbol(id) => write!(f, "Symbol({})", id),
            Value::Function(_) => write!(f, "[Function]"),
            Value::NativeFunction(_) => write!(f, "[NativeFunction]"),
            Value::Object(_) => write!(f, "[Object]"),
            Value::Array(_) => write!(f, "[Array]"),
            Value::Promise(_) => write!(f, "[Promise]"),
            Value::Proxy(_) => write!(f, "[Proxy]"),
            Value::Generator(_) => write!(f, "[Generator]"),
            Value::TypedArray(_) => write!(f, "[TypedArray]"),
            Value::Map(_) => write!(f, "[Map]"),
            Value::Set(_) => write!(f, "[Set]"),
            Value::WeakMap(_) => write!(f, "[WeakMap]"),
            Value::WeakSet(_) => write!(f, "[WeakSet]"),
            Value::Date(_) => write!(f, "[Date]"),
            Value::RegExp(_) => write!(f, "[RegExp]"),
            Value::Buffer(_) => write!(f, "[Buffer]"),
            Value::NativeObject(id) => write!(f, "[NativeObject({})]", id.0),
        }
    }
}
