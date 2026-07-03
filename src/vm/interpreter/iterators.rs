use super::*;
use crate::errors::{Error, Result};
use crate::objects::js_promise::PromiseState;
use crate::objects::Value;
use crate::props;
use crate::runtime_env::native_fns::constants as c;

impl Interpreter {
    fn make_builtin_iter_props() -> FxHashMap<String, Value> {
        props! {
            "__type" => Value::String("array".to_string()),
            "__index" => Value::Integer(0),
            "map" => Value::NativeFunction(c::ITERATOR_MAP),
            "filter" => Value::NativeFunction(c::ITERATOR_FILTER),
            "take" => Value::NativeFunction(c::ITERATOR_TAKE),
            "drop" => Value::NativeFunction(c::ITERATOR_DROP),
            "forEach" => Value::NativeFunction(c::ITERATOR_FOR_EACH),
            "toArray" => Value::NativeFunction(c::ITERATOR_TO_ARRAY),
        }
    }

    pub(crate) fn exec_get_iterator(&mut self, iterable: Value) -> Result<Value> {
        match &iterable {
            Value::Array(arr_idx) => {
                let elements = if let HeapValue::Array(arr) = &self.heap[*arr_idx] {
                    arr.elements.clone()
                } else {
                    Vec::new()
                };
                let data_idx = self
                    .gc
                    .allocate(&mut self.heap, HeapValue::Array(JsArray { elements }));
                let mut props = Self::make_builtin_iter_props();
                props.insert("__data".to_string(), Value::Array(data_idx));
                let iter_idx = self.gc.allocate(
                    &mut self.heap,
                    HeapValue::Object(JsObject {
                        properties: props,
                        prototype: None,
                        extensible: true,
                    }),
                );
                Ok(Value::Object(iter_idx))
            }
            Value::String(s) => {
                let chars: Vec<Value> = s.chars().map(|c| Value::String(c.to_string())).collect();
                let data_idx = self.gc.allocate(
                    &mut self.heap,
                    HeapValue::Array(JsArray { elements: chars }),
                );
                let mut props = Self::make_builtin_iter_props();
                props.insert("__data".to_string(), Value::Array(data_idx));
                props.insert("__type".to_string(), Value::String("string".to_string()));
                let iter_idx = self.gc.allocate(
                    &mut self.heap,
                    HeapValue::Object(JsObject {
                        properties: props,
                        prototype: None,
                        extensible: true,
                    }),
                );
                Ok(Value::Object(iter_idx))
            }
            Value::Cons(c) => {
                let flat = c.flatten();
                let chars: Vec<Value> = flat
                    .chars()
                    .map(|ch| Value::String(ch.to_string()))
                    .collect();
                let data_idx = self.gc.allocate(
                    &mut self.heap,
                    HeapValue::Array(JsArray { elements: chars }),
                );
                let mut props = Self::make_builtin_iter_props();
                props.insert("__data".to_string(), Value::Array(data_idx));
                props.insert("__type".to_string(), Value::String("string".to_string()));
                let iter_idx = self.gc.allocate(
                    &mut self.heap,
                    HeapValue::Object(JsObject {
                        properties: props,
                        prototype: None,
                        extensible: true,
                    }),
                );
                Ok(Value::Object(iter_idx))
            }
            // Phase 4A — Lazy Map iterator: store a reference to the Map heap
            // entry instead of cloning `keys.clone()` + `values.clone()` and
            // allocating an `[k, v]` pair array for every entry. With 50K
            // entries this saves 2 × 50K × 32-byte clones + 50K heap
            // allocations per `for…of m` loop. The iterator reads from the
            // Map's `keys` / `values` vecs directly on each `next()`. The
            // Map stays alive through the GC because the iterator's
            // `__target` property holds a `Value::Map(map_idx)`.
            Value::Map(map_idx) => {
                let mut props = Self::make_builtin_iter_props();
                props.insert("__type".to_string(), Value::String("map".to_string()));
                props.insert("__index".to_string(), Value::Integer(0));
                props.insert("__target".to_string(), Value::Map(*map_idx));
                let iter_idx = self.gc.allocate(
                    &mut self.heap,
                    HeapValue::Object(JsObject {
                        properties: props,
                        prototype: None,
                        extensible: true,
                    }),
                );
                Ok(Value::Object(iter_idx))
            }
            // Phase 4A — Lazy Set iterator: same as Map, but with `values` only.
            Value::Set(set_idx) => {
                let mut props = Self::make_builtin_iter_props();
                props.insert("__type".to_string(), Value::String("set".to_string()));
                props.insert("__index".to_string(), Value::Integer(0));
                props.insert("__target".to_string(), Value::Set(*set_idx));
                let iter_idx = self.gc.allocate(
                    &mut self.heap,
                    HeapValue::Object(JsObject {
                        properties: props,
                        prototype: None,
                        extensible: true,
                    }),
                );
                Ok(Value::Object(iter_idx))
            }
            _ => {
                let iterator_symbol = Value::Symbol(crate::objects::SYMBOL_ITERATOR);
                let iterator_fn = self.get_property(&iterable, &iterator_symbol)?;
                match iterator_fn {
                    Value::Function(_) | Value::NativeFunction(_) => {
                        let iterator = self.call_value(&iterator_fn, &iterable, &[])?;
                        Ok(iterator)
                    }
                    _ => Err(Error::TypeError(
                        "Value is not iterable (no Symbol.iterator method)".into(),
                    )),
                }
            }
        }
    }

