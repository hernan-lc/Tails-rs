use super::*;
use crate::errors::{Error, Result};
use crate::objects::js_promise::PromiseState;
use crate::objects::Value;

impl Interpreter {
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
                let iter_idx = self.gc.allocate(
                    &mut self.heap,
                    HeapValue::Iterator(JsIterator {
                        kind: "array".to_string(),
                        index: 0,
                        target: None,
                        data: Some(Value::Array(data_idx)),
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
                let iter_idx = self.gc.allocate(
                    &mut self.heap,
                    HeapValue::Iterator(JsIterator {
                        kind: "string".to_string(),
                        index: 0,
                        target: None,
                        data: Some(Value::Array(data_idx)),
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
                let iter_idx = self.gc.allocate(
                    &mut self.heap,
                    HeapValue::Iterator(JsIterator {
                        kind: "string".to_string(),
                        index: 0,
                        target: None,
                        data: Some(Value::Array(data_idx)),
                    }),
                );
                Ok(Value::Object(iter_idx))
            }
            Value::Map(map_idx) => {
                let iter_idx = self.gc.allocate(
                    &mut self.heap,
                    HeapValue::Iterator(JsIterator {
                        kind: "map".to_string(),
                        index: 0,
                        target: Some(Value::Map(*map_idx)),
                        data: None,
                    }),
                );
                Ok(Value::Object(iter_idx))
            }
            Value::Set(set_idx) => {
                let iter_idx = self.gc.allocate(
                    &mut self.heap,
                    HeapValue::Iterator(JsIterator {
                        kind: "set".to_string(),
                        index: 0,
                        target: Some(Value::Set(*set_idx)),
                        data: None,
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
                let iter_idx = self.gc.allocate(
                    &mut self.heap,
                    HeapValue::Iterator(JsIterator {
                        kind: "array".to_string(),
                        index: 0,
                        target: None,
                        data: Some(Value::Array(data_idx)),
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
                let iter_idx = self.gc.allocate(
                    &mut self.heap,
                    HeapValue::Iterator(JsIterator {
                        kind: "string".to_string(),
                        index: 0,
                        target: None,
                        data: Some(Value::Array(data_idx)),
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
                let iter_idx = self.gc.allocate(
                    &mut self.heap,
                    HeapValue::Iterator(JsIterator {
                        kind: "string".to_string(),
                        index: 0,
                        target: None,
                        data: Some(Value::Array(data_idx)),
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
            if let HeapValue::Iterator(iter) = &self.heap[*iter_idx] {
                let kind = iter.kind.clone();
                let index = iter.index;
                let target_val = iter.target.clone();
                let data_val = iter.data.clone();
                match kind.as_str() {
                    "array" | "string" => {
                        if let Some(Value::Array(arr_idx)) = &data_val {
                            if let HeapValue::Array(arr) = &self.heap[*arr_idx] {
                                if index >= arr.elements.len() {
                                    return Ok(ControlFlowOutcome::Jump(target));
                                }
                                let value = arr.elements[index].clone();
                                if let HeapValue::Iterator(iter_mut) = &mut self.heap[*iter_idx] {
                                    iter_mut.index = index + 1;
                                }
                                self.stack.push(value);
                            }
                        }
                        return Ok(ControlFlowOutcome::Next);
                    }
                    "map" => {
                        if let Some(Value::Map(map_idx)) = &target_val {
                            if let HeapValue::Map(m) = &self.heap[*map_idx] {
                                if index >= m.keys.len() {
                                    return Ok(ControlFlowOutcome::Jump(target));
                                }
                                let pair_idx = self.heap.len();
                                self.heap.push(HeapValue::Array(JsArray {
                                    elements: vec![m.keys[index].clone(), m.values[index].clone()],
                                }));
                                self.stack.push(Value::Array(pair_idx));
                                if let HeapValue::Iterator(iter_mut) = &mut self.heap[*iter_idx] {
                                    iter_mut.index = index + 1;
                                }
                            }
                        }
                        return Ok(ControlFlowOutcome::Next);
                    }
                    "set" => {
                        if let Some(Value::Set(set_idx)) = &target_val {
                            if let HeapValue::Set(s) = &self.heap[*set_idx] {
                                if index >= s.values.len() {
                                    return Ok(ControlFlowOutcome::Jump(target));
                                }
                                let value = s.values[index].clone();
                                self.stack.push(value);
                                if let HeapValue::Iterator(iter_mut) = &mut self.heap[*iter_idx] {
                                    iter_mut.index = index + 1;
                                }
                            }
                        }
                        return Ok(ControlFlowOutcome::Next);
                    }
                    _ => {}
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
            let index;
            let data_val: Option<Value>;
            if let HeapValue::Iterator(iter) = &self.heap[iter_idx] {
                index = iter.index;
                data_val = iter.data.clone();
            } else {
                let next_fn = self.get_property(&iterator, &Value::String("next".to_string()))?;
                let next_result = self.call_value(&next_fn, &iterator, &[])?;
                let done = self.get_property(&next_result, &Value::String("done".to_string()))?;
                match done {
                    Value::Boolean(true) => return Ok(ControlFlowOutcome::Jump(target)),
                    _ => {
                        let value =
                            self.get_property(&next_result, &Value::String("value".to_string()))?;
                        let awaited_value = Self::resolve_value_promise(&self.heap, &value);
                        self.stack.push(awaited_value);
                        return Ok(ControlFlowOutcome::Next);
                    }
                }
            }
            if let Some(data_val) = data_val {
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
                if let HeapValue::Iterator(iter_mut) = &mut self.heap[iter_idx] {
                    iter_mut.index = index + 1;
                }
                let awaited_value = Self::resolve_value_promise(&self.heap, &value);
                self.stack.push(awaited_value);
                return Ok(ControlFlowOutcome::Next);
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
