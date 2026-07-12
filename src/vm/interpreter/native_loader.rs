use crate::objects::Value;
use crate::props;
use crate::runtime_env::native_fns::constants as c;
use crate::vm::gc::GarbageCollector;
use crate::vm::interpreter::{HeapValue, JsObject, PropertyStorage};
use crate::well_known as wk;
use rustc_hash::FxHashMap;

type NativeModuleFactory = fn(&mut Vec<HeapValue>, &mut GarbageCollector) -> PropertyStorage;

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
    ) -> crate::errors::Result<PropertyStorage> {
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
        wk::MOD_FS => registry.register(wk::MOD_FS, create_fs_module),
        #[cfg(feature = "fs")]
        wk::MOD_FS_PROMISES => registry.register(wk::MOD_FS_PROMISES, create_fs_promises_module),
        #[cfg(feature = "path")]
        wk::MOD_PATH => registry.register(wk::MOD_PATH, create_path_module),
        #[cfg(feature = "process")]
        wk::MOD_PROCESS => registry.register(wk::MOD_PROCESS, create_process_module),
        wk::MOD_BUFFER => registry.register(wk::MOD_BUFFER, create_buffer_module),
        "intl" => registry.register("intl", create_intl_module),
        wk::MOD_EVENTS => registry.register(wk::MOD_EVENTS, create_events_module),
        #[cfg(feature = "os")]
        wk::MOD_OS => registry.register(wk::MOD_OS, create_os_module),
        wk::MOD_CRYPTO => registry.register(wk::MOD_CRYPTO, create_crypto_module),
        wk::MOD_ASSERT => registry.register(wk::MOD_ASSERT, create_assert_module),
        "child_process" => registry.register("child_process", create_child_process_module),
        wk::MOD_URL => registry.register(wk::MOD_URL, create_url_module),
        wk::MOD_UTIL => registry.register(wk::MOD_UTIL, create_util_module),
        wk::MOD_TIMERS => registry.register(wk::MOD_TIMERS, create_timers_module),
        wk::MOD_QUERYSTRING => registry.register(wk::MOD_QUERYSTRING, create_querystring_module),
        wk::MOD_STREAM => registry.register(wk::MOD_STREAM, create_stream_module),
        "tty" => registry.register("tty", create_tty_module),
        #[cfg(feature = "zlib")]
        wk::MOD_ZLIB => registry.register(wk::MOD_ZLIB, create_zlib_module),
        #[cfg(feature = "tls")]
        wk::MOD_TLS => registry.register(wk::MOD_TLS, create_tls_module),
        #[cfg(feature = "dns")]
        wk::MOD_DNS => registry.register(wk::MOD_DNS, create_dns_module),
        #[cfg(feature = "http")]
        wk::MOD_HTTP => registry.register(wk::MOD_HTTP, create_http_module),
        #[cfg(feature = "net")]
        wk::MOD_NET => registry.register(wk::MOD_NET, create_net_module),
        _ => {}
    }
}

#[cfg(feature = "fs")]
pub fn create_fs_module(_heap: &mut Vec<HeapValue>, _gc: &mut GarbageCollector) -> PropertyStorage {
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
) -> PropertyStorage {
    props! {
        "readFile" => Value::NativeFunction(c::FS_READ_FILE),
        "readdir" => Value::NativeFunction(c::FS_READDIR),
        "read_file" => Value::NativeFunction(c::FS_READ_FILE),
        "writeFile" => Value::NativeFunction(c::FS_WRITE_FILE),
        "write_file" => Value::NativeFunction(c::FS_WRITE_FILE),
        "stat" => Value::NativeFunction(c::FS_STAT),
        "mkdir" => Value::NativeFunction(c::FS_MKDIR),
        "unlink" => Value::NativeFunction(c::FS_UNLINK),
        "copyFile" => Value::NativeFunction(c::FS_COPY_FILE),
        "copy_file" => Value::NativeFunction(c::FS_COPY_FILE),
        "rename" => Value::NativeFunction(c::FS_RENAME),
    }
}

