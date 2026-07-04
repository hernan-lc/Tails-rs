use crate::objects::Value;
use rustc_hash::FxHashMap;
use rustc_hash::FxHashSet;

/// Swap-remove `item` from `vec` in O(1). Returns whether the item was found.
fn swap_remove_vec<T: PartialEq>(vec: &mut Vec<T>, item: &T) -> bool {
    if let Some(idx) = vec.iter().position(|v| v == item) {
        let last = vec.len() - 1;
        if idx != last {
            vec.swap(idx, last);
        }
        vec.pop();
        true
    } else {
        false
    }
}

#[derive(Debug, Clone)]
pub struct JsMap {
    map: FxHashMap<Value, usize>,
    pub(crate) keys: Vec<Value>,
    pub(crate) values: Vec<Value>,
}

impl JsMap {
    pub fn new() -> Self {
        Self {
            map: FxHashMap::default(),
            keys: Vec::new(),
            values: Vec::new(),
        }
    }

    pub fn get(&self, key: &Value) -> Option<&Value> {
        self.map.get(key).map(|&idx| &self.values[idx])
    }

    pub fn set(&mut self, key: Value, value: Value) {
        if let Some(&idx) = self.map.get(&key) {
            self.values[idx] = value;
        } else {
            let idx = self.keys.len();
            self.map.insert(key.clone(), idx);
            self.keys.push(key);
            self.values.push(value);
        }
    }

    pub fn has(&self, key: &Value) -> bool {
        self.map.contains_key(key)
    }

    pub fn delete(&mut self, key: &Value) -> bool {
        if let Some(idx) = self.map.remove(key) {
            let last = self.keys.len() - 1;
            if idx != last {
                self.keys.swap(idx, last);
                self.values.swap(idx, last);
                self.keys.pop();
                self.values.pop();
                // Update index of key that was moved from last to idx
                let moved_key = self.keys[idx].clone();
                *self.map.get_mut(&moved_key).unwrap() = idx;
            } else {
                self.keys.pop();
                self.values.pop();
            }
            true
        } else {
            false
        }
    }

    pub fn clear(&mut self) {
        self.map.clear();
        self.keys.clear();
        self.values.clear();
    }

    pub fn size(&self) -> usize {
        self.keys.len()
    }

    pub fn keys(&self) -> Vec<Value> {
        self.keys.clone()
    }

    pub fn values(&self) -> Vec<Value> {
        self.values.clone()
    }

    pub fn entries(&self) -> Vec<(Value, Value)> {
        self.keys
            .iter()
            .zip(self.values.iter())
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    pub fn for_each(&self, f: &mut dyn FnMut(&Value, &Value)) {
        for (k, v) in self.keys.iter().zip(self.values.iter()) {
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
    set: FxHashSet<Value>,
    pub(crate) values: Vec<Value>,
}

impl JsSet {
    pub fn new() -> Self {
        Self {
            set: FxHashSet::default(),
            values: Vec::new(),
        }
    }

    pub fn add(&mut self, value: Value) {
        if self.set.insert(value.clone()) {
            self.values.push(value);
        }
    }

    pub fn has(&self, value: &Value) -> bool {
        self.set.contains(value)
    }

    pub fn delete(&mut self, value: &Value) -> bool {
        if self.set.remove(value) {
            swap_remove_vec(&mut self.values, value)
        } else {
            false
        }
    }

    pub fn clear(&mut self) {
        self.set.clear();
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


