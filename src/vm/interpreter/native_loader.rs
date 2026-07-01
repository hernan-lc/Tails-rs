use crate::objects::Value;
use crate::vm::gc::GarbageCollector;
use crate::vm::interpreter::{HeapValue, JsObject};
use std::collections::HashMap;

type NativeModuleFactory = fn(&mut Vec<HeapValue>, &mut GarbageCollector) -> HashMap<String, Value>;

pub struct NativeModuleRegistry {
    modules: HashMap<String, Box<NativeModuleFactory>>,
    dynamic_libraries: Vec<DynamicLibraryEntry>,
}

struct DynamicLibraryEntry {
    _library: libloading::Library,
}

impl NativeModuleRegistry {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            dynamic_libraries: Vec::new(),
        }
    }

    pub fn register(&mut self, name: &str, factory: NativeModuleFactory) {
        self.modules.insert(name.to_string(), Box::new(factory));
    }

    pub fn has_module(&self, name: &str) -> bool {
        self.modules.contains_key(name)
    }

    pub fn load_module(
        &self,
        name: &str,
        heap: &mut Vec<HeapValue>,
        gc: &mut GarbageCollector,
    ) -> crate::errors::Result<HashMap<String, Value>> {
        if let Some(factory) = self.modules.get(name) {
            Ok(factory(heap, gc))
        } else {
            Err(crate::errors::Error::RuntimeError(format!(
                "Native module '{}' not found in registry",
                name
            )))
        }
    }

    pub fn try_load_dynamic(&mut self, name: &str) -> Option<HashMap<String, Value>> {
        let dist_dir = std::env::current_dir().ok()?.join("dist");
        let lib_path = find_library_in_dir(&dist_dir, name)?;

        self.try_load_from_path(&lib_path)
    }

    pub fn try_load_from_path(
        &mut self,
        lib_path: &std::path::Path,
    ) -> Option<HashMap<String, Value>> {
        if !lib_path.exists() {
            return None;
        }

        unsafe {
            let library = libloading::Library::new(lib_path).ok()?;

            let mut exports = HashMap::new();

            type InitFn = fn() -> *mut tails_abi::ModuleHandle;

            // Try module-specific init first, then fallback to generic
            let module_stem = lib_path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            let base_name = module_stem
                .strip_prefix("lib")
                .unwrap_or(module_stem);

            let init_result = {
                let init_name = format!("tails_native_init_{}\0", base_name.replace('-', "_"));
                if let Ok(init_fn) = library.get::<InitFn>(init_name.as_bytes()) {
                    Some(init_fn)
                } else {
                    let init_name2 = format!("tails_native_init_{}\0", base_name);
                    if let Ok(init_fn) = library.get::<InitFn>(init_name2.as_bytes()) {
                        Some(init_fn)
                    } else {
                        library.get::<InitFn>(b"tails_native_init\0").ok()
                    }
                }
            };

            if let Some(init_fn) = init_result {
                let handle = init_fn();
                if !handle.is_null() {
                    let handle = Box::from_raw(handle);
                    for func_name in handle.module.functions.keys() {
                        let id = self.modules.len() + 2000;
                        exports.insert(func_name.clone(), Value::NativeFunction(id));
                    }
                }
            }

            self.dynamic_libraries
                .push(DynamicLibraryEntry { _library: library });

            Some(exports)
        }
    }
}

pub fn extract_module_name(source: &str) -> &str {
    let path = std::path::Path::new(source);
    let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or(source);
    if file_stem.contains('/') {
        file_stem.rsplit('/').next().unwrap_or(file_stem)
    } else {
        file_stem
    }
}

pub fn discover_module(name: &str, registry: &mut NativeModuleRegistry) {
    match name {
        #[cfg(feature = "fs")]
        "fs" => registry.register("fs", create_fs_module),
        #[cfg(feature = "fs")]
        "fs/promises" => registry.register("fs/promises", create_fs_promises_module),
        #[cfg(feature = "path")]
        "path" => registry.register("path", create_path_module),
        #[cfg(feature = "process")]
        "process" => registry.register("process", create_process_module),
        "buffer" => registry.register("buffer", create_buffer_module),
        "intl" => registry.register("intl", create_intl_module),
        "events" => registry.register("events", create_events_module),
        #[cfg(feature = "os")]
        "os" => registry.register("os", create_os_module),
        "crypto" => registry.register("crypto", create_crypto_module),
        "assert" => registry.register("assert", create_assert_module),
        "child_process" => registry.register("child_process", create_child_process_module),
        "url" => registry.register("url", create_url_module),
        _ => {}
    }
}

