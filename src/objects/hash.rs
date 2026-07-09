use std::hash::{Hash, Hasher};

use crate::objects::Value;

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Value::Undefined => 0u8.hash(state),
            Value::Null => 1u8.hash(state),
            Value::Boolean(b) => {
                2u8.hash(state);
                b.hash(state);
            }
            // Numbers share tag 3 and hash as f64 bits so Integer(5) and
            // Float(5.0) collide under SameValueZero (required for Map/Set).
            Value::Integer(i) => {
                3u8.hash(state);
                (*i as f64).to_bits().hash(state);
            }
            Value::Float(f) => {
                3u8.hash(state);
                let bits = if f.is_nan() {
                    f64::NAN.to_bits()
                } else if *f == 0.0 {
                    0.0f64.to_bits() // +0 and -0
                } else {
                    f.to_bits()
                };
                bits.hash(state);
            }
            Value::String(s) => {
                5u8.hash(state);
                s.as_ref().hash(state);
            }
            Value::Cons(c) => {
                5u8.hash(state);
                let h = c.hash_cons_string();
                h.hash(state);
            }
            Value::BigInt(i) => {
                6u8.hash(state);
                i.hash(state);
            }
            Value::Symbol(id) => {
                7u8.hash(state);
                id.hash(state);
            }
            Value::Function(i) => {
                8u8.hash(state);
                i.hash(state);
            }
            Value::NativeFunction(i) => {
                9u8.hash(state);
                i.hash(state);
            }
            Value::Object(i) => {
                10u8.hash(state);
                i.hash(state);
            }
            Value::Array(i) => {
                11u8.hash(state);
                i.hash(state);
            }
            Value::Promise(i) => {
                12u8.hash(state);
                i.hash(state);
            }
            Value::Proxy(i) => {
                13u8.hash(state);
                i.hash(state);
            }
            Value::Generator(i) => {
                14u8.hash(state);
                i.hash(state);
            }
            Value::TypedArray(i) => {
                15u8.hash(state);
                i.hash(state);
            }
            Value::Map(i) => {
                16u8.hash(state);
                i.hash(state);
            }
            Value::Set(i) => {
                17u8.hash(state);
                i.hash(state);
            }
            Value::WeakMap(i) => {
                18u8.hash(state);
                i.hash(state);
            }
            Value::WeakSet(i) => {
                19u8.hash(state);
                i.hash(state);
            }
            Value::Date(i) => {
                20u8.hash(state);
                i.hash(state);
            }
            Value::RegExp(i) => {
                21u8.hash(state);
                i.hash(state);
            }
            Value::Buffer(i) => {
                22u8.hash(state);
                i.hash(state);
            }
            Value::NativeObject(id) => {
                23u8.hash(state);
                id.hash(state);
            }
        }
    }
}
