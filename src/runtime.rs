use crate::compiler::type_checker::Type;
use crate::compiler::Compiler;
use crate::errors::Result;
use crate::objects::Value;
use crate::vm::Interpreter;
use std::collections::HashMap;
use std::path::Path;

#[derive(Default)]
pub struct RuntimeConfig {
    pub enable_type_checking: bool,
    pub max_heap_size: usize,
}

pub struct TailsRuntime {
    interpreter: Interpreter,
    config: RuntimeConfig,
}

impl TailsRuntime {
    pub fn new(config: RuntimeConfig) -> Result<Self> {
        let interpreter = Interpreter::new()?;
        Ok(Self {
            interpreter,
            config,
        })
    }

    pub fn eval(&mut self, source: &str) -> Result<Value> {
        let mut compiler = Compiler::new(self.config.enable_type_checking);
        // Pass known globals to the compiler for type checking
        if self.config.enable_type_checking {
            let globals: HashMap<String, Type> = self
                .interpreter
                .globals
                .keys()
                .map(|k| (k.clone(), Type::Any))
                .collect();
            compiler.set_known_globals(globals);
        }
        let compiled = compiler.compile(source)?;
        self.interpreter.execute(&compiled)
    }

    pub fn eval_module(&mut self, source: &str, base_path: &Path) -> Result<Value> {
        let module_key = base_path.to_string_lossy().to_string();
        let prev = self.interpreter.current_module_path.clone();
        self.interpreter.current_module_path = Some(module_key.clone());
        let compiler = Compiler::new(self.config.enable_type_checking);
        let compiled = compiler.compile(source)?;
        let result = self.interpreter.execute_module(&compiled);
        // Register module exports
        let exports = std::mem::take(&mut self.interpreter.module_exports);
        // If module result is Undefined but there's a default export, return that instead
        let final_result = if matches!(result, Ok(Value::Undefined)) {
            exports.get("default").cloned().unwrap_or(Value::Undefined)
        } else {
            result?
        };
        self.interpreter.module_registry.insert(module_key, exports);
        self.interpreter.current_module_path = prev;
        Ok(final_result)
    }

    pub fn import(&mut self, module_path: &Path) -> Result<Value> {
        let source = std::fs::read_to_string(module_path).map_err(|e| {
            crate::errors::Error::RuntimeError(format!("Failed to read module: {}", e))
        })?;
        self.eval_module(&source, module_path)
    }

    pub fn get_global(&self, name: &str) -> Option<Value> {
        self.interpreter.get_global(name)
    }

    pub fn set_global(&mut self, name: &str, value: Value) {
        self.interpreter.set_global(name, value);
    }

    pub fn get_module_export(&self, module_path: &str, name: &str) -> Option<Value> {
        self.interpreter
            .module_registry
            .get(module_path)
            .and_then(|exports| exports.get(name).cloned())
    }

    pub fn new_object(&mut self) -> Value {
        self.interpreter.new_object()
    }

    pub fn new_array(&mut self) -> Value {
        self.interpreter.new_array()
    }

    pub fn get_property(&mut self, object: &Value, key: &str) -> Option<Value> {
        self.interpreter.get_property_str(object, key)
    }

    pub fn set_property(&mut self, object: &Value, key: &str, value: Value) {
        self.interpreter.set_property_str(object, key, value);
    }

    pub fn get_array_length(&self, array: &Value) -> Option<i64> {
        self.interpreter.get_array_length(array)
    }

    pub fn get_array_element(&self, array: &Value, index: usize) -> Option<Value> {
        self.interpreter.get_array_element(array, index)
    }

    pub fn push_array_element(&mut self, array: &Value, value: Value) {
        self.interpreter.push_array_element(array, value);
    }

    pub fn call_function(&mut self, func: &Value, this: &Value, args: &[Value]) -> Result<Value> {
        self.interpreter.call_value(func, this, args)
    }

    pub fn call_global(&mut self, name: &str, args: &[Value]) -> Result<Value> {
        let func = self.get_global(name).ok_or_else(|| {
            crate::errors::Error::RuntimeError(format!("Function '{}' not found in globals", name))
        })?;
        self.call_function(&func, &Value::Undefined, args)
    }
}

impl Default for TailsRuntime {
    fn default() -> Self {
        Self::new(RuntimeConfig::default()).expect("Failed to create default runtime")
    }
}
