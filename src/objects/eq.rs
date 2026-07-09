use crate::objects::Value;

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
            (Value::String(a), Value::Cons(c)) => c.eq_flat(a.as_ref()),
            (Value::Cons(c), Value::String(b)) => c.eq_flat(b.as_ref()),
            (Value::Cons(a), Value::Cons(b)) => a.eq_cons(b),
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
