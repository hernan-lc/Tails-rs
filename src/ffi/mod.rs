pub mod native;

use std::os::raw::c_char;
use std::ffi::CStr;
use crate::TailsRuntime;
use crate::objects::Value;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct TailsValue {
    pub tag: u32,
    pub data: u64,
}

#[repr(C)]
pub enum TailsValueType {
    Undefined = 0,
    Null = 1,
    Boolean = 2,
    Number = 3,
    String = 4,
    Object = 5,
}

#[no_mangle]
pub extern "C" fn tails_runtime_new() -> *mut TailsRuntime {
    let runtime = TailsRuntime::default();
    Box::into_raw(Box::new(runtime))
}

#[no_mangle]
pub extern "C" fn tails_runtime_free(runtime: *mut TailsRuntime) {
    if !runtime.is_null() {
        unsafe {
            let _ = Box::from_raw(runtime);
        }
    }
}

#[no_mangle]
pub extern "C" fn tails_eval(runtime: *mut TailsRuntime, source: *const c_char) -> TailsValue {
    if runtime.is_null() || source.is_null() {
        return TailsValue {
            tag: TailsValueType::Undefined as u32,
            data: 0,
        };
    }
    
    let runtime = unsafe { &mut *runtime };
    let source = unsafe { CStr::from_ptr(source) };
    
    match source.to_str() {
        Ok(source_str) => match runtime.eval(source_str) {
            Ok(value) => value_to_tails_value(value),
            Err(_) => TailsValue {
                tag: TailsValueType::Undefined as u32,
                data: 0,
            },
        },
        Err(_) => TailsValue {
            tag: TailsValueType::Undefined as u32,
            data: 0,
        },
    }
}

#[no_mangle]
pub extern "C" fn tails_get_global(runtime: *mut TailsRuntime, name: *const c_char) -> TailsValue {
    if runtime.is_null() || name.is_null() {
        return TailsValue {
            tag: TailsValueType::Undefined as u32,
            data: 0,
        };
    }
    
    let runtime = unsafe { &*runtime };
    let name = unsafe { CStr::from_ptr(name) };
    
    match name.to_str() {
        Ok(name_str) => match runtime.get_global(name_str) {
            Some(value) => value_to_tails_value(value),
            None => TailsValue {
                tag: TailsValueType::Undefined as u32,
                data: 0,
            },
        },
        Err(_) => TailsValue {
            tag: TailsValueType::Undefined as u32,
            data: 0,
        },
    }
}

#[no_mangle]
pub extern "C" fn tails_set_global(
    runtime: *mut TailsRuntime,
    name: *const c_char,
    value: TailsValue,
) {
    if runtime.is_null() || name.is_null() {
        return;
    }
    
    let runtime = unsafe { &mut *runtime };
    let name = unsafe { CStr::from_ptr(name) };
    
    if let Ok(name_str) = name.to_str() {
        let value = tails_value_to_value(value);
        runtime.set_global(name_str, value);
    }
}

fn value_to_tails_value(value: Value) -> TailsValue {
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
            data: n as u64,
        },
        Value::Float(n) => TailsValue {
            tag: TailsValueType::Number as u32,
            data: n.to_bits(),
        },
        Value::String(s) => {
            let ptr = s.as_ptr() as u64;
            TailsValue {
                tag: TailsValueType::String as u32,
                data: ptr,
            }
        }
        Value::BigInt(_) | Value::Function(_) | Value::NativeFunction(_) | Value::Object(_) | Value::Array(_) | Value::Promise(_) => {
            TailsValue {
                tag: TailsValueType::Object as u32,
                data: 0,
            }
        }
    }
}

fn tails_value_to_value(value: TailsValue) -> Value {
    match value.tag {
        0 => Value::Undefined,
        1 => Value::Null,
        2 => Value::Boolean(value.data != 0),
        3 => Value::Float(f64::from_bits(value.data)),
        4 => {
            // For simplicity, we'll return an empty string
            // In a real implementation, you'd need proper memory management
            Value::String(String::new())
        }
        _ => Value::Undefined,
    }
}
