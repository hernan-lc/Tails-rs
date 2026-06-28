use crate::objects::Value;
use crate::vm::gc::GarbageCollector;
use crate::vm::interpreter::{HeapValue, JsObject};
use std::collections::HashMap;

pub struct NativeModuleRegistry {
    modules: HashMap<String, Box<dyn Fn(&mut Vec<HeapValue>, &mut GarbageCollector) -> HashMap<String, Value>>>,
}

impl NativeModuleRegistry {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }

    pub fn register<F>(&mut self, name: &str, factory: F)
    where
        F: Fn(&mut Vec<HeapValue>, &mut GarbageCollector) -> HashMap<String, Value> + 'static,
    {
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
        "fs" => registry.register("fs", |heap, gc| create_fs_module(heap, gc)),
        "path" => registry.register("path", |heap, gc| create_path_module(heap, gc)),
        "process" => registry.register("process", |heap, gc| create_process_module(heap, gc)),
        "buffer" => registry.register("buffer", |heap, gc| create_buffer_module(heap, gc)),
        "intl" => registry.register("intl", |heap, gc| create_intl_module(heap, gc)),
        "events" => registry.register("events", |heap, gc| create_events_module(heap, gc)),
        "os" => registry.register("os", |heap, gc| create_os_module(heap, gc)),
        "crypto" => registry.register("crypto", |heap, gc| create_crypto_module(heap, gc)),
        _ => {}
    }
}

pub fn create_fs_module(_heap: &mut Vec<HeapValue>, _gc: &mut GarbageCollector) -> HashMap<String, Value> {
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

pub fn create_path_module(_heap: &mut Vec<HeapValue>, _gc: &mut GarbageCollector) -> HashMap<String, Value> {
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
    props.insert(
        "pid".into(),
        Value::Integer(std::process::id() as i64),
    );

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
    props.insert(
        "EventEmitter".into(),
        Value::NativeFunction(322),
    );

    // Prototype methods
    let mut proto_props = HashMap::new();
    proto_props.insert("on".into(), Value::NativeFunction(323));
    proto_props.insert("emit".into(), Value::NativeFunction(324));
    proto_props.insert("off".into(), Value::NativeFunction(325));
    proto_props.insert("listenerCount".into(), Value::NativeFunction(326));

    let proto_idx = gc.allocate(
        heap,
        HeapValue::Object(JsObject {
            properties: proto_props,
            prototype: None,
            extensible: true,
        }),
    );
    props.insert(
        "prototype".into(),
        Value::Object(proto_idx),
    );

    props
}

pub fn create_os_module(
    _heap: &mut Vec<HeapValue>,
    _gc: &mut GarbageCollector,
) -> HashMap<String, Value> {
    let mut props = HashMap::new();
    props.insert("platform".into(), Value::NativeFunction(329));
    props.insert("arch".into(), Value::NativeFunction(330));
    props.insert("cpus".into(), Value::NativeFunction(331));
    props.insert("totalmem".into(), Value::NativeFunction(332));
    props.insert("freemem".into(), Value::NativeFunction(333));
    props.insert("uptime".into(), Value::NativeFunction(334));
    props.insert("hostname".into(), Value::NativeFunction(335));
    props.insert("type".into(), Value::NativeFunction(336));
    props.insert("release".into(), Value::NativeFunction(337));
    props.insert("homedir".into(), Value::NativeFunction(338));
    props.insert("tmpdir".into(), Value::NativeFunction(339));
    props
}

pub fn create_crypto_module(
    _heap: &mut Vec<HeapValue>,
    _gc: &mut GarbageCollector,
) -> HashMap<String, Value> {
    let mut props = HashMap::new();
    props.insert("randomBytes".into(), Value::NativeFunction(340));
    props.insert("randomUUID".into(), Value::NativeFunction(341));
    props.insert("createHash".into(), Value::NativeFunction(342));
    props
}
