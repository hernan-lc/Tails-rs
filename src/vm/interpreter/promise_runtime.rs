use super::{HeapValue, Interpreter};
use crate::objects::js_promise::PromiseState;
use crate::objects::Value;

impl Interpreter {
    pub(crate) fn drain_microtasks(&mut self) {
        // Phase 8.6: Process microtasks in a tight loop until the queue is
        // fully drained.  This avoids re-entering the outer event-loop
        // between chain links (each .then() callback that resolves a
        // promise enqueues new microtasks that should be processed in the
        // same drain pass).
        loop {
            if self.async_runtime.is_idle() {
                return;
            }
            let tasks = self.async_runtime.run_microtasks();
            if tasks.is_empty() {
                return;
            }
            for task in tasks {
                let _ = self.call_value(&task.callback, &Value::Undefined, &[task.arg]);
            }
        }
    }

    pub(crate) fn create_resolve_fn(&mut self, promise_idx: usize) -> Value {
        let heap_idx = self
            .gc
            .allocate(&mut self.heap, HeapValue::DeferredResolve(promise_idx));
        Value::Function(heap_idx)
    }

    pub(crate) fn create_reject_fn(&mut self, promise_idx: usize) -> Value {
        let heap_idx = self
            .gc
            .allocate(&mut self.heap, HeapValue::DeferredReject(promise_idx));
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
