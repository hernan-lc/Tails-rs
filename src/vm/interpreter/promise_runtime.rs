use super::{HeapValue, Interpreter};
use crate::errors::Error;
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
                let result = self.call_value(&task.callback, &Value::Undefined, &[task.arg]);
                self.settle_chained(task.chained_promise, result);
            }
        }
    }

    /// After a chain-continuation microtask runs, settle its `chained_promise`
    /// with the handler's outcome. [`NO_CHAIN`](crate::runtime_env::async_runtime::NO_CHAIN)
    /// means the task is a standalone microtask (e.g. `queueMicrotask`) whose
    /// result is intentionally dropped.
    fn settle_chained(
        &mut self,
        chained_promise: usize,
        result: std::result::Result<Value, Error>,
    ) {
        if chained_promise == crate::runtime_env::async_runtime::NO_CHAIN {
            return;
        }
        match result {
            Ok(value) => self.resolve_chained_value(chained_promise, value),
            Err(e) => self.reject_promise(chained_promise, Value::from_string(e.to_string())),
        }
    }

    /// Settle `chained_promise` with `value`, unwrapping it first when `value`
    /// is itself a (possibly pending) promise — matching Promises/A+ thenable
    /// adoption so `return somePromise` inside a `.then` chains correctly.
    fn resolve_chained_value(&mut self, chained_promise: usize, value: Value) {
        if let Value::Promise(ret_idx) = value {
            let resolve_fn = self.create_resolve_fn(chained_promise);
            let reject_fn = self.create_reject_fn(chained_promise);
            let _ = self.call_value(
                &Value::NativeFunction(crate::runtime_env::native_fns::constants::PROMISE_THEN),
                &Value::Promise(ret_idx),
                &[resolve_fn],
            );
            let _ = self.call_value(
                &Value::NativeFunction(crate::runtime_env::native_fns::constants::PROMISE_CATCH),
                &Value::Promise(ret_idx),
                &[reject_fn],
            );
        } else {
            self.resolve_promise(chained_promise, value);
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
                let catch_handlers = std::mem::take(&mut promise.catch_handlers);
                promise.state = PromiseState::Fulfilled(value.clone());
                // onFulfilled handlers run and forward their result to the
                // promise returned by `.then`.
                for h in then_handlers {
                    self.async_runtime.enqueue_microtask_chained(
                        Value::Function(h.callback),
                        value.clone(),
                        h.chained_promise,
                    );
                }
                // `.catch` chains pass a fulfilled value through untouched.
                for h in catch_handlers {
                    self.resolve_promise(h.chained_promise, value.clone());
                }
                // `.finally` runs its callback, then passes the value through.
                for h in finally_handlers {
                    self.async_runtime
                        .enqueue_microtask(Value::Function(h.callback));
                    self.resolve_promise(h.chained_promise, value.clone());
                }
            }
        }
    }

    pub(crate) fn reject_promise(&mut self, promise_idx: usize, reason: Value) {
        if let HeapValue::Promise(promise) = &mut self.heap[promise_idx] {
            if promise.state == PromiseState::Pending {
                let catch_handlers = std::mem::take(&mut promise.catch_handlers);
                let finally_handlers = std::mem::take(&mut promise.finally_handlers);
                let then_handlers = std::mem::take(&mut promise.then_handlers);
                promise.state = PromiseState::Rejected(reason.clone());
                // onRejected handlers run and forward their result to the
                // promise returned by `.catch`.
                for h in catch_handlers {
                    self.async_runtime.enqueue_microtask_chained(
                        Value::Function(h.callback),
                        reason.clone(),
                        h.chained_promise,
                    );
                }
                // Unhandled `.then` chains adopt the rejection (pass-through).
                for h in then_handlers {
                    self.reject_promise(h.chained_promise, reason.clone());
                }
                // `.finally` runs its callback, then passes the rejection through.
                for h in finally_handlers {
                    self.async_runtime
                        .enqueue_microtask(Value::Function(h.callback));
                    self.reject_promise(h.chained_promise, reason.clone());
                }
            }
        }
    }
}
