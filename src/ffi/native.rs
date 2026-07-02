use crate::objects::Value;
use rustc_hash::FxHashMap;
use std::sync::{Arc, Mutex};

pub type NativeFunction = fn(args: &[Value]) -> Result<Value, String>;

pub struct NativeRegistry {
    functions: Vec<NativeFunction>,
    named_functions: FxHashMap<String, usize>,
}

impl NativeRegistry {
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
            named_functions: FxHashMap::default(),
        }
    }

    pub fn register(&mut self, func: NativeFunction) -> usize {
        let id = self.functions.len();
        self.functions.push(func);
        id
    }

    pub fn register_named(&mut self, name: &str, func: NativeFunction) -> usize {
        let id = self.register(func);
        self.named_functions.insert(name.to_string(), id);
        id
    }

    pub fn get(&self, id: usize) -> Option<&NativeFunction> {
        self.functions.get(id)
    }

    pub fn get_by_name(&self, name: &str) -> Option<&NativeFunction> {
        self.named_functions
            .get(name)
            .and_then(|id| self.functions.get(*id))
    }

    pub fn call(&self, id: usize, args: &[Value]) -> Result<Value, String> {
        let func = self
            .get(id)
            .ok_or_else(|| "Function not found".to_string())?;
        func(args)
    }

    pub fn call_by_name(&self, name: &str, args: &[Value]) -> Result<Value, String> {
        let func = self
            .get_by_name(name)
            .ok_or_else(|| format!("Function '{}' not found", name))?;
        func(args)
    }

    pub fn function_count(&self) -> usize {
        self.functions.len()
    }
}

impl Default for NativeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SharedNativeRegistry {
    inner: Arc<Mutex<NativeRegistry>>,
}

impl SharedNativeRegistry {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(NativeRegistry::new())),
        }
    }

    pub fn register(&self, func: NativeFunction) -> usize {
        self.inner.lock().unwrap().register(func)
    }

    pub fn register_named(&self, name: &str, func: NativeFunction) -> usize {
        self.inner.lock().unwrap().register_named(name, func)
    }

    pub fn call(&self, id: usize, args: &[Value]) -> Result<Value, String> {
        self.inner.lock().unwrap().call(id, args)
    }

    pub fn call_by_name(&self, name: &str, args: &[Value]) -> Result<Value, String> {
        self.inner.lock().unwrap().call_by_name(name, args)
    }
}

impl Default for SharedNativeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn tails_register_native(
    registry: *mut NativeRegistry,
    func: NativeFunction,
) -> usize {
    if registry.is_null() {
        return usize::MAX;
    }

    let registry = unsafe { &mut *registry };
    registry.register(func)
}

#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn tails_register_native_named(
    registry: *mut NativeRegistry,
    name: *const std::os::raw::c_char,
    func: NativeFunction,
) -> usize {
    if registry.is_null() || name.is_null() {
        return usize::MAX;
    }

    let registry = unsafe { &mut *registry };
    let name = unsafe { std::ffi::CStr::from_ptr(name) };

    match name.to_str() {
        Ok(name_str) => registry.register_named(name_str, func),
        Err(_) => usize::MAX,
    }
}

#[no_mangle]
pub extern "C" fn tails_call_native(
    registry: *const NativeRegistry,
    id: usize,
    args: *const super::TailsValue,
    args_len: usize,
) -> super::TailsValue {
    if registry.is_null() {
        return super::TailsValue { tag: 0, data: 0 };
    }

    let registry = unsafe { &*registry };
    let args = if args.is_null() || args_len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args, args_len) }
    };

    let values: Vec<Value> = args
        .iter()
        .map(|v| super::tails_value_to_value(*v))
        .collect();

    match registry.call(id, &values) {
        Ok(value) => super::value_to_tails_value(value),
        Err(_) => super::TailsValue { tag: 0, data: 0 },
    }
}

#[no_mangle]
pub extern "C" fn tails_registry_new() -> *mut NativeRegistry {
    let registry = NativeRegistry::new();
    Box::into_raw(Box::new(registry))
}

#[no_mangle]
pub extern "C" fn tails_registry_free(registry: *mut NativeRegistry) {
    if !registry.is_null() {
        unsafe {
            let _ = Box::from_raw(registry);
        }
    }
}

#[no_mangle]
pub extern "C" fn tails_registry_count(registry: *const NativeRegistry) -> usize {
    if registry.is_null() {
        return 0;
    }

    let registry = unsafe { &*registry };
    registry.function_count()
}