#[cfg(feature = "fs")]
pub fn create_fs_module(
    _heap: &mut Vec<HeapValue>,
    _gc: &mut GarbageCollector,
) -> HashMap<String, Value> {
    let mut props = HashMap::new();
    props.insert("readFileSync".into(), Value::NativeFunction(286));
    props.insert("writeFileSync".into(), Value::NativeFunction(287));
    props.insert("existsSync".into(), Value::NativeFunction(288));
    props.insert("mkdirSync".into(), Value::NativeFunction(289));
    props.insert("readdirSync".into(), Value::NativeFunction(290));
    props.insert("statSync".into(), Value::NativeFunction(291));
    props.insert("unlinkSync".into(), Value::NativeFunction(292));
    props.insert("rmSync".into(), Value::NativeFunction(293));
    props.insert("copyFileSync".into(), Value::NativeFunction(294));
    props.insert("renameSync".into(), Value::NativeFunction(295));
    props.insert("appendFileSync".into(), Value::NativeFunction(296));
    props
}

#[cfg(feature = "fs")]
pub fn create_fs_promises_module(
    _heap: &mut Vec<HeapValue>,
    _gc: &mut GarbageCollector,
) -> HashMap<String, Value> {
    let mut props = HashMap::new();
    props.insert("readdir".into(), Value::NativeFunction(333));
    props.insert("readFile".into(), Value::NativeFunction(334));
    props.insert("writeFile".into(), Value::NativeFunction(335));
    props.insert("stat".into(), Value::NativeFunction(336));
    props.insert("mkdir".into(), Value::NativeFunction(337));
    props.insert("unlink".into(), Value::NativeFunction(338));
    props.insert("copyFile".into(), Value::NativeFunction(339));
    props.insert("rename".into(), Value::NativeFunction(340));
    props
}

#[cfg(feature = "path")]
pub fn create_path_module(
    _heap: &mut Vec<HeapValue>,
    _gc: &mut GarbageCollector,
) -> HashMap<String, Value> {
    let mut props = HashMap::new();
    props.insert("join".into(), Value::NativeFunction(265));
    props.insert("resolve".into(), Value::NativeFunction(266));
    props.insert("basename".into(), Value::NativeFunction(267));
    props.insert("dirname".into(), Value::NativeFunction(268));
    props.insert("extname".into(), Value::NativeFunction(269));
    props.insert("relative".into(), Value::NativeFunction(270));
    props.insert("isAbsolute".into(), Value::NativeFunction(271));
    props.insert("normalize".into(), Value::NativeFunction(272));
    props.insert(
        "sep".into(),
        Value::String(std::path::MAIN_SEPARATOR.to_string()),
    );
    props.insert(
        "delimiter".into(),
        Value::String(
            if cfg!(target_os = "windows") {
                ";"
            } else {
                ":"
            }
            .to_string(),
        ),
    );
    props
}

#[cfg(feature = "process")]
pub fn create_process_module(
    heap: &mut Vec<HeapValue>,
    gc: &mut GarbageCollector,
) -> HashMap<String, Value> {
    let mut props = HashMap::new();

    // Scalar properties
    props.insert("exit".into(), Value::NativeFunction(239));
    props.insert("cwd".into(), Value::NativeFunction(240));
    props.insert("chdir".into(), Value::NativeFunction(241));
    props.insert(
        "platform".into(),
        Value::String(
            if cfg!(target_os = "linux") {
                "linux"
            } else if cfg!(target_os = "macos") {
                "darwin"
            } else if cfg!(target_os = "windows") {
                "win32"
            } else {
                "unknown"
            }
            .into(),
        ),
    );
    props.insert(
        "arch".into(),
        Value::String(
            if cfg!(target_arch = "x86_64") {
                "x64"
            } else if cfg!(target_arch = "aarch64") {
                "arm64"
            } else {
                "unknown"
            }
            .into(),
        ),
    );
    props.insert("pid".into(), Value::Integer(std::process::id() as i64));

    // process.env
    let mut env_props = HashMap::new();
    for (key, value) in std::env::vars() {
        env_props.insert(key, Value::String(value));
    }
    let env_obj_idx = gc.allocate(
        heap,
        HeapValue::Object(JsObject {
            properties: env_props,
            prototype: None,
            extensible: true,
        }),
    );
    props.insert("env".into(), Value::Object(env_obj_idx));

    // process.argv
    let argv: Vec<Value> = std::env::args().map(Value::String).collect();
    let argv_idx = gc.allocate(
        heap,
        HeapValue::Array(crate::vm::interpreter::JsArray { elements: argv }),
    );
    props.insert("argv".into(), Value::Array(argv_idx));

    // process.stdout
    let mut stdout_props = HashMap::new();
    stdout_props.insert("write".into(), Value::NativeFunction(242));
    let stdout_idx = gc.allocate(
        heap,
        HeapValue::Object(JsObject {
            properties: stdout_props,
            prototype: None,
            extensible: true,
        }),
    );
    props.insert("stdout".into(), Value::Object(stdout_idx));

    // process.stderr
    let mut stderr_props = HashMap::new();
    stderr_props.insert("write".into(), Value::NativeFunction(242));
    let stderr_idx = gc.allocate(
        heap,
        HeapValue::Object(JsObject {
            properties: stderr_props,
            prototype: None,
            extensible: true,
        }),
    );
    props.insert("stderr".into(), Value::Object(stderr_idx));

    props.insert("hrtime".into(), Value::NativeFunction(243));
    props.insert("hrtime.bigint".into(), Value::NativeFunction(244));
    props.insert("nextTick".into(), Value::NativeFunction(245));

    props
}

