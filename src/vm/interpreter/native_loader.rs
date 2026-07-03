use crate::objects::Value;
use crate::props;
use crate::runtime_env::native_fns::constants as c;
use crate::vm::gc::GarbageCollector;
use crate::vm::interpreter::{HeapValue, JsObject};
use rustc_hash::FxHashMap;

type NativeModuleFactory = fn(&mut Vec<HeapValue>, &mut GarbageCollector) -> FxHashMap<String, Value>;

pub struct NativeModuleRegistry {
    modules: FxHashMap<String, Box<NativeModuleFactory>>,
}

impl NativeModuleRegistry {
    pub fn new() -> Self {
        Self {
            modules: FxHashMap::default(),
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
    ) -> crate::errors::Result<FxHashMap<String, Value>> {
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
        "util" => registry.register("util", create_util_module),
        "timers" => registry.register("timers", create_timers_module),
        "querystring" => registry.register("querystring", create_querystring_module),
        "stream" => registry.register("stream", create_stream_module),
        #[cfg(feature = "zlib")]
        "zlib" => registry.register("zlib", create_zlib_module),
        #[cfg(feature = "tls")]
        "tls" => registry.register("tls", create_tls_module),
        #[cfg(feature = "dns")]
        "dns" => registry.register("dns", create_dns_module),
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
) -> FxHashMap<String, Value> {
    props! {
        "readFileSync" => Value::NativeFunction(c::FS_READ_FILE_SYNC),
        "writeFileSync" => Value::NativeFunction(c::FS_WRITE_FILE_SYNC),
        "existsSync" => Value::NativeFunction(c::FS_EXISTS_SYNC),
        "mkdirSync" => Value::NativeFunction(c::FS_MKDIR_SYNC),
        "readdirSync" => Value::NativeFunction(c::FS_READDIR_SYNC),
        "statSync" => Value::NativeFunction(c::FS_STAT_SYNC),
        "unlinkSync" => Value::NativeFunction(c::FS_UNLINK_SYNC),
        "rmSync" => Value::NativeFunction(c::FS_RM_SYNC),
        "copyFileSync" => Value::NativeFunction(c::FS_COPY_FILE_SYNC),
        "renameSync" => Value::NativeFunction(c::FS_RENAME_SYNC),
        "appendFileSync" => Value::NativeFunction(c::FS_APPEND_FILE_SYNC),
    }
}

#[cfg(feature = "fs")]
pub fn create_fs_promises_module(
    _heap: &mut Vec<HeapValue>,
    _gc: &mut GarbageCollector,
) -> FxHashMap<String, Value> {
    props! {
        "readdir" => Value::NativeFunction(c::FS_READDIR),
        "read_file" => Value::NativeFunction(c::FS_READ_FILE),
        "write_file" => Value::NativeFunction(c::FS_WRITE_FILE),
        "stat" => Value::NativeFunction(c::FS_STAT),
        "mkdir" => Value::NativeFunction(c::FS_MKDIR),
        "unlink" => Value::NativeFunction(c::FS_UNLINK),
        "copy_file" => Value::NativeFunction(c::FS_COPY_FILE),
        "rename" => Value::NativeFunction(c::FS_RENAME),
    }
}

#[cfg(feature = "path")]
pub fn create_path_module(
    _heap: &mut Vec<HeapValue>,
    _gc: &mut GarbageCollector,
) -> FxHashMap<String, Value> {
    props! {
        "join" => Value::NativeFunction(c::PATH_JOIN),
        "resolve" => Value::NativeFunction(c::PATH_RESOLVE),
        "basename" => Value::NativeFunction(c::PATH_BASENAME),
        "dirname" => Value::NativeFunction(c::PATH_DIRNAME),
        "extname" => Value::NativeFunction(c::PATH_EXTNAME),
        "relative" => Value::NativeFunction(c::PATH_RELATIVE),
        "isAbsolute" => Value::NativeFunction(c::PATH_IS_ABSOLUTE),
        "normalize" => Value::NativeFunction(c::PATH_NORMALIZE),
        "sep" => Value::String(std::path::MAIN_SEPARATOR.to_string()),
        "delimiter" => Value::String(
            if cfg!(target_os = "windows") {
                ";"
            } else {
                ":"
            }
            .to_string(),
        ),
    }
}

#[cfg(feature = "process")]
pub fn create_process_module(
    heap: &mut Vec<HeapValue>,
    gc: &mut GarbageCollector,
) -> FxHashMap<String, Value> {
    // Start with the static set of `process.<name>` properties (function
    // pointers, scalar compile-time constants, and the `pid`). The
    // dynamic sub-objects (env / argv / stdout / stderr) are merged in
    // after the heap allocations below.
    let mut props: FxHashMap<String, Value> = props! {
        "exit" => Value::NativeFunction(c::PROCESS_EXIT),
        "cwd" => Value::NativeFunction(c::PROCESS_CWD),
        "chdir" => Value::NativeFunction(c::PROCESS_CHDIR),
        "platform" => Value::String(
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
        "arch" => Value::String(
            if cfg!(target_arch = "x86_64") {
                "x64"
            } else if cfg!(target_arch = "aarch64") {
                "arm64"
            } else {
                "unknown"
            }
            .into(),
        ),
        "pid" => Value::Integer(std::process::id() as i64),
        "hrtime" => Value::NativeFunction(c::PROCESS_HRTIME),
        "hrtime.bigint" => Value::NativeFunction(c::PROCESS_HRTIME_BIGINT),
        "nextTick" => Value::NativeFunction(c::PROCESS_NEXT_TICK),
        // API completeness additions (v0.5.0+).
        "kill" => Value::NativeFunction(c::PROCESS_KILL),
        "uptime" => Value::NativeFunction(c::PROCESS_UPTIME),
        "memoryUsage" => Value::NativeFunction(c::PROCESS_MEMORY_USAGE),
        "on" => Value::NativeFunction(c::PROCESS_ON),
    };

    // process.env
    let mut env_props = FxHashMap::default();
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
    let stdout_props = props! {
        "write" => Value::NativeFunction(c::PROCESS_STDOUT_WRITE),
    };
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
    let stderr_props = props! {
        "write" => Value::NativeFunction(c::PROCESS_STDOUT_WRITE),
    };
    let stderr_idx = gc.allocate(
        heap,
        HeapValue::Object(JsObject {
            properties: stderr_props,
            prototype: None,
            extensible: true,
        }),
    );
    props.insert("stderr".into(), Value::Object(stderr_idx));

    props
}

pub fn create_buffer_module(
    heap: &mut Vec<HeapValue>,
    gc: &mut GarbageCollector,
) -> FxHashMap<String, Value> {
    let buffer_proto_idx = gc.allocate(
        heap,
        HeapValue::Object(JsObject {
            properties: props! {
                "toString" => Value::NativeFunction(c::BUFFER_TO_STRING),
                "write" => Value::NativeFunction(c::BUFFER_WRITE),
                "slice" => Value::NativeFunction(c::BUFFER_SLICE),
                "copy" => Value::NativeFunction(c::BUFFER_COPY),
                "fill" => Value::NativeFunction(c::BUFFER_FILL),
                "compare" => Value::NativeFunction(c::BUFFER_COMPARE),
                "equals" => Value::NativeFunction(c::BUFFER_EQUALS),
                "indexOf" => Value::NativeFunction(c::BUFFER_INDEX_OF),
                "length" => Value::Integer(0),
            },
            prototype: None,
            extensible: true,
        }),
    );
    props! {
        "alloc" => Value::NativeFunction(c::BUFFER_ALLOC),
        "from" => Value::NativeFunction(c::BUFFER_FROM),
        "concat" => Value::NativeFunction(c::BUFFER_CONCAT),
        "isBuffer" => Value::NativeFunction(c::BUFFER_IS_BUFFER),
        "isEncoding" => Value::NativeFunction(c::BUFFER_IS_ENCODING),
        "byteLength" => Value::NativeFunction(c::BUFFER_BYTE_LENGTH),
        "transcode" => Value::NativeFunction(c::BUFFER_TRANSCODE),
        "toString" => Value::NativeFunction(c::BUFFER_TO_STRING),
        "write" => Value::NativeFunction(c::BUFFER_WRITE),
        "slice" => Value::NativeFunction(c::BUFFER_SLICE),
        "copy" => Value::NativeFunction(c::BUFFER_COPY),
        "fill" => Value::NativeFunction(c::BUFFER_FILL),
        "compare" => Value::NativeFunction(c::BUFFER_COMPARE),
        "equals" => Value::NativeFunction(c::BUFFER_EQUALS),
        "indexOf" => Value::NativeFunction(c::BUFFER_INDEX_OF),
        "prototype" => Value::Object(buffer_proto_idx),
    }
}

pub fn create_intl_module(
    heap: &mut Vec<HeapValue>,
    gc: &mut GarbageCollector,
) -> FxHashMap<String, Value> {
    let intl_obj_idx = gc.allocate(
        heap,
        HeapValue::Object(JsObject {
            properties: props! {
                "DateTimeFormat" => Value::NativeFunction(c::DATETIME_FORMAT_CONSTRUCTOR),
                "NumberFormat" => Value::NativeFunction(c::NUMBER_FORMAT_CONSTRUCTOR),
            },
            prototype: None,
            extensible: true,
        }),
    );
    props! {
        "default" => Value::Object(intl_obj_idx),
    }
}

pub fn create_events_module(
    heap: &mut Vec<HeapValue>,
    gc: &mut GarbageCollector,
) -> FxHashMap<String, Value> {
    let proto_idx = gc.allocate(
        heap,
        HeapValue::Object(JsObject {
            properties: props! {
                "on" => Value::NativeFunction(c::EVENT_EMITTER_ON),
                "emit" => Value::NativeFunction(c::EVENT_EMITTER_EMIT),
                "off" => Value::NativeFunction(c::EVENT_EMITTER_OFF),
                "listenerCount" => Value::NativeFunction(c::EVENT_EMITTER_LISTENER_COUNT),
            },
            prototype: None,
            extensible: true,
        }),
    );
    props! {
        "EventEmitter" => Value::NativeFunction(c::EVENT_EMITTER_CONSTRUCTOR),
        "prototype" => Value::Object(proto_idx),
    }
}

#[cfg(feature = "os")]
pub fn create_os_module(
    _heap: &mut Vec<HeapValue>,
    _gc: &mut GarbageCollector,
) -> FxHashMap<String, Value> {
    props! {
        "platform" => Value::NativeFunction(c::OS_PLATFORM),
        "arch" => Value::NativeFunction(c::OS_ARCH),
        "cpus" => Value::NativeFunction(c::OS_CPUS),
        "totalmem" => Value::NativeFunction(c::OS_TOTALMEM),
        "freemem" => Value::NativeFunction(c::OS_FREEMEM),
        "uptime" => Value::NativeFunction(c::OS_UPTIME),
        "hostname" => Value::NativeFunction(c::OS_HOSTNAME),
        "type" => Value::NativeFunction(c::OS_TYPE),
        "release" => Value::NativeFunction(c::OS_RELEASE),
        "homedir" => Value::NativeFunction(c::OS_HOMEDIR),
        "tmpdir" => Value::NativeFunction(c::OS_TMPDIR),
    }
}

pub fn create_crypto_module(
    _heap: &mut Vec<HeapValue>,
    _gc: &mut GarbageCollector,
) -> FxHashMap<String, Value> {
    props! {
        "randomBytes" => Value::NativeFunction(c::CRYPTO_RANDOM_BYTES),
        "randomUUID" => Value::NativeFunction(c::CRYPTO_RANDOM_UUID),
        "createHash" => Value::NativeFunction(c::CRYPTO_CREATE_HASH),
    }
}

pub fn create_assert_module(
    heap: &mut Vec<HeapValue>,
    gc: &mut GarbageCollector,
) -> FxHashMap<String, Value> {
    let assert_obj_idx = gc.allocate(
        heap,
        HeapValue::Object(JsObject {
            properties: props! {
                "strictEqual" => Value::NativeFunction(c::HEADERS_HAS),
                "ok" => Value::NativeFunction(c::HEADERS_SET),
                "equal" => Value::NativeFunction(c::HEADERS_HAS),
                "deepEqual" => Value::NativeFunction(c::HEADERS_HAS),
            },
            prototype: None,
            extensible: true,
        }),
    );
    props! {
        "default" => Value::Object(assert_obj_idx),
        "strictEqual" => Value::NativeFunction(c::HEADERS_HAS),
        "ok" => Value::NativeFunction(c::HEADERS_SET),
        "equal" => Value::NativeFunction(c::HEADERS_HAS),
        "deepEqual" => Value::NativeFunction(c::HEADERS_HAS),
    }
}

pub fn create_child_process_module(
    _heap: &mut Vec<HeapValue>,
    _gc: &mut GarbageCollector,
) -> FxHashMap<String, Value> {
    props! {
        "execSync" => Value::NativeFunction(c::CHILD_PROCESS_EXEC_SYNC),
        "exec" => Value::NativeFunction(c::CHILD_PROCESS_EXEC),
        "spawn" => Value::NativeFunction(c::CHILD_PROCESS_SPAWN),
    }
}

pub fn create_url_module(
    _heap: &mut Vec<HeapValue>,
    _gc: &mut GarbageCollector,
) -> FxHashMap<String, Value> {
    props! {
        "fileURLToPath" => Value::NativeFunction(c::URL_FILE_URL_TO_PATH),
    }
}

pub fn create_util_module(
    _heap: &mut Vec<HeapValue>,
    _gc: &mut GarbageCollector,
) -> FxHashMap<String, Value> {
    props! {
        "format" => Value::NativeFunction(c::UTIL_FORMAT),
        "inspect" => Value::NativeFunction(c::UTIL_INSPECT),
        "promisify" => Value::NativeFunction(c::UTIL_PROMISIFY),
        "callbackify" => Value::NativeFunction(c::UTIL_CALLBACKIFY),
    }
}

pub fn create_timers_module(
    _heap: &mut Vec<HeapValue>,
    _gc: &mut GarbageCollector,
) -> FxHashMap<String, Value> {
    props! {
        "setImmediate" => Value::NativeFunction(c::SET_IMMEDIATE),
        "clearImmediate" => Value::NativeFunction(c::CLEAR_IMMEDIATE),
    }
}

pub fn create_querystring_module(
    _heap: &mut Vec<HeapValue>,
    _gc: &mut GarbageCollector,
) -> FxHashMap<String, Value> {
    props! {
        "parse" => Value::NativeFunction(c::QUERYSTRING_PARSE),
        "stringify" => Value::NativeFunction(c::QUERYSTRING_STRINGIFY),
        "encode" => Value::NativeFunction(c::QUERYSTRING_ENCODE),
        "decode" => Value::NativeFunction(c::QUERYSTRING_DECODE),
    }
}

pub fn create_stream_module(
    heap: &mut Vec<HeapValue>,
    gc: &mut GarbageCollector,
) -> FxHashMap<String, Value> {
    let readable_proto_idx = gc.allocate(
        heap,
        HeapValue::Object(JsObject {
            properties: props! {
                "read" => Value::NativeFunction(c::STREAM_READABLE_READ),
                "pipe" => Value::NativeFunction(c::STREAM_READABLE_PIPE),
                "unpipe" => Value::NativeFunction(c::STREAM_READABLE_UNPIPE),
                "push" => Value::NativeFunction(c::STREAM_READABLE_PUSH),
                "destroy" => Value::NativeFunction(c::STREAM_READABLE_DESTROY),
            },
            prototype: None,
            extensible: true,
        }),
    );
    let writable_proto_idx = gc.allocate(
        heap,
        HeapValue::Object(JsObject {
            properties: props! {
                "write" => Value::NativeFunction(c::STREAM_WRITABLE_WRITE),
                "end" => Value::NativeFunction(c::STREAM_WRITABLE_END),
                "destroy" => Value::NativeFunction(c::STREAM_WRITABLE_DESTROY),
                "cork" => Value::NativeFunction(c::STREAM_WRITABLE_CORK),
                "uncork" => Value::NativeFunction(c::STREAM_WRITABLE_UNCORK),
            },
            prototype: None,
            extensible: true,
        }),
    );
    props! {
        "Readable" => Value::NativeFunction(c::STREAM_CONSTRUCTOR),
        "Writable" => Value::NativeFunction(c::STREAM_CONSTRUCTOR),
        "Transform" => Value::NativeFunction(c::STREAM_CONSTRUCTOR),
        "PassThrough" => Value::NativeFunction(c::STREAM_PASSTHROUGH_CONSTRUCTOR),
        "pipeline" => Value::NativeFunction(c::STREAM_PIPELINE),
        "finished" => Value::NativeFunction(c::STREAM_FINISHED),
        "readablePrototype" => Value::Object(readable_proto_idx),
        "writablePrototype" => Value::Object(writable_proto_idx),
    }
}

#[cfg(feature = "zlib")]
pub fn create_zlib_module(
    _heap: &mut Vec<HeapValue>,
    _gc: &mut GarbageCollector,
) -> FxHashMap<String, Value> {
    props! {
        "gzipSync" => Value::NativeFunction(c::ZLIB_GZIP_SYNC),
        "gunzipSync" => Value::NativeFunction(c::ZLIB_GUNZIP_SYNC),
        "deflateSync" => Value::NativeFunction(c::ZLIB_DEFLATE_SYNC),
        "inflateSync" => Value::NativeFunction(c::ZLIB_INFLATE_SYNC),
        "deflateRawSync" => Value::NativeFunction(c::ZLIB_DEFLATE_RAW_SYNC),
        "inflateRawSync" => Value::NativeFunction(c::ZLIB_INFLATE_RAW_SYNC),
        "gzip" => Value::NativeFunction(c::ZLIB_GZIP),
        "gunzip" => Value::NativeFunction(c::ZLIB_GUNZIP),
        "deflate" => Value::NativeFunction(c::ZLIB_DEFLATE),
        "inflate" => Value::NativeFunction(c::ZLIB_INFLATE),
    }
}

#[cfg(feature = "tls")]
pub fn create_tls_module(
    _heap: &mut Vec<HeapValue>,
    _gc: &mut GarbageCollector,
) -> FxHashMap<String, Value> {
    props! {
        "connect" => Value::NativeFunction(c::TLS_CONNECT),
        "createSecureContext" => Value::NativeFunction(c::TLS_CREATE_SECURE_CONTEXT),
        "TLSSocket" => Value::NativeFunction(c::TLS_SOCKET_WRITE),
        "createServer" => Value::NativeFunction(c::TLS_CREATE_SERVER),
    }
}

#[cfg(feature = "dns")]
pub fn create_dns_module(
    _heap: &mut Vec<HeapValue>,
    _gc: &mut GarbageCollector,
) -> FxHashMap<String, Value> {
    props! {
        "resolve" => Value::NativeFunction(c::DNS_RESOLVE),
        "lookup" => Value::NativeFunction(c::DNS_LOOKUP),
        "resolve4" => Value::NativeFunction(c::DNS_RESOLVE4),
        "resolve6" => Value::NativeFunction(c::DNS_RESOLVE6),
        "resolveMx" => Value::NativeFunction(c::DNS_RESOLVE_MX),
    }
}

#[cfg(feature = "http")]
pub fn create_http_module(
    _heap: &mut Vec<HeapValue>,
    _gc: &mut GarbageCollector,
) -> FxHashMap<String, Value> {
    props! {
        "createServer" => Value::NativeFunction(c::HTTP_CREATE_SERVER),
    }
}

#[cfg(feature = "net")]
pub fn create_net_module(
    _heap: &mut Vec<HeapValue>,
    _gc: &mut GarbageCollector,
) -> FxHashMap<String, Value> {
    props! {
        "createConnection" => Value::NativeFunction(c::NET_CREATE_CONNECTION),
    }
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
    // Also try replacing "/" with "-" for scoped names like "fs/promises" -> "fs-promises"
    let name_hyphen = name.replace('/', "-");

    // Try all combinations of name variants and extensions
    let name_variants = vec![name, &name_underscore, &name_hyphen];

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
