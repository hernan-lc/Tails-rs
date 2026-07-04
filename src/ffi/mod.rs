#![allow(clippy::not_unsafe_ptr_arg_deref)]

pub mod conversions;
#[macro_use]
pub mod macros;
pub mod native;
pub mod safe_string;
pub mod safe_wrappers;

use crate::objects::Value;
use crate::TailsRuntime;
use conversions::{tails_value_to_value, value_to_tails_value};
use safe_wrappers::SafeCStr;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

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
    Array = 6,
    Function = 7,
    Promise = 8,
    Proxy = 9,
    NativeFunction = 10,
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
        return empty_tails_value!();
    }
    let runtime = unsafe { &mut *runtime };
    let source = match unsafe { SafeCStr::new(source) }.to_str() {
        Some(s) => s,
        None => return empty_tails_value!(),
    };
    match runtime.eval(source) {
        Ok(value) => value_to_tails_value(value),
        Err(_) => empty_tails_value!(),
    }
}

#[no_mangle]
pub extern "C" fn tails_get_global(runtime: *mut TailsRuntime, name: *const c_char) -> TailsValue {
    if runtime.is_null() || name.is_null() {
        return empty_tails_value!();
    }
    let runtime = unsafe { &*runtime };
    let name = match unsafe { SafeCStr::new(name) }.to_str() {
        Some(s) => s,
        None => return empty_tails_value!(),
    };
    match runtime.get_global(name) {
        Some(value) => value_to_tails_value(value),
        None => empty_tails_value!(),
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
        let val = tails_value_to_value(value);
        runtime.set_global(name_str, val);
    }
}

#[no_mangle]
pub extern "C" fn tails_value_get_tag(value: TailsValue) -> u32 {
    value.tag
}

#[no_mangle]
pub extern "C" fn tails_is_type(value: TailsValue, tag: TailsValueType) -> bool {
    value.tag == tag as u32
}

#[no_mangle]
pub extern "C" fn tails_is_undefined(value: TailsValue) -> bool {
    tails_is_type(value, TailsValueType::Undefined)
}

#[no_mangle]
pub extern "C" fn tails_is_null(value: TailsValue) -> bool {
    tails_is_type(value, TailsValueType::Null)
}

#[no_mangle]
pub extern "C" fn tails_is_boolean(value: TailsValue) -> bool {
    tails_is_type(value, TailsValueType::Boolean)
}

#[no_mangle]
pub extern "C" fn tails_is_number(value: TailsValue) -> bool {
    tails_is_type(value, TailsValueType::Number)
}

#[no_mangle]
pub extern "C" fn tails_is_string(value: TailsValue) -> bool {
    tails_is_type(value, TailsValueType::String)
}

#[no_mangle]
pub extern "C" fn tails_is_object(value: TailsValue) -> bool {
    tails_is_type(value, TailsValueType::Object)
}

#[no_mangle]
pub extern "C" fn tails_is_array(value: TailsValue) -> bool {
    tails_is_type(value, TailsValueType::Array)
}

#[no_mangle]
pub extern "C" fn tails_is_function(value: TailsValue) -> bool {
    tails_is_type(value, TailsValueType::Function)
        || tails_is_type(value, TailsValueType::NativeFunction)
}

#[no_mangle]
pub extern "C" fn tails_get_boolean(value: TailsValue) -> bool {
    value.data != 0
}

#[no_mangle]
pub extern "C" fn tails_get_number(value: TailsValue) -> f64 {
    f64::from_bits(value.data)
}

#[no_mangle]
pub extern "C" fn tails_get_string(value: TailsValue) -> *const c_char {
    if value.tag != TailsValueType::String as u32 {
        return std::ptr::null();
    }

    let ptr = value.data as *const u8;
    if ptr.is_null() {
        return std::ptr::null();
    }

    unsafe {
        let mut len = 0;
        while *ptr.add(len) != 0 {
            len += 1;
        }
        let slice = std::slice::from_raw_parts(ptr, len);
        match CStr::from_bytes_with_nul(slice) {
            Ok(cstr) => cstr.as_ptr(),
            Err(_) => std::ptr::null(),
        }
    }
}

#[no_mangle]
pub extern "C" fn tails_string_new(runtime: *mut TailsRuntime, s: *const c_char) -> TailsValue {
    if runtime.is_null() || s.is_null() {
        return empty_tails_value!();
    }
    let cstr = unsafe { CStr::from_ptr(s) };
    match cstr.to_str() {
        Ok(s) => {
            let value = Value::String(s.to_string());
            value_to_tails_value(value)
        }
        Err(_) => empty_tails_value!(),
    }
}

#[no_mangle]
pub extern "C" fn tails_number_new(value: f64) -> TailsValue {
    TailsValue {
        tag: TailsValueType::Number as u32,
        data: value.to_bits(),
    }
}