pub fn create_buffer_module(
    heap: &mut Vec<HeapValue>,
    gc: &mut GarbageCollector,
) -> HashMap<String, Value> {
    let mut props = HashMap::new();

    // Static methods
    props.insert("alloc".into(), Value::NativeFunction(247));
    props.insert("from".into(), Value::NativeFunction(248));
    props.insert("concat".into(), Value::NativeFunction(249));
    props.insert("isBuffer".into(), Value::NativeFunction(250));
    props.insert("byteLength".into(), Value::NativeFunction(251));

    // Prototype methods
    props.insert("toString".into(), Value::NativeFunction(252));
    props.insert("write".into(), Value::NativeFunction(253));
    props.insert("slice".into(), Value::NativeFunction(254));
    props.insert("copy".into(), Value::NativeFunction(255));
    props.insert("fill".into(), Value::NativeFunction(256));
    props.insert("compare".into(), Value::NativeFunction(257));
    props.insert("equals".into(), Value::NativeFunction(258));
    props.insert("indexOf".into(), Value::NativeFunction(259));

    // Prototype object
    let buffer_proto_idx = gc.allocate(
        heap,
        HeapValue::Object(JsObject {
            properties: {
                let mut proto_props = HashMap::new();
                proto_props.insert("toString".into(), Value::NativeFunction(252));
                proto_props.insert("write".into(), Value::NativeFunction(253));
                proto_props.insert("slice".into(), Value::NativeFunction(254));
                proto_props.insert("copy".into(), Value::NativeFunction(255));
                proto_props.insert("fill".into(), Value::NativeFunction(256));
                proto_props.insert("compare".into(), Value::NativeFunction(257));
                proto_props.insert("equals".into(), Value::NativeFunction(258));
                proto_props.insert("indexOf".into(), Value::NativeFunction(259));
                proto_props.insert("length".into(), Value::Integer(0));
                proto_props
            },
            prototype: None,
            extensible: true,
        }),
    );
    props.insert("prototype".into(), Value::Object(buffer_proto_idx));

    props
}

pub fn create_intl_module(
    heap: &mut Vec<HeapValue>,
    gc: &mut GarbageCollector,
) -> HashMap<String, Value> {
    let mut props = HashMap::new();

    let mut intl_obj_props = HashMap::new();
    intl_obj_props.insert("DateTimeFormat".into(), Value::NativeFunction(260));
    intl_obj_props.insert("NumberFormat".into(), Value::NativeFunction(261));

    let intl_obj_idx = gc.allocate(
        heap,
        HeapValue::Object(JsObject {
            properties: intl_obj_props,
            prototype: None,
            extensible: true,
        }),
    );
    props.insert("default".into(), Value::Object(intl_obj_idx));

    props
}

pub fn create_events_module(
    heap: &mut Vec<HeapValue>,
    gc: &mut GarbageCollector,
) -> HashMap<String, Value> {
    let mut props = HashMap::new();

    // EventEmitter constructor
    props.insert("EventEmitter".into(), Value::NativeFunction(312));

    // Prototype methods
    let mut proto_props = HashMap::new();
    proto_props.insert("on".into(), Value::NativeFunction(313));
    proto_props.insert("emit".into(), Value::NativeFunction(314));
    proto_props.insert("off".into(), Value::NativeFunction(315));
    proto_props.insert("listenerCount".into(), Value::NativeFunction(316));

    let proto_idx = gc.allocate(
        heap,
        HeapValue::Object(JsObject {
            properties: proto_props,
            prototype: None,
            extensible: true,
        }),
    );
    props.insert("prototype".into(), Value::Object(proto_idx));

    props
}

