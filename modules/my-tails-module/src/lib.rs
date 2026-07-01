use std::collections::HashMap;
use std::sync::Mutex;
use tails_abi::{
    FromNativeValue, ModuleHandle, NativeFn, NativeModule, NativeValue, TailsModule, ToNativeValue,
};

// --- Counter class ---

struct Counter {
    count: f64,
}

static COUNTERS: Mutex<Option<HashMap<u32, Counter>>> = Mutex::new(None);

fn with_counters<F, R>(f: F) -> R
where
    F: FnOnce(&mut HashMap<u32, Counter>) -> R,
{
    let mut guard = COUNTERS.lock().unwrap();
    if guard.is_none() {
        *guard = Some(HashMap::new());
    }
    f(guard.as_mut().unwrap())
}

static COUNTER_ID: Mutex<u32> = Mutex::new(1);

fn next_id() -> u32 {
    let mut id = COUNTER_ID.lock().unwrap();
    let val = *id;
    *id += 1;
    val
}

#[no_mangle]
pub extern "C" fn __tails_ffi_counter_new(
    _interp: *mut std::ffi::c_void,
    _this: NativeValue,
    args: *const NativeValue,
    argc: i32,
) -> NativeValue {
    let args_slice = if args.is_null() || argc <= 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args, argc as usize) }
    };

    let initial = if let Some(first) = args_slice.first() {
        f64::from_native_value(*first).unwrap_or(0.0)
    } else {
        0.0
    };

    let id = next_id();
    with_counters(|map| {
        map.insert(id, Counter { count: initial });
    });

    // Return the id as a NativeValue object (tag=5 for object)
    NativeValue {
        tag: 5,
        data: id as u64,
    }
}

#[no_mangle]
pub extern "C" fn __tails_ffi_counter_increment(
    _interp: *mut std::ffi::c_void,
    this: NativeValue,
    _args: *const NativeValue,
    _argc: i32,
) -> NativeValue {
    let id = this.data as u32;
    with_counters(|map| {
        if let Some(counter) = map.get_mut(&id) {
            counter.count += 1.0;
        }
    });
    NativeValue { tag: 0, data: 0 } // undefined
}

#[no_mangle]
pub extern "C" fn __tails_ffi_counter_decrement(
    _interp: *mut std::ffi::c_void,
    this: NativeValue,
    _args: *const NativeValue,
    _argc: i32,
) -> NativeValue {
    let id = this.data as u32;
    with_counters(|map| {
        if let Some(counter) = map.get_mut(&id) {
            counter.count -= 1.0;
        }
    });
    NativeValue { tag: 0, data: 0 } // undefined
}

#[no_mangle]
pub extern "C" fn __tails_ffi_counter_get_count(
    _interp: *mut std::ffi::c_void,
    this: NativeValue,
    _args: *const NativeValue,
    _argc: i32,
) -> NativeValue {
    let id = this.data as u32;
    let count = with_counters(|map| map.get(&id).map(|c| c.count).unwrap_or(0.0));
    ToNativeValue::to_native_value(&count).unwrap_or_else(|_| NativeValue { tag: 0, data: 0 })
}

// --- Simple functions ---

#[no_mangle]
pub extern "C" fn __tails_ffi_greet(
    _interp: *mut std::ffi::c_void,
    _this: NativeValue,
    args: *const NativeValue,
    argc: i32,
) -> NativeValue {
    let args_slice = if args.is_null() || argc <= 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args, argc as usize) }
    };

    let name = if let Some(first) = args_slice.first() {
        String::from_native_value(*first).unwrap_or_default()
    } else {
        String::new()
    };

    let result = format!("Hello, {}!", name);
    ToNativeValue::to_native_value(&result).unwrap_or_else(|_| NativeValue { tag: 0, data: 0 })
}

#[no_mangle]
pub extern "C" fn __tails_ffi_add(
    _interp: *mut std::ffi::c_void,
    _this: NativeValue,
    args: *const NativeValue,
    argc: i32,
) -> NativeValue {
    let args_slice = if args.is_null() || argc <= 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args, argc as usize) }
    };

    let a = if let Some(first) = args_slice.first() {
        f64::from_native_value(*first).unwrap_or(0.0)
    } else {
        0.0
    };

    let b = if let Some(second) = args_slice.get(1) {
        f64::from_native_value(*second).unwrap_or(0.0)
    } else {
        0.0
    };

    let result = a + b;
    ToNativeValue::to_native_value(&result).unwrap_or_else(|_| NativeValue { tag: 0, data: 0 })
}

#[no_mangle]
pub extern "C" fn __tails_ffi_multiply(
    _interp: *mut std::ffi::c_void,
    _this: NativeValue,
    args: *const NativeValue,
    argc: i32,
) -> NativeValue {
    let args_slice = if args.is_null() || argc <= 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args, argc as usize) }
    };

    let a = if let Some(first) = args_slice.first() {
        f64::from_native_value(*first).unwrap_or(0.0)
    } else {
        0.0
    };

    let b = if let Some(second) = args_slice.get(1) {
        f64::from_native_value(*second).unwrap_or(0.0)
    } else {
        0.0
    };

    let result = a * b;
    ToNativeValue::to_native_value(&result).unwrap_or_else(|_| NativeValue { tag: 0, data: 0 })
}

#[no_mangle]
pub extern "C" fn tails_native_init() -> *mut ModuleHandle {
    let module = NativeModule::new("my-tails-module");
    let mut handle = ModuleHandle::new(module);

    handle
        .module
        .register("greet", __tails_ffi_greet as NativeFn);
    handle.module.register("add", __tails_ffi_add as NativeFn);
    handle
        .module
        .register("multiply", __tails_ffi_multiply as NativeFn);
    handle
        .module
        .register("Counter", __tails_ffi_counter_new as NativeFn);
    handle.module.register(
        "counter_increment",
        __tails_ffi_counter_increment as NativeFn,
    );
    handle.module.register(
        "counter_decrement",
        __tails_ffi_counter_decrement as NativeFn,
    );
    handle.module.register(
        "counter_getCount",
        __tails_ffi_counter_get_count as NativeFn,
    );

    Box::into_raw(Box::new(handle))
}

#[used]
#[doc(hidden)]
#[no_mangle]
pub static __TAILS_DTS_GREET: &str = "export function greet(name: string) => string;";

#[used]
#[doc(hidden)]
#[no_mangle]
pub static __TAILS_DTS_ADD: &str = "export function add(a: number, b: number) => number;";

#[used]
#[doc(hidden)]
#[no_mangle]
pub static __TAILS_DTS_MULTIPLY: &str = "export function multiply(a: number, b: number) => number;";

#[used]
#[doc(hidden)]
#[no_mangle]
pub static __TAILS_DTS_COUNTER_CLASS: &str = "export class Counter { constructor(initial?: number); increment(): void; decrement(): void; getCount(): number; }";

#[used]
#[doc(hidden)]
#[no_mangle]
pub static __TAILS_MODULE_META_MY_MODULE: &str = "my-tails-module";
