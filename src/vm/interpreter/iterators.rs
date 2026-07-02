use super::*;
use crate::errors::{Error, Result};
use crate::objects::js_promise::PromiseState;
use crate::objects::Value;
use crate::runtime_env::native_fns::constants as c;
use std::collections::HashMap;

impl Interpreter {
    fn make_builtin_iter_props() -> HashMap<String, Value> {
        let mut props = HashMap::new();
        props.insert("__type".to_string(), Value::String("array".to_string()));
        props.insert("__index".to_string(), Value::Integer(0));
        props.insert("map".to_string(), Value::NativeFunction(c::ITERATOR_MAP));
        props.insert(
            "filter".to_string(),
            Value::NativeFunction(c::ITERATOR_FILTER),
        );
        props.insert("take".to_string(), Value::NativeFunction(c::ITERATOR_TAKE));
        props.insert("drop".to_string(), Value::NativeFunction(c::ITERATOR_DROP));
        props.insert(
            "forEach".to_string(),
            Value::NativeFunction(c::ITERATOR_FOR_EACH),
        );
        props.insert(
            "toArray".to_string(),
            Value::NativeFunction(c::ITERATOR_TO_ARRAY),
        );
        props
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
            Value::Map(map_idx) => {
                let (keys, values) = if let HeapValue::Map(m) = &self.heap[*map_idx] {
                    (m.keys.clone(), m.values.clone())
                } else {
                    (Vec::new(), Vec::new())
                };
                let mut elements = Vec::new();
                for (k, v) in keys.iter().zip(values.iter()) {
                    let pair_idx = self.heap.len();
                    self.heap.push(HeapValue::Array(JsArray {
                        elements: vec![k.clone(), v.clone()],
                    }));
                    elements.push(Value::Array(pair_idx));
                }
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
            Value::Set(set_idx) => {
                let values = if let HeapValue::Set(s) = &self.heap[*set_idx] {
                    s.values.clone()
                } else {
                    Vec::new()
                };
                let data_idx = self.gc.allocate(
                    &mut self.heap,
                    HeapValue::Array(JsArray { elements: values }),
                );
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
                    }
                    return Ok(ControlFlowOutcome::Next);
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
