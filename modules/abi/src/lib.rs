use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::Mutex;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct NativeValue {
    pub tag: u32,
    pub data: u64,
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct NativeString {
    pub ptr: *const u8,
    pub len: usize,
}

impl NativeString {
    pub fn to_string_lossy(&self) -> String {
        if self.ptr.is_null() || self.len == 0 {
            return String::new();
        }
        unsafe {
            let slice = std::slice::from_raw_parts(self.ptr, self.len);
            String::from_utf8_lossy(slice).to_string()
        }
    }
}

pub const TAG_UNDEFINED: u32 = 0;
pub const TAG_NULL: u32 = 1;
pub const TAG_BOOLEAN: u32 = 2;
pub const TAG_NUMBER: u32 = 3;
pub const TAG_STRING: u32 = 4;
pub const TAG_OBJECT: u32 = 5;
pub const TAG_ARRAY: u32 = 6;

pub type NativeFn = extern "C" fn(
    interp: *mut std::ffi::c_void,
    this: NativeValue,
    args: *const NativeValue,
    argc: i32,
) -> NativeValue;

pub struct NativeModule {
    pub name: String,
    pub functions: HashMap<String, NativeFn>,
}

impl NativeModule {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            functions: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: &str, func: NativeFn) {
        self.functions.insert(name.to_string(), func);
    }
}

pub struct ModuleHandle {
    pub module: NativeModule,
}

impl ModuleHandle {
    pub fn new(module: NativeModule) -> Self {
        Self { module }
    }
}

pub fn undefined() -> NativeValue {
    NativeValue {
        tag: TAG_UNDEFINED,
        data: 0,
    }
}

pub fn null() -> NativeValue {
    NativeValue {
        tag: TAG_NULL,
        data: 0,
    }
}

pub fn boolean(val: bool) -> NativeValue {
    NativeValue {
        tag: TAG_BOOLEAN,
        data: if val { 1 } else { 0 },
    }
}

pub fn number(val: f64) -> NativeValue {
    NativeValue {
        tag: TAG_NUMBER,
        data: val.to_bits(),
    }
}

pub fn integer(val: i64) -> NativeValue {
    NativeValue {
        tag: TAG_NUMBER,
        data: val as f64 as u64,
    }
}

pub fn string(s: &str) -> NativeValue {
    match CString::new(s) {
        Ok(cs) => NativeValue {
            tag: TAG_STRING,
            data: cs.into_raw() as u64,
        },
        Err(_) => undefined(),
    }
}

pub fn get_number(val: NativeValue) -> f64 {
    f64::from_bits(val.data)
}

pub fn get_integer(val: NativeValue) -> i64 {
    f64::from_bits(val.data) as i64
}

pub fn get_boolean(val: NativeValue) -> bool {
    val.data != 0
}

pub fn get_string(val: NativeValue) -> String {
    if val.tag != TAG_STRING || val.data == 0 {
        return String::new();
    }
    let ptr = val.data as *const c_char;
    unsafe {
        match CStr::from_ptr(ptr).to_str() {
            Ok(s) => s.to_string(),
            Err(_) => String::new(),
        }
    }
}

pub fn free_string(val: NativeValue) {
    if val.tag == TAG_STRING && val.data != 0 {
        unsafe {
            let _ = CString::from_raw(val.data as *mut c_char);
        }
    }
}

pub fn object_new() -> NativeValue {
    NativeValue {
        tag: TAG_OBJECT,
        data: 0,
    }
}

pub fn array_new() -> NativeValue {
    NativeValue {
        tag: TAG_ARRAY,
        data: 0,
    }
}

pub trait NativeModuleTrait: Send {
    fn name(&self) -> &str;
    fn get_function(&self, name: &str) -> Option<NativeFn>;
    fn keys(&self) -> Vec<String>;
}

pub struct NativeModuleRegistry {
    modules: HashMap<String, Box<dyn NativeModuleTrait>>,
}

impl NativeModuleRegistry {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }

    pub fn register(&mut self, module: Box<dyn NativeModuleTrait>) {
        let name = module.name().to_string();
        self.modules.insert(name, module);
    }

    pub fn get(&self, name: &str) -> Option<&dyn NativeModuleTrait> {
        self.modules.get(name).map(|m| m.as_ref())
    }
}

pub static MODULE_REGISTRY: Mutex<Option<NativeModuleRegistry>> = Mutex::new(None);

pub fn init_registry() {
    let mut guard = MODULE_REGISTRY.lock().unwrap();
    if guard.is_none() {
        *guard = Some(NativeModuleRegistry::new());
    }
}

pub fn register_module(module: Box<dyn NativeModuleTrait>) {
    let mut guard = MODULE_REGISTRY.lock().unwrap();
    if let Some(ref mut registry) = *guard {
        registry.register(module);
    }
}
