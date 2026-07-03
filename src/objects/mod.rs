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

/// Phase 1.7: ConsString — a binary tree (rope) representation for
/// deferred string concatenation. Instead of allocating a fresh
/// `String` of `a.len() + b.len()` bytes on every `+`, we build a
/// lazy tree node. The actual flat `String` is only produced when the
/// value is consumed by an operation that requires contiguous bytes
/// (comparisons, display, `to_number`, etc.).
///
/// `total_len` is cached for O(1) `is_truthy` / `length` checks.
#[derive(Debug, Clone)]
pub struct ConsString {
    pub left: Box<Value>,
    pub right: Box<Value>,
    pub total_len: usize,
}

impl ConsString {
    pub fn new(left: Value, right: Value) -> Self {
        let total_len = Self::value_len(&left) + Self::value_len(&right);
        Self {
            left: Box::new(left),
            right: Box::new(right),
            total_len,
        }
    }

    fn value_len(v: &Value) -> usize {
        match v {
            Value::String(s) => s.len(),
            Value::Cons(c) => c.total_len,
            _ => 0,
        }
    }

    /// Flatten the tree into a single `String`. Uses a stack-based
    /// iterative approach to avoid stack overflow on deep trees.
    pub fn flatten(&self) -> String {
        let mut result = String::with_capacity(self.total_len);
        let mut stack: Vec<&Value> = vec![&self.right, &self.left];
        while let Some(node) = stack.pop() {
            match node {
                Value::String(s) => result.push_str(s),
                Value::Cons(c) => {
                    stack.push(&c.right);
                    stack.push(&c.left);
                }
                _ => {}
            }
        }
        result
    }

    /// Write the flattened string into an existing buffer without
    /// allocating a new String.
    pub fn flatten_into(&self, buf: &mut String) {
        let mut stack: Vec<&Value> = vec![&self.right, &self.left];
        while let Some(node) = stack.pop() {
            match node {
                Value::String(s) => buf.push_str(s),
                Value::Cons(c) => {
                    stack.push(&c.right);
                    stack.push(&c.left);
                }
                _ => {}
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Undefined,
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Cons(ConsString),
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

/// Flatten any `Value` into a `String`. For `Value::String` this is
/// a clone; for `Value::Cons` it walks the rope tree iteratively.
pub fn flatten_value(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Cons(c) => c.flatten(),
        _ => v.to_string(),
    }
}

/// Write the flattened representation of a string-like `Value` into
/// `buf`. For `Value::String` this is a `push_str`; for
/// `Value::Cons` it walks the rope tree iteratively.
pub fn flatten_value_into(v: &Value, buf: &mut String) {
    match v {
        Value::String(s) => buf.push_str(s),
        Value::Cons(c) => c.flatten_into(buf),
        _ => buf.push_str(&v.to_string()),
    }
}

/// O(1) string length for any string-like `Value`. Returns `None` for
/// non-string values.
pub fn value_str_len(v: &Value) -> Option<usize> {
    match v {
        Value::String(s) => Some(s.len()),
        Value::Cons(c) => Some(c.total_len),
        _ => None,
    }
}

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
            (Value::String(a), Value::Cons(c)) => a.as_str() == c.flatten().as_str(),
            (Value::Cons(c), Value::String(b)) => c.flatten().as_str() == b.as_str(),
            (Value::Cons(a), Value::Cons(b)) => a.flatten() == b.flatten(),
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
            Value::Cons(c) => {
                let flat = c.flatten();
                write!(f, "{}", flat)
            }
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
