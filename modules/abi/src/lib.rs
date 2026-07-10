pub mod loader;
pub mod module;
pub mod traits;

use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use once_cell::sync::Lazy;

pub use loader::{extract_module_name, load_native_module, resolve_native_path, NativeLibrary};
pub use module::{NativeModuleExport, TailsModule};
pub use traits::{FromNativeValue, ToNativeValue};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct NativeValue {
    pub tag: u32,
    pub data: u64,
}

impl Default for NativeValue {
    fn default() -> Self {
        undefined()
    }
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
        // Safety: caller of NativeString construction guarantees `ptr` is valid
        // for `len` bytes for the lifetime of this value (typically a static or
        // arena-owned buffer owned by the native module).
        let slice = unsafe { std::slice::from_raw_parts(self.ptr, self.len) };
        String::from_utf8_lossy(slice).to_string()
    }
}

/// Take ownership of a `ModuleHandle` allocated by a native module init function.
///
/// # Safety
/// - `handle` must be non-null
/// - `handle` must point to a valid `ModuleHandle` allocated with `Box::into_raw`
/// - After this call, the pointer must not be used again
pub unsafe fn take_module_handle(handle: *mut ModuleHandle) -> Box<ModuleHandle> {
    Box::from_raw(handle)
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
        data: (val as f64).to_bits(),
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
    // Safety: TAG_STRING values store a pointer to a valid NUL-terminated C string
    // owned by the runtime / caller for the duration of this call.
    match unsafe { CStr::from_ptr(ptr) }.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => String::new(),
    }
}

pub fn free_string(val: NativeValue) {
    if val.tag == TAG_STRING && val.data != 0 {
        // Safety: data must be a pointer previously created with CString::into_raw
        // (or equivalent) for this string value; ownership is transferred here.
        let _ = unsafe { CString::from_raw(val.data as *mut c_char) };
    }
}

// ============================================================================
// Handle Registry — zero-copy passing of complex values (objects/arrays)
// ============================================================================

type RegistryType = Mutex<HashMap<u64, simd_json::OwnedValue>>;

static LOCAL_REGISTRY: Lazy<RegistryType> = Lazy::new(|| Mutex::new(HashMap::new()));

static mut SHARED_REGISTRY: *const RegistryType = std::ptr::null();

static LOCAL_HANDLE_COUNTER: AtomicU64 = AtomicU64::new(1);

static mut SHARED_COUNTER_PTR: *const AtomicU64 = std::ptr::null();

#[no_mangle]
pub extern "C" fn tails_abi_get_shared_registry() -> *const std::ffi::c_void {
    unsafe {
        if SHARED_REGISTRY.is_null() {
            SHARED_REGISTRY = &*LOCAL_REGISTRY as *const RegistryType;
        }
        SHARED_REGISTRY as *const std::ffi::c_void
    }
}

#[no_mangle]
pub extern "C" fn tails_abi_set_shared_registry(ptr: *const std::ffi::c_void) {
    unsafe {
        SHARED_REGISTRY = ptr as *const RegistryType;
    }
}

#[no_mangle]
pub extern "C" fn tails_abi_get_shared_counter_ptr() -> *const std::ffi::c_void {
    &LOCAL_HANDLE_COUNTER as *const AtomicU64 as *const std::ffi::c_void
}

#[no_mangle]
pub extern "C" fn tails_abi_set_shared_counter(ptr: *const std::ffi::c_void) {
    unsafe {
        SHARED_COUNTER_PTR = ptr as *const AtomicU64;
    }
}

pub fn tails_abi_get_next_handle_id() -> u64 {
    unsafe {
        if !SHARED_COUNTER_PTR.is_null() {
            (*SHARED_COUNTER_PTR).fetch_add(1, Ordering::Relaxed)
        } else {
            LOCAL_HANDLE_COUNTER.fetch_add(1, Ordering::Relaxed)
        }
    }
}

pub(crate) fn resolve_shared_registry() -> &'static RegistryType {
    unsafe {
        if !SHARED_REGISTRY.is_null() {
            &*SHARED_REGISTRY
        } else {
            SHARED_REGISTRY = &*LOCAL_REGISTRY as *const RegistryType;
            &*SHARED_REGISTRY
        }
    }
}

pub fn store_handle(value: simd_json::OwnedValue) -> NativeValue {
    let id = tails_abi_get_next_handle_id();
    let tag = match &value {
        simd_json::OwnedValue::Array(_) => TAG_ARRAY,
        _ => TAG_OBJECT,
    };
    resolve_shared_registry().lock().unwrap().insert(id, value);
    NativeValue {
        tag,
        data: id,
    }
}

pub fn get_handle(id: u64) -> Option<simd_json::OwnedValue> {
    resolve_shared_registry().lock().unwrap().get(&id).cloned()
}

pub fn take_handle(id: u64) -> Option<simd_json::OwnedValue> {
    resolve_shared_registry().lock().unwrap().remove(&id)
}

pub fn peek_handle(
    id: u64,
) -> Option<std::sync::MutexGuard<'static, HashMap<u64, simd_json::OwnedValue>>> {
    let guard = resolve_shared_registry().lock().unwrap();
    if guard.contains_key(&id) {
        Some(guard)
    } else {
        None
    }
}

pub fn free_handle(id: u64) {
    resolve_shared_registry().lock().unwrap().remove(&id);
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

impl Default for NativeModuleRegistry {
    fn default() -> Self {
        Self::new()
    }
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