#[no_mangle]
pub extern "C" fn tails_boolean_new(value: bool) -> TailsValue {
    TailsValue {
        tag: TailsValueType::Boolean as u32,
        data: if value { 1 } else { 0 },
    }
}

#[no_mangle]
pub extern "C" fn tails_null() -> TailsValue {
    TailsValue {
        tag: TailsValueType::Null as u32,
        data: 0,
    }
}

#[no_mangle]
pub extern "C" fn tails_undefined() -> TailsValue {
    TailsValue {
        tag: TailsValueType::Undefined as u32,
        data: 0,
    }
}

#[no_mangle]
pub extern "C" fn tails_object_new(runtime: *mut TailsRuntime) -> TailsValue {
    if runtime.is_null() {
        return empty_tails_value!();
    }
    let runtime = unsafe { &mut *runtime };
    let value = runtime.new_object();
    value_to_tails_value(value)
}

#[no_mangle]
pub extern "C" fn tails_object_get(
    runtime: *mut TailsRuntime,
    object: TailsValue,
    key: *const c_char,
) -> TailsValue {
    if runtime.is_null() || key.is_null() {
        return empty_tails_value!();
    }
    let runtime = unsafe { &mut *runtime };
    let key = unsafe { CStr::from_ptr(key) };
    match key.to_str() {
        Ok(key_str) => {
            let obj_value = tails_value_to_value(object);
            match runtime.get_property(&obj_value, key_str) {
                Some(value) => value_to_tails_value(value),
                None => empty_tails_value!(),
            }
        }
        Err(_) => empty_tails_value!(),
    }
}

#[no_mangle]
pub extern "C" fn tails_object_set(
    runtime: *mut TailsRuntime,
    object: TailsValue,
    key: *const c_char,
    value: TailsValue,
) {
    if runtime.is_null() || key.is_null() {
        return;
    }
    let runtime = unsafe { &mut *runtime };
    let key = unsafe { CStr::from_ptr(key) };
    if let Ok(key_str) = key.to_str() {
        let obj_value = tails_value_to_value(object);
        let val = tails_value_to_value(value);
        runtime.set_property(&obj_value, key_str, val);
    }
}

#[no_mangle]
pub extern "C" fn tails_array_new(runtime: *mut TailsRuntime) -> TailsValue {
    if runtime.is_null() {
        return empty_tails_value!();
    }
    let runtime = unsafe { &mut *runtime };
    let value = runtime.new_array();
    value_to_tails_value(value)
}

#[no_mangle]
pub extern "C" fn tails_array_length(runtime: *mut TailsRuntime, array: TailsValue) -> i32 {
    if runtime.is_null() || array.tag != TailsValueType::Array as u32 {
        return -1;
    }

    let runtime = unsafe { &*runtime };
    let arr_value = tails_value_to_value(array);
    runtime.get_array_length(&arr_value).unwrap_or(-1) as i32
}

#[no_mangle]
pub extern "C" fn tails_array_get(
    runtime: *mut TailsRuntime,
    array: TailsValue,
    index: i32,
) -> TailsValue {
    if runtime.is_null() || array.tag != TailsValueType::Array as u32 {
        return empty_tails_value!();
    }
    let runtime = unsafe { &*runtime };
    let arr_value = tails_value_to_value(array);
    match runtime.get_array_element(&arr_value, index as usize) {
        Some(value) => value_to_tails_value(value),
        None => empty_tails_value!(),
    }
}

#[no_mangle]
pub extern "C" fn tails_array_push(
    runtime: *mut TailsRuntime,
    array: TailsValue,
    value: TailsValue,
) -> i32 {
    if runtime.is_null() || array.tag != TailsValueType::Array as u32 {
        return -1;
    }
    let runtime = unsafe { &mut *runtime };
    let arr_value = tails_value_to_value(array);
    let val = tails_value_to_value(value);
    runtime.push_array_element(&arr_value, val);
    runtime.get_array_length(&arr_value).unwrap_or(0) as i32
}

#[no_mangle]
pub extern "C" fn tails_call(
    runtime: *mut TailsRuntime,
    func: TailsValue,
    this: TailsValue,
    args: *const TailsValue,
    args_len: i32,
) -> TailsValue {
    if runtime.is_null() {
        return empty_tails_value!();
    }
    let runtime = unsafe { &mut *runtime };
    let func_value = tails_value_to_value(func);
    let this_value = tails_value_to_value(this);
    let args = if args.is_null() || args_len <= 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args, args_len as usize) }
    };
    let values: Vec<Value> = args.iter().map(|v| tails_value_to_value(*v)).collect();
    match runtime.call_function(&func_value, &this_value, &values) {
        Ok(value) => value_to_tails_value(value),
        Err(_) => empty_tails_value!(),
    }
}

#[no_mangle]
pub extern "C" fn tails_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}
