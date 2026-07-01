use std::collections::HashMap;

use crate::{NativeFn, NativeValue};

pub struct TailsModule {
    name: String,
    functions: HashMap<String, NativeFn>,
    constants: HashMap<String, NativeValue>,
}

impl TailsModule {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            functions: HashMap::new(),
            constants: HashMap::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn export_function(mut self, name: &str, func: NativeFn) -> Self {
        self.functions.insert(name.to_string(), func);
        self
    }

    pub fn export_constant(mut self, name: &str, value: NativeValue) -> Self {
        self.constants.insert(name.to_string(), value);
        self
    }

    pub fn export_object(mut self, name: &str, _props: HashMap<String, NativeValue>) -> Self {
        let obj = crate::object_new();
        self.constants.insert(name.to_string(), obj);
        self
    }

    pub fn into_exports(self) -> HashMap<String, NativeValue> {
        let mut exports = HashMap::new();
        for (name, func) in &self.functions {
            let value = NativeValue {
                tag: 10,
                data: *func as usize as u64,
            };
            exports.insert(name.clone(), value);
        }
        for (name, value) in &self.constants {
            exports.insert(name.clone(), *value);
        }
        exports
    }
}

pub struct NativeModuleExport {
    pub name: String,
    pub value: NativeValue,
}

impl NativeModuleExport {
    pub fn function(name: &str, func: NativeFn) -> Self {
        Self {
            name: name.to_string(),
            value: NativeValue {
                tag: 10,
                data: func as usize as u64,
            },
        }
    }

    pub fn string(name: &str, value: &str) -> Self {
        Self {
            name: name.to_string(),
            value: crate::string(value),
        }
    }

    pub fn number(name: &str, value: f64) -> Self {
        Self {
            name: name.to_string(),
            value: crate::number(value),
        }
    }

    pub fn boolean(name: &str, value: bool) -> Self {
        Self {
            name: name.to_string(),
            value: crate::boolean(value),
        }
    }
}
