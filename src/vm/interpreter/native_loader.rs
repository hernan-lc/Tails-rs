use crate::objects::Value;
use std::collections::HashMap;

pub struct NativeModuleRegistry {
    modules: HashMap<String, Box<dyn Fn() -> HashMap<String, Value>>>,
}

impl NativeModuleRegistry {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }

    pub fn register<F>(&mut self, name: &str, factory: F)
    where
        F: Fn() -> HashMap<String, Value> + 'static,
    {
        self.modules.insert(name.to_string(), Box::new(factory));
    }

    pub fn has_module(&self, name: &str) -> bool {
        self.modules.contains_key(name)
    }

    pub fn load_module(
        &self,
        name: &str,
        _heap: &mut Vec<crate::vm::interpreter::HeapValue>,
        _gc: &mut crate::vm::gc::GarbageCollector,
    ) -> crate::errors::Result<HashMap<String, Value>> {
        if let Some(factory) = self.modules.get(name) {
            Ok(factory())
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

pub fn create_fs_module() -> HashMap<String, Value> {
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

pub fn create_path_module() -> HashMap<String, Value> {
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
