use super::{HeapValue, Interpreter};
use crate::objects::js_promise::PromiseState;
use crate::objects::Value;

impl Interpreter {
    pub(crate) fn drain_microtasks(&mut self) {
        // Process microtasks until the queue is empty. Use `take_microtasks`
        // (mem::replace of the VecDeque) instead of collecting into a fresh
        // Vec each wave — avoids alloc + double-move of every Microtask.
        // Newly enqueued handlers from a wave are processed in the next.
        loop {
            if !self.async_runtime.has_microtasks() {
                return;
            }
            let tasks = self.async_runtime.take_microtasks();
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
                // Take handlers with mem::take — no intermediate Vec of Values.
                let then_handlers = std::mem::take(&mut promise.then_handlers);
                let finally_handlers = std::mem::take(&mut promise.finally_handlers);
                promise.state = PromiseState::Fulfilled(value.clone());
                for h in then_handlers {
                    self.async_runtime
                        .enqueue_microtask_with_arg(Value::Function(h.callback), value.clone());
                }
                for h in finally_handlers {
                    self.async_runtime
                        .enqueue_microtask(Value::Function(h.callback));
                }
            }
        }
    }

    pub(crate) fn reject_promise(&mut self, promise_idx: usize, reason: Value) {
        if let HeapValue::Promise(promise) = &mut self.heap[promise_idx] {
            if promise.state == PromiseState::Pending {
                let catch_handlers = std::mem::take(&mut promise.catch_handlers);
                let finally_handlers = std::mem::take(&mut promise.finally_handlers);
                promise.state = PromiseState::Rejected(reason.clone());
                for h in catch_handlers {
                    self.async_runtime
                        .enqueue_microtask_with_arg(Value::Function(h.callback), reason.clone());
                }
                for h in finally_handlers {
                    self.async_runtime
                        .enqueue_microtask(Value::Function(h.callback));
                }
            }
        }
    }
}
