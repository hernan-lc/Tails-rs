use crate::objects::Value;
use rustc_hash::FxHashMap;
use rustc_hash::FxHashSet;

/// Ordered Map with a specialized integer-key fast path.
///
/// `Value` is ~96 bytes (dominated by `ConsString`), so using it directly as
/// an `FxHashMap` key causes pathological cache behaviour under growth.
/// Integer keys (the common case for `Map` benchmarks and numeric loops) go
/// through a dense `FxHashMap<i64, usize>` instead.
#[derive(Debug, Clone)]
pub struct JsMap {
    /// Integer-key → index into `keys`/`values`.
    int_map: FxHashMap<i64, usize>,
    /// Non-integer keys → index.
    map: FxHashMap<Value, usize>,
    pub(crate) keys: Vec<Value>,
    pub(crate) values: Vec<Value>,
}

/// Classify a value as an integer map key when it is an exact integer
/// (SameValueZero-friendly for Integer/whole Float).
#[inline]
fn as_int_key(key: &Value) -> Option<i64> {
    match key {
        Value::Integer(i) => Some(*i),
        Value::Float(f) if f.is_finite() && *f == (*f as i64) as f64 && *f != -0.0 => {
            Some(*f as i64)
        }
        // +0.0 / -0.0 → 0 under SameValueZero
        Value::Float(f) if *f == 0.0 => Some(0),
        _ => None,
    }
}

impl JsMap {
    pub fn new() -> Self {
        Self {
            int_map: FxHashMap::default(),
            map: FxHashMap::default(),
            keys: Vec::new(),
            values: Vec::new(),
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            int_map: FxHashMap::with_capacity_and_hasher(cap, Default::default()),
            map: FxHashMap::with_capacity_and_hasher(cap / 4 + 1, Default::default()),
            keys: Vec::with_capacity(cap),
            values: Vec::with_capacity(cap),
        }
    }

    #[inline]
    pub fn get(&self, key: &Value) -> Option<&Value> {
        if let Some(i) = as_int_key(key) {
            return self.int_map.get(&i).map(|&idx| &self.values[idx]);
        }
        self.map.get(key).map(|&idx| &self.values[idx])
    }

    #[inline]
    pub fn set(&mut self, key: Value, value: Value) {
        if let Some(i) = as_int_key(&key) {
            if let Some(&idx) = self.int_map.get(&i) {
                self.values[idx] = value;
                return;
            }
            let idx = self.keys.len();
            self.int_map.insert(i, idx);
            self.keys.push(key);
            self.values.push(value);
            return;
        }
        if let Some(&idx) = self.map.get(&key) {
            self.values[idx] = value;
            return;
        }
        let idx = self.keys.len();
        self.map.insert(key.clone(), idx);
        self.keys.push(key);
        self.values.push(value);
    }

    #[inline]
    pub fn has(&self, key: &Value) -> bool {
        if let Some(i) = as_int_key(key) {
            return self.int_map.contains_key(&i);
        }
        self.map.contains_key(key)
    }

    pub fn delete(&mut self, key: &Value) -> bool {
        let idx = if let Some(i) = as_int_key(key) {
            match self.int_map.remove(&i) {
                Some(idx) => idx,
                None => return false,
            }
        } else {
            match self.map.remove(key) {
                Some(idx) => idx,
                None => return false,
            }
        };

        let last = self.keys.len() - 1;
        if idx != last {
            self.keys.swap(idx, last);
            self.values.swap(idx, last);
            self.keys.pop();
            self.values.pop();
            // Fix index of the swapped-in key
            let moved = &self.keys[idx];
            if let Some(i) = as_int_key(moved) {
                if let Some(slot) = self.int_map.get_mut(&i) {
                    *slot = idx;
                }
            } else if let Some(slot) = self.map.get_mut(moved) {
                *slot = idx;
            }
        } else {
            self.keys.pop();
            self.values.pop();
        }
        true
    }

    pub fn clear(&mut self) {
        self.int_map.clear();
        self.map.clear();
        self.keys.clear();
        self.values.clear();
    }

    #[inline]
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
    int_set: FxHashSet<i64>,
    set: FxHashSet<Value>,
    pub(crate) values: Vec<Value>,
}

impl JsSet {
    pub fn new() -> Self {
        Self {
            int_set: FxHashSet::default(),
            set: FxHashSet::default(),
            values: Vec::new(),
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            int_set: FxHashSet::with_capacity_and_hasher(cap, Default::default()),
            set: FxHashSet::with_capacity_and_hasher(cap / 4 + 1, Default::default()),
            values: Vec::with_capacity(cap),
        }
    }

    #[inline]
    pub fn add(&mut self, value: Value) {
        if let Some(i) = as_int_key(&value) {
            if self.int_set.insert(i) {
                self.values.push(value);
            }
            return;
        }
        if self.set.insert(value.clone()) {
            self.values.push(value);
        }
    }

    #[inline]
    pub fn has(&self, value: &Value) -> bool {
        if let Some(i) = as_int_key(value) {
            return self.int_set.contains(&i);
        }
        self.set.contains(value)
    }

    pub fn delete(&mut self, value: &Value) -> bool {
        let removed = if let Some(i) = as_int_key(value) {
            self.int_set.remove(&i)
        } else {
            self.set.remove(value)
        };
        if removed {
            if let Some(idx) = self.values.iter().position(|v| v == value) {
                let last = self.values.len() - 1;
                if idx != last {
                    self.values.swap(idx, last);
                }
                self.values.pop();
            }
            true
        } else {
            false
        }
    }

    pub fn clear(&mut self) {
        self.int_set.clear();
        self.set.clear();
        self.values.clear();
    }

    #[inline]
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
    map: FxHashMap<usize, Value>,
}

impl JsWeakMap {
    pub fn new() -> Self {
        Self {
            map: FxHashMap::default(),
        }
    }

    pub fn get(&self, key: &Value) -> Option<&Value> {
        if let Value::Object(idx) = key {
            self.map.get(idx)
        } else {
            None
        }
    }

    pub fn set(&mut self, key: Value, value: Value) {
        if let Value::Object(idx) = key {
            self.map.insert(idx, value);
        }
    }

    pub fn has(&self, key: &Value) -> bool {
        if let Value::Object(idx) = key {
            self.map.contains_key(idx)
        } else {
            false
        }
    }

    pub fn delete(&mut self, key: &Value) -> bool {
        if let Value::Object(idx) = key {
            self.map.remove(idx).is_some()
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
    values: FxHashSet<usize>,
}

impl JsWeakSet {
    pub fn new() -> Self {
        Self {
            values: FxHashSet::default(),
        }
    }

    pub fn add(&mut self, value: Value) {
        if let Value::Object(idx) = value {
            self.values.insert(idx);
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
            self.values.remove(idx)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer_keys_roundtrip() {
        let mut m = JsMap::new();
        for i in 0..1000i64 {
            m.set(Value::Integer(i), Value::Integer(i * 2));
        }
        assert_eq!(m.size(), 1000);
        assert_eq!(m.get(&Value::Integer(42)), Some(&Value::Integer(84)));
        // Whole floats share the int fast-path (SameValueZero).
        assert_eq!(m.get(&Value::Float(42.0)), Some(&Value::Integer(84)));
        assert!(m.delete(&Value::Integer(42)));
        assert!(!m.has(&Value::Integer(42)));
        assert_eq!(m.size(), 999);
    }
}
