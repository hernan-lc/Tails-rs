use super::{HeapValue, Interpreter, JsFunction, JsObject};
use crate::objects::js_promise::PromiseState;
use crate::objects::Value;
use rustc_hash::FxHashMap;
use std::cell::RefCell;
use std::rc::Rc;

impl Interpreter {
    pub(crate) fn drain_microtasks(&mut self) {
        // Batch collect all pending microtasks before execution
        // This avoids re-entrant issues and reduces overhead
        let tasks = self.async_runtime.run_microtasks();
        for task in tasks {
            let _ = self.call_value(&task.callback, &Value::Undefined, &[task.arg]);
        }
    }

    pub(crate) fn create_resolve_fn(&mut self, promise_idx: usize) -> Value {
        let proto_idx = self
            .gc
            .allocate(&mut self.heap, HeapValue::Object(JsObject::new()));
        let heap_idx = self.gc.allocate(
            &mut self.heap,
            HeapValue::Function(JsFunction {
                name: Some("resolve".into()),
                params: vec!["value".into()],
                rest_param: None,
                bytecode_index: usize::MAX,
                closure: Rc::new(RefCell::new(vec![Value::Promise(promise_idx)])),
                prototype: Some(proto_idx),
                super_class: None,
                properties: FxHashMap::default(),
                owner_module: None,
                module_scope: None,
                is_generator: false,
                source_file: None,
                source_line: None,
                is_arrow: false,
                captured_this: None,
            }),
        );
        Value::Function(heap_idx)
    }

    pub(crate) fn create_reject_fn(&mut self, promise_idx: usize) -> Value {
        let proto_idx = self
            .gc
            .allocate(&mut self.heap, HeapValue::Object(JsObject::new()));
        let heap_idx = self.gc.allocate(
            &mut self.heap,
            HeapValue::Function(JsFunction {
                name: Some("reject".into()),
                params: vec!["reason".into()],
                rest_param: None,
                bytecode_index: usize::MAX,
                closure: Rc::new(RefCell::new(vec![Value::Promise(promise_idx)])),
                prototype: Some(proto_idx),
                super_class: None,
                properties: FxHashMap::default(),
                owner_module: None,
                module_scope: None,
                is_generator: false,
                source_file: None,
                source_line: None,
                is_arrow: false,
                captured_this: None,
            }),
        );
        Value::Function(heap_idx)
    }

    pub(crate) fn resolve_promise(&mut self, promise_idx: usize, value: Value) {
        if let HeapValue::Promise(promise) = &mut self.heap[promise_idx] {
            if promise.state == PromiseState::Pending {
                promise.state = PromiseState::Fulfilled(value.clone());
                let handlers: Vec<Value> = promise
                    .then_handlers
                    .iter()
                    .map(|h| Value::Function(h.callback))
                    .collect();
                promise.then_handlers.clear();
                for handler in handlers {
                    self.async_runtime
                        .enqueue_microtask_with_arg(handler, value.clone());
                }
                let finally_handlers: Vec<Value> = promise
                    .finally_handlers
                    .iter()
                    .map(|h| Value::Function(h.callback))
                    .collect();
                promise.finally_handlers.clear();
                for handler in finally_handlers {
                    self.async_runtime.enqueue_microtask(handler);
                }
            }
        }
    }

    pub(crate) fn reject_promise(&mut self, promise_idx: usize, reason: Value) {
        if let HeapValue::Promise(promise) = &mut self.heap[promise_idx] {
            if promise.state == PromiseState::Pending {
                promise.state = PromiseState::Rejected(reason.clone());
                let handlers: Vec<Value> = promise
                    .catch_handlers
                    .iter()
                    .map(|h| Value::Function(h.callback))
                    .collect();
                promise.catch_handlers.clear();
                for handler in handlers {
                    self.async_runtime
                        .enqueue_microtask_with_arg(handler, reason.clone());
                }
                let finally_handlers: Vec<Value> = promise
                    .finally_handlers
                    .iter()
                    .map(|h| Value::Function(h.callback))
                    .collect();
                promise.finally_handlers.clear();
                for handler in finally_handlers {
                    self.async_runtime.enqueue_microtask(handler);
                }
            }
        }
    }
}
