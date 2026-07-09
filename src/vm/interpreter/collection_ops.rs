//! Shared Map/Set bytecode helpers — one implementation for both the hot
//! dispatch path (`bytecode.rs`) and the cascading op table (`ops.rs`).

use super::{HeapValue, Interpreter};
use crate::errors::Result;
use crate::objects::Value;
use crate::well_known as wk;

impl Interpreter {
    /// Call `this[method](...args)` where `method` is a well-known name.
    #[inline]
    pub(crate) fn call_named_method(
        &mut self,
        this: &Value,
        method: &str,
        args: &[Value],
    ) -> Result<Value> {
        let f = self.get_property(this, &Value::string(method))?;
        self.call_value(&f, this, args)
    }

    /// `map.set(key, value)` or generic `.set` fallback. Pushes the result.
    pub(crate) fn exec_map_set(&mut self, object: Value, key: Value, value: Value) -> Result<()> {
        match object {
            Value::Map(map_idx) => {
                if let HeapValue::Map(map) = &mut self.heap[map_idx] {
                    map.set(key, value);
                }
                self.stack.push(Value::Map(map_idx));
            }
            other => {
                let result = self.call_named_method(&other, wk::SET_PROP, &[key, value])?;
                self.stack.push(result);
            }
        }
        Ok(())
    }

    /// `map.set` with a variable argument list (cold path for non-2-arg forms).
    pub(crate) fn exec_map_set_args(&mut self, object: Value, args: &[Value]) -> Result<()> {
        let key = args.first().cloned().unwrap_or(Value::Undefined);
        let value = args.get(1).cloned().unwrap_or(Value::Undefined);
        match object {
            Value::Map(map_idx) => {
                if let HeapValue::Map(map) = &mut self.heap[map_idx] {
                    map.set(key, value);
                }
                self.stack.push(Value::Map(map_idx));
            }
            other => {
                let result = self.call_named_method(&other, wk::SET_PROP, args)?;
                self.stack.push(result);
            }
        }
        Ok(())
    }

    /// `map.get(key)` or generic `.get` fallback. Pushes the result.
    pub(crate) fn exec_map_get(&mut self, object: Value, key: Value) -> Result<()> {
        match object {
            Value::Map(map_idx) => {
                let result = if let HeapValue::Map(map) = &self.heap[map_idx] {
                    map.get(&key).cloned().unwrap_or(Value::Undefined)
                } else {
                    Value::Undefined
                };
                self.stack.push(result);
            }
            other => {
                let result = self.call_named_method(&other, wk::GET, &[key])?;
                self.stack.push(result);
            }
        }
        Ok(())
    }

    /// `map|set.has(key)`. Pushes a boolean.
    pub(crate) fn exec_collection_has(&mut self, object: Value, key: Value) -> Result<()> {
        let result = match object {
            Value::Map(map_idx) => {
                if let HeapValue::Map(map) = &self.heap[map_idx] {
                    Value::Boolean(map.has(&key))
                } else {
                    Value::Boolean(false)
                }
            }
            Value::Set(set_idx) => {
                if let HeapValue::Set(set) = &self.heap[set_idx] {
                    Value::Boolean(set.has(&key))
                } else {
                    Value::Boolean(false)
                }
            }
            other => self.call_named_method(&other, wk::HAS, &[key])?,
        };
        self.stack.push(result);
        Ok(())
    }

    /// `map|set.delete(key)`. Pushes a boolean.
    pub(crate) fn exec_collection_delete(&mut self, object: Value, key: Value) -> Result<()> {
        let result = match object {
            Value::Map(map_idx) => {
                if let HeapValue::Map(map) = &mut self.heap[map_idx] {
                    Value::Boolean(map.delete(&key))
                } else {
                    Value::Boolean(false)
                }
            }
            Value::Set(set_idx) => {
                if let HeapValue::Set(set) = &mut self.heap[set_idx] {
                    Value::Boolean(set.delete(&key))
                } else {
                    Value::Boolean(false)
                }
            }
            other => self.call_named_method(&other, wk::DELETE, &[key])?,
        };
        self.stack.push(result);
        Ok(())
    }

    /// `set.add(value)`. Pushes the set (or method result).
    pub(crate) fn exec_set_add(&mut self, object: Value, value: Value) -> Result<()> {
        match object {
            Value::Set(set_idx) => {
                if let HeapValue::Set(set) = &mut self.heap[set_idx] {
                    set.add(value);
                }
                self.stack.push(Value::Set(set_idx));
            }
            other => {
                let result = self.call_named_method(&other, wk::ADD, &[value])?;
                self.stack.push(result);
            }
        }
        Ok(())
    }

    /// Advance a generic iterator via `.next()` and decode `{ done, value }`.
    /// Returns `Ok(Some(value))` when there is another item, `Ok(None)` when done.
    pub(crate) fn iterator_next_value(&mut self, iterator: &Value) -> Result<Option<Value>> {
        let next_fn = self.get_property(iterator, &Value::string(wk::NEXT))?;
        let next_result = self.call_value(&next_fn, iterator, &[])?;

        // Fast path: generator-style objects expose `done`/`value` as own props.
        if let Value::Object(obj_idx) = &next_result {
            if let HeapValue::Object(obj) = &self.heap[*obj_idx] {
                if let Some(done_val) = obj.properties.get(wk::DONE) {
                    if matches!(done_val, Value::Boolean(true)) {
                        return Ok(None);
                    }
                    if let Some(value) = obj.properties.get(wk::VALUE) {
                        return Ok(Some(value.clone()));
                    }
                }
            }
        }

        let done = self.get_property(&next_result, &Value::string(wk::DONE))?;
        if matches!(done, Value::Boolean(true)) {
            return Ok(None);
        }
        let value = self.get_property(&next_result, &Value::string(wk::VALUE))?;
        Ok(Some(value))
    }
}
