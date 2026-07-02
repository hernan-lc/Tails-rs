use crate::objects::Value;
use rustc_hash::FxHashMap;

pub struct Builtins {
    pub globals: FxHashMap<String, Value>,
}

impl Builtins {
    pub fn new() -> Self {
        let mut globals = FxHashMap::default();

        // Math constants
        globals.insert("Math".to_string(), Value::Undefined);

        // Console (simplified)
        globals.insert("console".to_string(), Value::Undefined);

        // JSON
        globals.insert("JSON".to_string(), Value::Undefined);

        // Promise
        globals.insert("Promise".to_string(), Value::Undefined);

        // Array
        globals.insert("Array".to_string(), Value::Undefined);

        // Object
        globals.insert("Object".to_string(), Value::Undefined);

        // String
        globals.insert("String".to_string(), Value::Undefined);

        // Number
        globals.insert("Number".to_string(), Value::Undefined);

        // Boolean
        globals.insert("Boolean".to_string(), Value::Undefined);

        // parseInt
        globals.insert("parseInt".to_string(), Value::Undefined);

        // parseFloat
        globals.insert("parseFloat".to_string(), Value::Undefined);

        // isNaN
        globals.insert("isNaN".to_string(), Value::Undefined);

        // isFinite
        globals.insert("isFinite".to_string(), Value::Undefined);

        Self { globals }
    }
}

impl Default for Builtins {
    fn default() -> Self {
        Self::new()
    }
}
