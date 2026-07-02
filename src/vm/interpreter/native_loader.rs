use crate::objects::Value;
use crate::runtime_env::native_fns::constants as c;
use crate::vm::gc::GarbageCollector;
use crate::vm::interpreter::{HeapValue, JsObject};
use std::collections::HashMap;

type NativeModuleFactory = fn(&mut Vec<HeapValue>, &mut GarbageCollector) -> HashMap<String, Value>;

pub struct NativeModuleRegistry {
    modules: HashMap<String, Box<NativeModuleFactory>>,
}

impl NativeModuleRegistry {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
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
        #[cfg(feature = "http")]
        "http" => registry.register("http", create_http_module),
        #[cfg(feature = "net")]
        "net" => registry.register("net", create_net_module),
        _ => {}
    }
}

#[cfg(feature = "fs")]
pub fn create_fs_module(
    _heap: &mut Vec<HeapValue>,
    _gc: &mut GarbageCollector,
) -> HashMap<String, Value> {
    let mut props = HashMap::new();
    props.insert(
        "readFileSync".into(),
        Value::NativeFunction(c::FS_READ_FILE_SYNC),
    );
    props.insert(
        "writeFileSync".into(),
        Value::NativeFunction(c::FS_WRITE_FILE_SYNC),
    );
    props.insert(
        "existsSync".into(),
        Value::NativeFunction(c::FS_EXISTS_SYNC),
    );
    props.insert("mkdirSync".into(), Value::NativeFunction(c::FS_MKDIR_SYNC));
    props.insert(
        "readdirSync".into(),
        Value::NativeFunction(c::FS_READDIR_SYNC),
    );
    props.insert("statSync".into(), Value::NativeFunction(c::FS_STAT_SYNC));
    props.insert(
        "unlinkSync".into(),
        Value::NativeFunction(c::FS_UNLINK_SYNC),
    );
    props.insert("rmSync".into(), Value::NativeFunction(c::FS_RM_SYNC));
    props.insert(
        "copyFileSync".into(),
        Value::NativeFunction(c::FS_COPY_FILE_SYNC),
    );
    props.insert(
        "renameSync".into(),
        Value::NativeFunction(c::FS_RENAME_SYNC),
    );
    props.insert(
        "appendFileSync".into(),
        Value::NativeFunction(c::FS_APPEND_FILE_SYNC),
    );
    props
}

#[cfg(feature = "fs")]
pub fn create_fs_promises_module(
    _heap: &mut Vec<HeapValue>,
    _gc: &mut GarbageCollector,
) -> HashMap<String, Value> {
    let mut props = HashMap::new();
    props.insert("readdir".into(), Value::NativeFunction(c::FS_READDIR));
    props.insert("readFile".into(), Value::NativeFunction(c::FS_READ_FILE));
    props.insert("writeFile".into(), Value::NativeFunction(c::FS_WRITE_FILE));
    props.insert("stat".into(), Value::NativeFunction(c::FS_STAT));
    props.insert("mkdir".into(), Value::NativeFunction(c::FS_MKDIR));
    props.insert("unlink".into(), Value::NativeFunction(c::FS_UNLINK));
    props.insert("copyFile".into(), Value::NativeFunction(c::FS_COPY_FILE));
    props.insert("rename".into(), Value::NativeFunction(c::FS_RENAME));
    props
}

#[cfg(feature = "path")]
pub fn create_path_module(
    _heap: &mut Vec<HeapValue>,
    _gc: &mut GarbageCollector,
) -> HashMap<String, Value> {
    let mut props = HashMap::new();
    props.insert("join".into(), Value::NativeFunction(c::PATH_JOIN));
    props.insert("resolve".into(), Value::NativeFunction(c::PATH_RESOLVE));
    props.insert("basename".into(), Value::NativeFunction(c::PATH_BASENAME));
    props.insert("dirname".into(), Value::NativeFunction(c::PATH_DIRNAME));
    props.insert("extname".into(), Value::NativeFunction(c::PATH_EXTNAME));
    props.insert("relative".into(), Value::NativeFunction(c::PATH_RELATIVE));
    props.insert(
        "isAbsolute".into(),
        Value::NativeFunction(c::PATH_IS_ABSOLUTE),
    );
    props.insert("normalize".into(), Value::NativeFunction(c::PATH_NORMALIZE));
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
    props.insert("exit".into(), Value::NativeFunction(c::PROCESS_EXIT));
    props.insert("cwd".into(), Value::NativeFunction(c::PROCESS_CWD));
    props.insert("chdir".into(), Value::NativeFunction(c::PROCESS_CHDIR));
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
    stdout_props.insert(
        "write".into(),
        Value::NativeFunction(c::PROCESS_STDOUT_WRITE),
    );
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
    stderr_props.insert(
        "write".into(),
        Value::NativeFunction(c::PROCESS_STDOUT_WRITE),
    );
    let stderr_idx = gc.allocate(
        heap,
        HeapValue::Object(JsObject {
            properties: stderr_props,
            prototype: None,
            extensible: true,
        }),
    );
    props.insert("stderr".into(), Value::Object(stderr_idx));

    props.insert("hrtime".into(), Value::NativeFunction(c::PROCESS_HRTIME));
    props.insert(
        "hrtime.bigint".into(),
        Value::NativeFunction(c::PROCESS_HRTIME_BIGINT),
    );
    props.insert(
        "nextTick".into(),
        Value::NativeFunction(c::PROCESS_NEXT_TICK),
    );

    props
}

