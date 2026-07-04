use std::fmt;

use crate::objects::Value;

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Undefined => write!(f, "undefined"),
            Value::Null => write!(f, "null"),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Integer(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::String(s) => write!(f, "{}", s),
            Value::Cons(c) => write!(f, "{}", c.flatten()),
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
