use crate::objects::Value;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use super::TailsValue;
use super::TailsValueType;

pub fn value_to_tails_value(value: Value) -> TailsValue {
    match value {
        Value::Undefined => TailsValue {
            tag: TailsValueType::Undefined as u32,
            data: 0,
        },
        Value::Null => TailsValue {
            tag: TailsValueType::Null as u32,
            data: 0,
        },
        Value::Boolean(b) => TailsValue {
            tag: TailsValueType::Boolean as u32,
            data: if b { 1 } else { 0 },
        },
        Value::Integer(n) => TailsValue {
            tag: TailsValueType::Number as u32,
            data: n as f64 as u64,
        },
        Value::Float(n) => TailsValue {
            tag: TailsValueType::Number as u32,
            data: n.to_bits(),
        },
        Value::String(s) => {
            let c_string = match CString::new(s) {
                Ok(cs) => cs,
                Err(_) => {
                    return TailsValue {
                        tag: TailsValueType::Undefined as u32,
                        data: 0,
                    }
                }
            };
            let ptr = c_string.into_raw() as u64;
            TailsValue {
                tag: TailsValueType::String as u32,
                data: ptr,
            }
        }
        Value::Cons(c) => {
            let flat = c.flatten();
            let c_string = match CString::new(flat) {
                Ok(cs) => cs,
                Err(_) => {
                    return TailsValue {
                        tag: TailsValueType::Undefined as u32,
                        data: 0,
                    }
                }
            };
            let ptr = c_string.into_raw() as u64;
            TailsValue {
                tag: TailsValueType::String as u32,
                data: ptr,
            }
        }
        Value::BigInt(_) => TailsValue {
            tag: TailsValueType::Number as u32,
            data: 0,
        },
        Value::Function(_) => TailsValue {
            tag: TailsValueType::Function as u32,
            data: 0,
        },
        Value::NativeFunction(_) => TailsValue {
            tag: TailsValueType::NativeFunction as u32,
            data: 0,
        },
        Value::Object(_) => TailsValue {
            tag: TailsValueType::Object as u32,
            data: 0,
        },
        Value::Array(_) => TailsValue {
            tag: TailsValueType::Array as u32,
            data: 0,
        },
        Value::Promise(_) => TailsValue {
            tag: TailsValueType::Promise as u32,
            data: 0,
        },
        Value::Proxy(_) => TailsValue {
            tag: TailsValueType::Proxy as u32,
            data: 0,
        },
        Value::Generator(_) => TailsValue {
            tag: TailsValueType::Function as u32,
            data: 0,
        },
        Value::TypedArray(_) => TailsValue {
            tag: TailsValueType::Object as u32,
            data: 0,
        },
        Value::Map(_) => TailsValue {
            tag: TailsValueType::Object as u32,
            data: 0,
        },
        Value::Set(_) => TailsValue {
            tag: TailsValueType::Object as u32,
            data: 0,
        },
        Value::WeakMap(_) => TailsValue {
            tag: TailsValueType::Object as u32,
            data: 0,
        },
        Value::WeakSet(_) => TailsValue {
            tag: TailsValueType::Object as u32,
            data: 0,
        },
        Value::Symbol(_) => TailsValue {
            tag: TailsValueType::Object as u32,
            data: 0,
        },
        Value::Date(_) | Value::RegExp(_) | Value::Buffer(_) | Value::NativeObject(_) => {
            TailsValue {
                tag: TailsValueType::Undefined as u32,
                data: 0,
            }
        }
    }
}

pub fn tails_value_to_value(value: TailsValue) -> Value {
    match value.tag {
        0 => Value::Undefined,
        1 => Value::Null,
        2 => Value::Boolean(value.data != 0),
        3 => Value::Float(f64::from_bits(value.data)),
        4 => {
            if value.data == 0 {
                Value::String(String::new())
            } else {
                let ptr = value.data as *const c_char;
                unsafe {
                    match CStr::from_ptr(ptr).to_str() {
                        Ok(s) => Value::String(s.to_string()),
                        Err(_) => Value::String(String::new()),
                    }
                }
            }
        }
        _ => Value::Undefined,
    }
}