    pub(crate) fn exec_get_async_iterator(&mut self, iterable: Value) -> Result<Value> {
        match &iterable {
            Value::Array(arr_idx) => {
                let elements = if let HeapValue::Array(arr) = &self.heap[*arr_idx] {
                    arr.elements.clone()
                } else {
                    Vec::new()
                };
                let data_idx = self
                    .gc
                    .allocate(&mut self.heap, HeapValue::Array(JsArray { elements }));
                let mut props = Self::make_builtin_iter_props();
                props.insert("__data".to_string(), Value::Array(data_idx));
                let iter_idx = self.gc.allocate(
                    &mut self.heap,
                    HeapValue::Object(JsObject {
                        properties: props,
                        prototype: None,
                        extensible: true,
                    }),
                );
                Ok(Value::Object(iter_idx))
            }
            Value::String(s) => {
                let chars: Vec<Value> = s.chars().map(|c| Value::String(c.to_string())).collect();
                let data_idx = self.gc.allocate(
                    &mut self.heap,
                    HeapValue::Array(JsArray { elements: chars }),
                );
                let mut props = Self::make_builtin_iter_props();
                props.insert("__data".to_string(), Value::Array(data_idx));
                props.insert("__type".to_string(), Value::String("string".to_string()));
                let iter_idx = self.gc.allocate(
                    &mut self.heap,
                    HeapValue::Object(JsObject {
                        properties: props,
                        prototype: None,
                        extensible: true,
                    }),
                );
                Ok(Value::Object(iter_idx))
            }
            Value::Cons(c) => {
                let flat = c.flatten();
                let chars: Vec<Value> = flat
                    .chars()
                    .map(|ch| Value::String(ch.to_string()))
                    .collect();
                let data_idx = self.gc.allocate(
                    &mut self.heap,
                    HeapValue::Array(JsArray { elements: chars }),
                );
                let mut props = Self::make_builtin_iter_props();
                props.insert("__data".to_string(), Value::Array(data_idx));
                props.insert("__type".to_string(), Value::String("string".to_string()));
                let iter_idx = self.gc.allocate(
                    &mut self.heap,
                    HeapValue::Object(JsObject {
                        properties: props,
                        prototype: None,
                        extensible: true,
                    }),
                );
                Ok(Value::Object(iter_idx))
            }
            _ => {
                let async_iter_symbol = Value::Symbol(crate::objects::SYMBOL_ASYNC_ITERATOR);
                let async_iter_fn = self.get_property(&iterable, &async_iter_symbol)?;
                let iterator_fn =
                    if matches!(async_iter_fn, Value::Function(_) | Value::NativeFunction(_)) {
                        async_iter_fn
                    } else {
                        let iterator_symbol = Value::Symbol(crate::objects::SYMBOL_ITERATOR);
                        self.get_property(&iterable, &iterator_symbol)?
                    };
                match iterator_fn {
                    Value::Function(_) | Value::NativeFunction(_) => {
                        let iterator = self.call_value(&iterator_fn, &iterable, &[])?;
                        Ok(iterator)
                    }
                    _ => Err(Error::TypeError("Value is not async iterable".into())),
                }
            }
        }
    }