#[cfg(feature = "os")]
pub fn create_os_module(
    _heap: &mut Vec<HeapValue>,
    _gc: &mut GarbageCollector,
) -> HashMap<String, Value> {
    let mut props = HashMap::new();
    props.insert("platform".into(), Value::NativeFunction(319));
    props.insert("arch".into(), Value::NativeFunction(320));
    props.insert("cpus".into(), Value::NativeFunction(321));
    props.insert("totalmem".into(), Value::NativeFunction(322));
    props.insert("freemem".into(), Value::NativeFunction(323));
    props.insert("uptime".into(), Value::NativeFunction(324));
    props.insert("hostname".into(), Value::NativeFunction(325));
    props.insert("type".into(), Value::NativeFunction(326));
    props.insert("release".into(), Value::NativeFunction(327));
    props.insert("homedir".into(), Value::NativeFunction(328));
    props.insert("tmpdir".into(), Value::NativeFunction(329));
    props
}

pub fn create_crypto_module(
    _heap: &mut Vec<HeapValue>,
    _gc: &mut GarbageCollector,
) -> HashMap<String, Value> {
    let mut props = HashMap::new();
    props.insert("randomBytes".into(), Value::NativeFunction(330));
    props.insert("randomUUID".into(), Value::NativeFunction(331));
    props.insert("createHash".into(), Value::NativeFunction(332));
    props
}

pub fn create_assert_module(
    heap: &mut Vec<HeapValue>,
    gc: &mut GarbageCollector,
) -> HashMap<String, Value> {
    let mut props = HashMap::new();

    let mut assert_props = HashMap::new();
    assert_props.insert("strictEqual".into(), Value::NativeFunction(365));
    assert_props.insert("ok".into(), Value::NativeFunction(364));
    assert_props.insert("equal".into(), Value::NativeFunction(365));
    assert_props.insert("deepEqual".into(), Value::NativeFunction(365));

    let assert_obj_idx = gc.allocate(
        heap,
        HeapValue::Object(JsObject {
            properties: assert_props,
            prototype: None,
            extensible: true,
        }),
    );

    props.insert("default".into(), Value::Object(assert_obj_idx));
    props.insert("strictEqual".into(), Value::NativeFunction(365));
    props.insert("ok".into(), Value::NativeFunction(364));
    props.insert("equal".into(), Value::NativeFunction(365));
    props.insert("deepEqual".into(), Value::NativeFunction(365));
    props
}

pub fn create_child_process_module(
    _heap: &mut Vec<HeapValue>,
    _gc: &mut GarbageCollector,
) -> HashMap<String, Value> {
    let mut props = HashMap::new();
    props.insert("execSync".into(), Value::NativeFunction(377));
    props.insert("exec".into(), Value::NativeFunction(378));
    props.insert("spawn".into(), Value::NativeFunction(379));
    props
}

pub fn create_url_module(
    _heap: &mut Vec<HeapValue>,
    _gc: &mut GarbageCollector,
) -> HashMap<String, Value> {
    let mut props = HashMap::new();
    props.insert("fileURLToPath".into(), Value::NativeFunction(382));
    props
}

pub fn find_library_in_dir(dir: &std::path::Path, name: &str) -> Option<std::path::PathBuf> {
    let extensions = if cfg!(target_os = "windows") {
        vec!["dll"]
    } else if cfg!(target_os = "macos") {
        vec!["dylib"]
    } else {
        vec!["so"]
    };

    // Convert hyphens to underscores for the file name (Rust crate naming convention)
    let name_underscore = name.replace('-', "_");

    // Try all combinations of name variants and extensions
    let name_variants = vec![name, &name_underscore];

    for ext in &extensions {
        for name_variant in &name_variants {
            // Try direct name with extension
            let path = dir.join(format!("{}.{}", name_variant, ext));
            if path.exists() {
                return Some(path);
            }
            // Try with lib prefix on Unix
            if *ext != "dll" {
                let path = dir.join(format!("lib{}.{}", name_variant, ext));
                if path.exists() {
                    return Some(path);
                }
            }
        }
    }

    // Also check if the name already has an extension and exists directly
    for name_variant in &name_variants {
        let path = dir.join(name_variant);
        if path.exists() {
            return Some(path);
        }
    }

    None
}