#[cfg(feature = "path")]
pub fn create_path_module(
    _heap: &mut Vec<HeapValue>,
    _gc: &mut GarbageCollector,
) -> PropertyStorage {
    props! {
        "join" => Value::NativeFunction(c::PATH_JOIN),
        "resolve" => Value::NativeFunction(c::PATH_RESOLVE),
        "basename" => Value::NativeFunction(c::PATH_BASENAME),
        "dirname" => Value::NativeFunction(c::PATH_DIRNAME),
        "extname" => Value::NativeFunction(c::PATH_EXTNAME),
        "relative" => Value::NativeFunction(c::PATH_RELATIVE),
        "isAbsolute" => Value::NativeFunction(c::PATH_IS_ABSOLUTE),
        "normalize" => Value::NativeFunction(c::PATH_NORMALIZE),
        "sep" => Value::from_string(std::path::MAIN_SEPARATOR.to_string()),
        "delimiter" => Value::from_string(if cfg!(target_os = "windows") {
                ";"
            } else {
                ":"
            }
            .to_string(),),
    }
}

#[cfg(feature = "process")]
pub fn create_process_module(
    heap: &mut Vec<HeapValue>,
    gc: &mut GarbageCollector,
) -> PropertyStorage {
    // Start with the static set of `process.<name>` properties (function
    // pointers, scalar compile-time constants, and the `pid`). The
    // dynamic sub-objects (env / argv / stdout / stderr) are merged in
    // after the heap allocations below.
    let mut props = props! {
        "exit" => Value::NativeFunction(c::PROCESS_EXIT),
        "cwd" => Value::NativeFunction(c::PROCESS_CWD),
        "chdir" => Value::NativeFunction(c::PROCESS_CHDIR),
        "platform" => Value::from_string(if cfg!(target_os = "linux") {
                "linux"
            } else if cfg!(target_os = "macos") {
                "darwin"
            } else if cfg!(target_os = "windows") {
                "win32"
            } else {
                "unknown"
            }
            .into(),),
        "arch" => Value::from_string(if cfg!(target_arch = "x86_64") {
                "x64"
            } else if cfg!(target_arch = "aarch64") {
                "arm64"
            } else {
                "unknown"
            }
            .into(),),
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
    let mut env_props = PropertyStorage::new();
    for (key, value) in std::env::vars() {
        env_props.insert(key, Value::from_string(value));
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
    let argv: Vec<Value> = std::env::args().map(Value::from_string).collect();
    let argv_idx = gc.allocate(
        heap,
        HeapValue::Array(crate::vm::interpreter::JsArray { elements: argv }),
    );
    props.insert("argv".into(), Value::Array(argv_idx));

    // process.stdout — Node exposes `.fd` (1) used by `tty.isatty(process.stdout.fd)`.
    let stdout_props = props! {
        "write" => Value::NativeFunction(c::PROCESS_STDOUT_WRITE),
        "fd" => Value::Integer(1),
        "isTTY" => Value::Boolean(atty_stdout()),
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

    // process.stderr — `.fd` is 2 (Node convention).
    let stderr_props = props! {
        "write" => Value::NativeFunction(c::PROCESS_STDOUT_WRITE),
        "fd" => Value::Integer(2),
        "isTTY" => Value::Boolean(atty_stderr()),
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

fn atty_stdout() -> bool {
    use std::io::IsTerminal;
    std::io::stdout().is_terminal()
}

fn atty_stderr() -> bool {
    use std::io::IsTerminal;
    std::io::stderr().is_terminal()
}

/// Minimal Node-compatible `tty` module (`isatty`).
pub fn create_tty_module(
    _heap: &mut Vec<HeapValue>,
    _gc: &mut GarbageCollector,
) -> PropertyStorage {
    props! {
        "isatty" => Value::NativeFunction(c::TTY_ISATTY),
    }
}

pub fn create_buffer_module(
    heap: &mut Vec<HeapValue>,
    gc: &mut GarbageCollector,
) -> PropertyStorage {
    let buffer_proto_idx = gc.allocate(
        heap,
        HeapValue::Object(JsObject {
            properties: props! {
                wk::TO_STRING => Value::NativeFunction(c::BUFFER_TO_STRING),
                "write" => Value::NativeFunction(c::BUFFER_WRITE),
                "slice" => Value::NativeFunction(c::BUFFER_SLICE),
                "copy" => Value::NativeFunction(c::BUFFER_COPY),
                "fill" => Value::NativeFunction(c::BUFFER_FILL),
                "compare" => Value::NativeFunction(c::BUFFER_COMPARE),
                "equals" => Value::NativeFunction(c::BUFFER_EQUALS),
                "indexOf" => Value::NativeFunction(c::BUFFER_INDEX_OF),
                wk::LENGTH => Value::Integer(0),
            },
            prototype: None,
            extensible: true,
        }),
    );
    // Buffer constructor object (Buffer.from, Buffer.alloc, Buffer.prototype, …)
    let buffer_ctor_props = props! {
        "alloc" => Value::NativeFunction(c::BUFFER_ALLOC),
        "from" => Value::NativeFunction(c::BUFFER_FROM),
        "concat" => Value::NativeFunction(c::BUFFER_CONCAT),
        "isBuffer" => Value::NativeFunction(c::BUFFER_IS_BUFFER),
        "isEncoding" => Value::NativeFunction(c::BUFFER_IS_ENCODING),
        "byteLength" => Value::NativeFunction(c::BUFFER_BYTE_LENGTH),
        "transcode" => Value::NativeFunction(c::BUFFER_TRANSCODE),
        wk::TO_STRING => Value::NativeFunction(c::BUFFER_TO_STRING),
        "write" => Value::NativeFunction(c::BUFFER_WRITE),
        "slice" => Value::NativeFunction(c::BUFFER_SLICE),
        "copy" => Value::NativeFunction(c::BUFFER_COPY),
        "fill" => Value::NativeFunction(c::BUFFER_FILL),
        "compare" => Value::NativeFunction(c::BUFFER_COMPARE),
        "equals" => Value::NativeFunction(c::BUFFER_EQUALS),
        "indexOf" => Value::NativeFunction(c::BUFFER_INDEX_OF),
        wk::PROTOTYPE => Value::Object(buffer_proto_idx),
    };
    let buffer_ctor_idx = gc.allocate(
        heap,
        HeapValue::Object(JsObject {
            properties: buffer_ctor_props,
            prototype: None,
            extensible: true,
        }),
    );
    // Node's `require('buffer')` is `{ Buffer, ... }` — not the ctor itself.
    // safer-buffer does `var Buffer = require('buffer').Buffer`.
    props! {
        "Buffer" => Value::Object(buffer_ctor_idx),
        // Also re-export statics at top level for callers that treat the
        // module as the constructor (legacy / our global Buffer).
        "alloc" => Value::NativeFunction(c::BUFFER_ALLOC),
        "from" => Value::NativeFunction(c::BUFFER_FROM),
        "concat" => Value::NativeFunction(c::BUFFER_CONCAT),
        "isBuffer" => Value::NativeFunction(c::BUFFER_IS_BUFFER),
        "isEncoding" => Value::NativeFunction(c::BUFFER_IS_ENCODING),
        "byteLength" => Value::NativeFunction(c::BUFFER_BYTE_LENGTH),
        wk::PROTOTYPE => Value::Object(buffer_proto_idx),
    }
}

pub fn create_intl_module(heap: &mut Vec<HeapValue>, gc: &mut GarbageCollector) -> PropertyStorage {
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
) -> PropertyStorage {
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
        wk::PROTOTYPE => Value::Object(proto_idx),
    }
}

#[cfg(feature = "os")]
pub fn create_os_module(_heap: &mut Vec<HeapValue>, _gc: &mut GarbageCollector) -> PropertyStorage {
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
) -> PropertyStorage {
    props! {
        "randomBytes" => Value::NativeFunction(c::CRYPTO_RANDOM_BYTES),
        "randomUUID" => Value::NativeFunction(c::CRYPTO_RANDOM_UUID),
        "createHash" => Value::NativeFunction(c::CRYPTO_CREATE_HASH),
    }
}

pub fn create_assert_module(
    heap: &mut Vec<HeapValue>,
    gc: &mut GarbageCollector,
) -> PropertyStorage {
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
) -> PropertyStorage {
    props! {
        "execSync" => Value::NativeFunction(c::CHILD_PROCESS_EXEC_SYNC),
        "exec" => Value::NativeFunction(c::CHILD_PROCESS_EXEC),
        "spawn" => Value::NativeFunction(c::CHILD_PROCESS_SPAWN),
    }
}

pub fn create_url_module(
    _heap: &mut Vec<HeapValue>,
    _gc: &mut GarbageCollector,
) -> PropertyStorage {
    props! {
        "fileURLToPath" => Value::NativeFunction(c::URL_FILE_URL_TO_PATH),
    }
}

pub fn create_util_module(
    _heap: &mut Vec<HeapValue>,
    _gc: &mut GarbageCollector,
) -> PropertyStorage {
    props! {
        "format" => Value::NativeFunction(c::UTIL_FORMAT),
        "inspect" => Value::NativeFunction(c::UTIL_INSPECT),
        "promisify" => Value::NativeFunction(c::UTIL_PROMISIFY),
        "callbackify" => Value::NativeFunction(c::UTIL_CALLBACKIFY),
        "deprecate" => Value::NativeFunction(c::UTIL_DEPRECATE),
        "inherits" => Value::NativeFunction(c::UTIL_INHERITS),
    }
}

pub fn create_timers_module(
    _heap: &mut Vec<HeapValue>,
    _gc: &mut GarbageCollector,
) -> PropertyStorage {
    props! {
        "setTimeout" => Value::NativeFunction(c::SET_TIMEOUT),
        "clearTimeout" => Value::NativeFunction(c::CLEAR_TIMEOUT),
        "setInterval" => Value::NativeFunction(c::SET_INTERVAL),
        "clearInterval" => Value::NativeFunction(c::CLEAR_INTERVAL),
        "setImmediate" => Value::NativeFunction(c::SET_IMMEDIATE),
        "clearImmediate" => Value::NativeFunction(c::CLEAR_IMMEDIATE),
    }
}

pub fn create_querystring_module(
    _heap: &mut Vec<HeapValue>,
    _gc: &mut GarbageCollector,
) -> PropertyStorage {
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
) -> PropertyStorage {
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
    // Transform/PassThrough expose both readable and writable methods (Node Duplex-like).
    let transform_proto_idx = gc.allocate(
        heap,
        HeapValue::Object(JsObject {
            properties: props! {
                "read" => Value::NativeFunction(c::STREAM_READABLE_READ),
                "pipe" => Value::NativeFunction(c::STREAM_READABLE_PIPE),
                "unpipe" => Value::NativeFunction(c::STREAM_READABLE_UNPIPE),
                "push" => Value::NativeFunction(c::STREAM_READABLE_PUSH),
                "destroy" => Value::NativeFunction(c::STREAM_READABLE_DESTROY),
                "write" => Value::NativeFunction(c::STREAM_WRITABLE_WRITE),
                "end" => Value::NativeFunction(c::STREAM_WRITABLE_END),
                "cork" => Value::NativeFunction(c::STREAM_WRITABLE_CORK),
                "uncork" => Value::NativeFunction(c::STREAM_WRITABLE_UNCORK),
            },
            prototype: None,
            extensible: true,
        }),
    );
    // Constructor objects so `Transform.prototype` works (iconv-lite, etc.).
    // They remain constructible via NativeFunction stored under a private key
    // looked up by Construct / property_access, OR we expose them as
    // NativeFunctions and serve `.prototype` from property_access using the
    // prototype objects stored here.
    props! {
        "Readable" => Value::NativeFunction(c::STREAM_CONSTRUCTOR),
        "Writable" => Value::NativeFunction(c::STREAM_CONSTRUCTOR),
        "Transform" => Value::NativeFunction(c::STREAM_CONSTRUCTOR),
        "PassThrough" => Value::NativeFunction(c::STREAM_PASSTHROUGH_CONSTRUCTOR),
        "pipeline" => Value::NativeFunction(c::STREAM_PIPELINE),
        "finished" => Value::NativeFunction(c::STREAM_FINISHED),
        "readablePrototype" => Value::Object(readable_proto_idx),
        "writablePrototype" => Value::Object(writable_proto_idx),
        "transformPrototype" => Value::Object(transform_proto_idx),
        // Node-compatible: also surface as the constructors' .prototype via
        // find_native_prototype / property_access (see STREAM_CONSTRUCTOR).
        wk::PROTOTYPE => Value::Object(transform_proto_idx),
    }
}

#[cfg(feature = "zlib")]
pub fn create_zlib_module(
    _heap: &mut Vec<HeapValue>,
    _gc: &mut GarbageCollector,
) -> PropertyStorage {
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
) -> PropertyStorage {
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
) -> PropertyStorage {
    props! {
        "resolve" => Value::NativeFunction(c::DNS_RESOLVE),
        "lookup" => Value::NativeFunction(c::DNS_LOOKUP),
        "resolve4" => Value::NativeFunction(c::DNS_RESOLVE4),
        "resolve6" => Value::NativeFunction(c::DNS_RESOLVE6),
        "resolveMx" => Value::NativeFunction(c::DNS_RESOLVE_MX),
    }
}

#[cfg(feature = "http")]
pub fn create_http_module(heap: &mut Vec<HeapValue>, gc: &mut GarbageCollector) -> PropertyStorage {
    // Node.js `http.METHODS` — used by Express (`var { METHODS } = require('node:http')`).
    const METHODS: &[&str] = &[
        "ACL",
        "BIND",
        "CHECKOUT",
        "CONNECT",
        "COPY",
        "DELETE",
        "GET",
        "HEAD",
        "LINK",
        "LOCK",
        "M-SEARCH",
        "MERGE",
        "MKACTIVITY",
        "MKCALENDAR",
        "MKCOL",
        "MOVE",
        "NOTIFY",
        "OPTIONS",
        "PATCH",
        "POST",
        "PROPFIND",
        "PROPPATCH",
        "PURGE",
        "PUT",
        "REBIND",
        "REPORT",
        "SEARCH",
        "SOURCE",
        "SUBSCRIBE",
        "TRACE",
        "UNBIND",
        "UNLINK",
        "UNLOCK",
        "UNSUBSCRIBE",
    ];
    let method_vals: Vec<Value> = METHODS
        .iter()
        .map(|m| Value::from_string((*m).to_string()))
        .collect();
    let methods_idx = gc.allocate(
        heap,
        HeapValue::Array(crate::vm::interpreter::JsArray {
            elements: method_vals,
        }),
    );
    // Stub constructors so Express can do
    // `Object.create(http.IncomingMessage.prototype)`.
    let make_ctor = |heap: &mut Vec<HeapValue>, gc: &mut GarbageCollector| {
        let proto_idx = gc.allocate(
            heap,
            HeapValue::Object(JsObject {
                properties: props! {
                    // Header accessors Express expects on http.ServerResponse.prototype.
                    // The runtime's per-request `res` already defines these as own
                    // properties (so they take precedence), but listing them here keeps
                    // the prototype chain faithful to Node's http module.
                    "setHeader" => Value::NativeFunction(c::HTTP_RES_SET_HEADER),
                    "getHeader" => Value::NativeFunction(c::HTTP_RES_GET_HEADER),
                    "removeHeader" => Value::NativeFunction(c::HTTP_RES_REMOVE_HEADER),
                },
                prototype: None,
                extensible: true,
            }),
        );
        let ctor_idx = gc.allocate(
            heap,
            HeapValue::Object(JsObject {
                properties: props! {
                    wk::PROTOTYPE => Value::Object(proto_idx),
                },
                prototype: None,
                extensible: true,
            }),
        );
        Value::Object(ctor_idx)
    };
    let incoming = make_ctor(heap, gc);
    let server_response = make_ctor(heap, gc);
    props! {
        "createServer" => Value::NativeFunction(c::HTTP_CREATE_SERVER),
        "METHODS" => Value::Array(methods_idx),
        "IncomingMessage" => incoming,
        "ServerResponse" => server_response,
    }
}

#[cfg(feature = "net")]
pub fn create_net_module(
    _heap: &mut Vec<HeapValue>,
    _gc: &mut GarbageCollector,
) -> PropertyStorage {
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