pub fn create_buffer_module(
    heap: &mut Vec<HeapValue>,
    gc: &mut GarbageCollector,
) -> HashMap<String, Value> {
    let mut props = HashMap::new();

    // Static methods
    props.insert("alloc".into(), Value::NativeFunction(c::BUFFER_ALLOC));
    props.insert("from".into(), Value::NativeFunction(c::BUFFER_FROM));
    props.insert("concat".into(), Value::NativeFunction(c::BUFFER_CONCAT));
    props.insert(
        "isBuffer".into(),
        Value::NativeFunction(c::BUFFER_IS_BUFFER),
    );
    props.insert(
        "byteLength".into(),
        Value::NativeFunction(c::BUFFER_BYTE_LENGTH),
    );

    // Prototype methods
    props.insert(
        "toString".into(),
        Value::NativeFunction(c::BUFFER_TO_STRING),
    );
    props.insert("write".into(), Value::NativeFunction(c::BUFFER_WRITE));
    props.insert("slice".into(), Value::NativeFunction(c::BUFFER_SLICE));
    props.insert("copy".into(), Value::NativeFunction(c::BUFFER_COPY));
    props.insert("fill".into(), Value::NativeFunction(c::BUFFER_FILL));
    props.insert("compare".into(), Value::NativeFunction(c::BUFFER_COMPARE));
    props.insert("equals".into(), Value::NativeFunction(c::BUFFER_EQUALS));
    props.insert("indexOf".into(), Value::NativeFunction(c::BUFFER_INDEX_OF));

    // Prototype object
    let buffer_proto_idx = gc.allocate(
        heap,
        HeapValue::Object(JsObject {
            properties: {
                let mut proto_props = HashMap::new();
                proto_props.insert(
                    "toString".into(),
                    Value::NativeFunction(c::BUFFER_TO_STRING),
                );
                proto_props.insert("write".into(), Value::NativeFunction(c::BUFFER_WRITE));
                proto_props.insert("slice".into(), Value::NativeFunction(c::BUFFER_SLICE));
                proto_props.insert("copy".into(), Value::NativeFunction(c::BUFFER_COPY));
                proto_props.insert("fill".into(), Value::NativeFunction(c::BUFFER_FILL));
                proto_props.insert("compare".into(), Value::NativeFunction(c::BUFFER_COMPARE));
                proto_props.insert("equals".into(), Value::NativeFunction(c::BUFFER_EQUALS));
                proto_props.insert("indexOf".into(), Value::NativeFunction(c::BUFFER_INDEX_OF));
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
    intl_obj_props.insert(
        "DateTimeFormat".into(),
        Value::NativeFunction(c::DATETIME_FORMAT_CONSTRUCTOR),
    );
    intl_obj_props.insert(
        "NumberFormat".into(),
        Value::NativeFunction(c::NUMBER_FORMAT_CONSTRUCTOR),
    );

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
    props.insert(
        "EventEmitter".into(),
        Value::NativeFunction(c::EVENT_EMITTER_CONSTRUCTOR),
    );

    // Prototype methods
    let mut proto_props = HashMap::new();
    proto_props.insert("on".into(), Value::NativeFunction(c::EVENT_EMITTER_ON));
    proto_props.insert("emit".into(), Value::NativeFunction(c::EVENT_EMITTER_EMIT));
    proto_props.insert("off".into(), Value::NativeFunction(c::EVENT_EMITTER_OFF));
    proto_props.insert(
        "listenerCount".into(),
        Value::NativeFunction(c::EVENT_EMITTER_LISTENER_COUNT),
    );

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
    props.insert("platform".into(), Value::NativeFunction(c::OS_PLATFORM));
    props.insert("arch".into(), Value::NativeFunction(c::OS_ARCH));
    props.insert("cpus".into(), Value::NativeFunction(c::OS_CPUS));
    props.insert("totalmem".into(), Value::NativeFunction(c::OS_TOTALMEM));
    props.insert("freemem".into(), Value::NativeFunction(c::OS_FREEMEM));
    props.insert("uptime".into(), Value::NativeFunction(c::OS_UPTIME));
    props.insert("hostname".into(), Value::NativeFunction(c::OS_HOSTNAME));
    props.insert("type".into(), Value::NativeFunction(c::OS_TYPE));
    props.insert("release".into(), Value::NativeFunction(c::OS_RELEASE));
    props.insert("homedir".into(), Value::NativeFunction(c::OS_HOMEDIR));
    props.insert("tmpdir".into(), Value::NativeFunction(c::OS_TMPDIR));
    props
}

pub fn create_crypto_module(
    _heap: &mut Vec<HeapValue>,
    _gc: &mut GarbageCollector,
) -> HashMap<String, Value> {
    let mut props = HashMap::new();
    props.insert(
        "randomBytes".into(),
        Value::NativeFunction(c::CRYPTO_RANDOM_BYTES),
    );
    props.insert(
        "randomUUID".into(),
        Value::NativeFunction(c::CRYPTO_RANDOM_UUID),
    );
    props.insert(
        "createHash".into(),
        Value::NativeFunction(c::CRYPTO_CREATE_HASH),
    );
    props
}

pub fn create_assert_module(
    heap: &mut Vec<HeapValue>,
    gc: &mut GarbageCollector,
) -> HashMap<String, Value> {
    let mut props = HashMap::new();

    let mut assert_props = HashMap::new();
    assert_props.insert("strictEqual".into(), Value::NativeFunction(c::HEADERS_HAS));
    assert_props.insert("ok".into(), Value::NativeFunction(c::HEADERS_SET));
    assert_props.insert("equal".into(), Value::NativeFunction(c::HEADERS_HAS));
    assert_props.insert("deepEqual".into(), Value::NativeFunction(c::HEADERS_HAS));

    let assert_obj_idx = gc.allocate(
        heap,
        HeapValue::Object(JsObject {
            properties: assert_props,
            prototype: None,
            extensible: true,
        }),
    );

    props.insert("default".into(), Value::Object(assert_obj_idx));
    props.insert("strictEqual".into(), Value::NativeFunction(c::HEADERS_HAS));
    props.insert("ok".into(), Value::NativeFunction(c::HEADERS_SET));
    props.insert("equal".into(), Value::NativeFunction(c::HEADERS_HAS));
    props.insert("deepEqual".into(), Value::NativeFunction(c::HEADERS_HAS));
    props
}

pub fn create_child_process_module(
    _heap: &mut Vec<HeapValue>,
    _gc: &mut GarbageCollector,
) -> HashMap<String, Value> {
    let mut props = HashMap::new();
    props.insert(
        "execSync".into(),
        Value::NativeFunction(c::CHILD_PROCESS_EXEC_SYNC),
    );
    props.insert("exec".into(), Value::NativeFunction(c::CHILD_PROCESS_EXEC));
    props.insert(
        "spawn".into(),
        Value::NativeFunction(c::CHILD_PROCESS_SPAWN),
    );
    props
}

pub fn create_url_module(
    _heap: &mut Vec<HeapValue>,
    _gc: &mut GarbageCollector,
) -> HashMap<String, Value> {
    let mut props = HashMap::new();
    props.insert(
        "fileURLToPath".into(),
        Value::NativeFunction(c::URL_FILE_URL_TO_PATH),
    );
    props
}

#[cfg(feature = "http")]
pub fn create_http_module(
    _heap: &mut Vec<HeapValue>,
    _gc: &mut GarbageCollector,
) -> HashMap<String, Value> {
    let mut props = HashMap::new();
    props.insert(
        "createServer".into(),
        Value::NativeFunction(c::HTTP_CREATE_SERVER),
    );
    props
}

#[cfg(feature = "net")]
pub fn create_net_module(
    _heap: &mut Vec<HeapValue>,
    _gc: &mut GarbageCollector,
) -> HashMap<String, Value> {
    let mut props = HashMap::new();
    props.insert(
        "createConnection".into(),
        Value::NativeFunction(c::NET_CREATE_CONNECTION),
    );
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