    pub(crate) fn exec_iterator_next(
        &mut self,
        iterator: Value,
        target: usize,
    ) -> Result<ControlFlowOutcome> {
        if let Value::Object(iter_idx) = &iterator {
            if let HeapValue::Object(iter_obj) = &self.heap[*iter_idx] {
                if let Some(Value::String(iter_type)) = iter_obj.properties.get("__type") {
                    let index = match iter_obj.properties.get("__index") {
                        Some(Value::Integer(i)) => *i as usize,
                        _ => 0,
                    };
                    if let Some(data_val) = iter_obj.properties.get("__data") {
                        match (iter_type.as_str(), data_val) {
                            ("array", Value::Array(arr_idx)) => {
                                if let HeapValue::Array(arr) = &self.heap[*arr_idx] {
                                    if index >= arr.elements.len() {
                                        return Ok(ControlFlowOutcome::Jump(target));
                                    }
                                    let value = arr.elements[index].clone();
                                    if let HeapValue::Object(iter_obj_mut) =
                                        &mut self.heap[*iter_idx]
                                    {
                                        iter_obj_mut.properties.insert(
                                            "__index".to_string(),
                                            Value::Integer((index + 1) as i64),
                                        );
                                    }
                                    self.stack.push(value);
                                }
                            }
                            ("string", Value::Array(chars_idx)) => {
                                if let HeapValue::Array(chars_arr) = &self.heap[*chars_idx] {
                                    if index >= chars_arr.elements.len() {
                                        return Ok(ControlFlowOutcome::Jump(target));
                                    }
                                    let value = chars_arr.elements[index].clone();
                                    if let HeapValue::Object(iter_obj_mut) =
                                        &mut self.heap[*iter_idx]
                                    {
                                        iter_obj_mut.properties.insert(
                                            "__index".to_string(),
                                            Value::Integer((index + 1) as i64),
                                        );
                                    }
                                    self.stack.push(value);
                                }
                            }
                            _ => {}
                        }
                        return Ok(ControlFlowOutcome::Next);
                    }
                    // Phase 4A — Lazy Map/Set iterators: read directly from
                    // the Map/Set heap entry indexed by `__target`. No clones
                    // of the key/value vecs, no `[k, v]` pair allocations
                    // ahead of time — the pair is built on the stack on
                    // each `next()`. The Map/Set stays alive through the
                    // GC because the iterator's `__target` property holds
                    // a `Value::Map` / `Value::Set(idx)` reference.
                    if let Some(target_val) = iter_obj.properties.get("__target") {
                        match (iter_type.as_str(), target_val) {
                            ("map", Value::Map(map_idx)) => {
                                if let HeapValue::Map(m) = &self.heap[*map_idx] {
                                    if index >= m.keys.len() {
                                        return Ok(ControlFlowOutcome::Jump(target));
                                    }
                                    let pair_idx = self.heap.len();
                                    self.heap.push(HeapValue::Array(JsArray {
                                        elements: vec![
                                            m.keys[index].clone(),
                                            m.values[index].clone(),
                                        ],
                                    }));
                                    self.stack.push(Value::Array(pair_idx));
                                }
                                if let HeapValue::Object(iter_obj_mut) = &mut self.heap[*iter_idx] {
                                    iter_obj_mut.properties.insert(
                                        "__index".to_string(),
                                        Value::Integer((index + 1) as i64),
                                    );
                                }
                                return Ok(ControlFlowOutcome::Next);
                            }
                            ("set", Value::Set(set_idx)) => {
                                if let HeapValue::Set(s) = &self.heap[*set_idx] {
                                    if index >= s.values.len() {
                                        return Ok(ControlFlowOutcome::Jump(target));
                                    }
                                    self.stack.push(s.values[index].clone());
                                }
                                if let HeapValue::Object(iter_obj_mut) = &mut self.heap[*iter_idx] {
                                    iter_obj_mut.properties.insert(
                                        "__index".to_string(),
                                        Value::Integer((index + 1) as i64),
                                    );
                                }
                                return Ok(ControlFlowOutcome::Next);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        let next_fn = self.get_property(&iterator, &Value::String("next".to_string()))?;
        let next_result = self.call_value(&next_fn, &iterator, &[])?;
        // OPTIMIZATION (Phase 6C): for generator results, extract `done` and
        // `value` directly from the JsObject properties without re-doing
        // get_property (which would allocate 2 more strings "done" and
        // "value" and walk the prototype chain). The native generator
        // implementation always returns an Object with these two keys.
        if let Value::Object(obj_idx) = &next_result {
            if let HeapValue::Object(obj) = &self.heap[*obj_idx] {
                if let Some(done_val) = obj.properties.get("done") {
                    if matches!(done_val, Value::Boolean(true)) {
                        return Ok(ControlFlowOutcome::Jump(target));
                    }
                    if let Some(value) = obj.properties.get("value") {
                        self.stack.push(value.clone());
                        return Ok(ControlFlowOutcome::Next);
                    }
                }
            }
        }
        // Slow path: fall back to the original property-access logic for
        // custom iterators that don't follow the generator protocol.
        let done = self.get_property(&next_result, &Value::String("done".to_string()))?;
        match done {
            Value::Boolean(true) => Ok(ControlFlowOutcome::Jump(target)),
            _ => {
                let value = self.get_property(&next_result, &Value::String("value".to_string()))?;
                self.stack.push(value);
                Ok(ControlFlowOutcome::Next)
            }
        }
    }

    pub(crate) fn exec_async_iterator_next(
        &mut self,
        iterator: Value,
        target: usize,
    ) -> Result<ControlFlowOutcome> {
        if let Value::Object(iter_idx) = &iterator {
            let iter_idx = *iter_idx;
            if let HeapValue::Object(ref iter_obj) = self.heap[iter_idx] {
                if let Some(Value::String(_iter_type)) = iter_obj.properties.get("__type") {
                    let index = match iter_obj.properties.get("__index") {
                        Some(Value::Integer(i)) => *i as usize,
                        _ => 0,
                    };
                    if let Some(data_val) = iter_obj.properties.get("__data").cloned() {
                        let done = match &data_val {
                            Value::Array(arr_idx) => {
                                if let HeapValue::Array(arr) = &self.heap[*arr_idx] {
                                    index >= arr.elements.len()
                                } else {
                                    true
                                }
                            }
                            _ => true,
                        };
                        if done {
                            return Ok(ControlFlowOutcome::Jump(target));
                        }
                        let value = match &data_val {
                            Value::Array(arr_idx) => {
                                if let HeapValue::Array(arr) = &self.heap[*arr_idx] {
                                    arr.elements[index].clone()
                                } else {
                                    Value::Undefined
                                }
                            }
                            _ => Value::Undefined,
                        };
                        if let HeapValue::Object(ref mut obj) = self.heap[iter_idx] {
                            obj.properties
                                .insert("__index".to_string(), Value::Integer((index + 1) as i64));
                        }
                        let awaited_value = Self::resolve_value_promise(&self.heap, &value);
                        self.stack.push(awaited_value);
                        return Ok(ControlFlowOutcome::Next);
                    }
                }
            }
        }

        let next_fn = self.get_property(&iterator, &Value::String("next".to_string()))?;
        let next_result = self.call_value(&next_fn, &iterator, &[])?;
        let done = self.get_property(&next_result, &Value::String("done".to_string()))?;
        match done {
            Value::Boolean(true) => Ok(ControlFlowOutcome::Jump(target)),
            _ => {
                let value = self.get_property(&next_result, &Value::String("value".to_string()))?;
                let awaited_value = Self::resolve_value_promise(&self.heap, &value);
                self.stack.push(awaited_value);
                Ok(ControlFlowOutcome::Next)
            }
        }
    }

    pub(crate) fn exec_iterator_close(&mut self, iterator: Value) -> Result<()> {
        if let Ok(return_fn) = self.get_property(&iterator, &Value::String("return".to_string())) {
            if matches!(return_fn, Value::Function(_) | Value::NativeFunction(_)) {
                let _ = self.call_value(&return_fn, &iterator, &[]);
            }
        }
        Ok(())
    }

    fn resolve_value_promise(heap: &[HeapValue], value: &Value) -> Value {
        if let Value::Promise(promise_idx) = value {
            if let HeapValue::Promise(p) = &heap[*promise_idx] {
                return match &p.state {
                    PromiseState::Fulfilled(v) => v.clone(),
                    PromiseState::Rejected(_) => Value::Undefined,
                    PromiseState::Pending => value.clone(),
                };
            }
        }
        value.clone()
    }
}
