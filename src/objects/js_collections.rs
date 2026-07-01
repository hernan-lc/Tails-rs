use crate::objects::Value;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub struct JsMap {
    pub entries: Vec<(Value, Value)>,
}

impl JsMap {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn get(&self, key: &Value) -> Option<&Value> {
        for (k, v) in &self.entries {
            if k == key {
                return Some(v);
            }
        }
        None
    }

    pub fn set(&mut self, key: Value, value: Value) {
        for (k, v) in &mut self.entries {
            if k == &key {
                *v = value;
                return;
            }
        }
        self.entries.push((key, value));
    }

    pub fn has(&self, key: &Value) -> bool {
        self.entries.iter().any(|(k, _)| k == key)
    }

    pub fn delete(&mut self, key: &Value) -> bool {
        let len = self.entries.len();
        self.entries.retain(|(k, _)| k != key);
        self.entries.len() < len
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn size(&self) -> usize {
        self.entries.len()
    }

    pub fn keys(&self) -> Vec<Value> {
        self.entries.iter().map(|(k, _)| k.clone()).collect()
    }

    pub fn values(&self) -> Vec<Value> {
        self.entries.iter().map(|(_, v)| v.clone()).collect()
    }

    pub fn entries(&self) -> Vec<(Value, Value)> {
        self.entries.clone()
    }

    pub fn for_each(&self, f: &mut dyn FnMut(&Value, &Value)) {
        for (k, v) in &self.entries {
            f(k, v);
        }
    }
}

impl Default for JsMap {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct JsSet {
    pub values: Vec<Value>,
}

impl JsSet {
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    pub fn add(&mut self, value: Value) {
        if !self.values.iter().any(|v| v == &value) {
            self.values.push(value);
        }
    }

    pub fn has(&self, value: &Value) -> bool {
        self.values.iter().any(|v| v == value)
    }

    pub fn delete(&mut self, value: &Value) -> bool {
        let len = self.values.len();
        self.values.retain(|v| v != value);
        self.values.len() < len
    }

    pub fn clear(&mut self) {
        self.values.clear();
    }

    pub fn size(&self) -> usize {
        self.values.len()
    }

    pub fn values(&self) -> Vec<Value> {
        self.values.clone()
    }

    pub fn for_each(&self, f: &mut dyn FnMut(&Value)) {
        for v in &self.values {
            f(v);
        }
    }
}

impl Default for JsSet {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct JsWeakMap {
    pub entries: Vec<(usize, Value)>,
}

impl JsWeakMap {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn get(&self, key: &Value) -> Option<&Value> {
        if let Value::Object(idx) = key {
            for (k, v) in &self.entries {
                if *k == *idx {
                    return Some(v);
                }
            }
        }
        None
    }

    pub fn set(&mut self, key: Value, value: Value) {
        if let Value::Object(idx) = key {
            for (k, v) in &mut self.entries {
                if *k == idx {
                    *v = value;
                    return;
                }
            }
            self.entries.push((idx, value));
        }
    }

    pub fn has(&self, key: &Value) -> bool {
        if let Value::Object(idx) = key {
            self.entries.iter().any(|(k, _)| k == idx)
        } else {
            false
        }
    }

    pub fn delete(&mut self, key: &Value) -> bool {
        if let Value::Object(idx) = key {
            let len = self.entries.len();
            self.entries.retain(|(k, _)| k != idx);
            self.entries.len() < len
        } else {
            false
        }
    }
}

impl Default for JsWeakMap {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct JsWeakSet {
    pub values: Vec<usize>,
}

impl JsWeakSet {
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    pub fn add(&mut self, value: Value) {
        if let Value::Object(idx) = value {
            if !self.values.contains(&idx) {
                self.values.push(idx);
            }
        }
    }

    pub fn has(&self, value: &Value) -> bool {
        if let Value::Object(idx) = value {
            self.values.contains(idx)
        } else {
            false
        }
    }

    pub fn delete(&mut self, value: &Value) -> bool {
        if let Value::Object(idx) = value {
            let len = self.values.len();
            self.values.retain(|v| v != idx);
            self.values.len() < len
        } else {
            false
        }
    }
}

impl Default for JsWeakSet {
    fn default() -> Self {
        Self::new()
    }
}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Value::Undefined => 0.hash(state),
            Value::Null => 1.hash(state),
            Value::Boolean(b) => b.hash(state),
            Value::Integer(i) => i.hash(state),
            Value::Float(f) => f.to_bits().hash(state),
            Value::String(s) => s.hash(state),
            Value::BigInt(i) => i.hash(state),
            Value::Symbol(id) => id.hash(state),
            Value::Function(i) => i.hash(state),
            Value::NativeFunction(i) => i.hash(state),
            Value::Object(i) => i.hash(state),
            Value::Array(i) => i.hash(state),
            Value::Promise(i) => i.hash(state),
            Value::Proxy(i) => i.hash(state),
            Value::Generator(i) => i.hash(state),
            Value::TypedArray(i) => i.hash(state),
            Value::Map(i) => i.hash(state),
            Value::Set(i) => i.hash(state),
            Value::WeakMap(i) => i.hash(state),
            Value::WeakSet(i) => i.hash(state),
            Value::Date(_) | Value::RegExp(_) | Value::Buffer(_) | Value::NativeObject(_) => {
                todo!()
            }
        }
    }
}
