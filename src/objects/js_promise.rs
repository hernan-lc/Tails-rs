use crate::objects::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum PromiseState {
    Pending,
    Fulfilled(Value),
    Rejected(Value),
}

#[derive(Debug, Clone)]
pub struct JsPromise {
    pub state: PromiseState,
    pub then_handlers: Vec<PromiseHandler>,
    pub catch_handlers: Vec<PromiseHandler>,
    pub finally_handlers: Vec<PromiseHandler>,
}

#[derive(Debug, Clone)]
pub struct PromiseHandler {
    pub callback: usize,
    pub resolve: bool,
    /// The promise returned by the `.then`/`.catch`/`.finally` call that
    /// registered this handler. Its settlement is driven by this handler's
    /// result (or, when the handler doesn't match the settlement type, by
    /// pass-through of the original outcome) so promise chains actually
    /// propagate values instead of leaving the next promise forever pending.
    pub chained_promise: usize,
}

impl JsPromise {
    pub fn new() -> Self {
        Self {
            state: PromiseState::Pending,
            then_handlers: Vec::new(),
            catch_handlers: Vec::new(),
            finally_handlers: Vec::new(),
        }
    }

    pub fn fulfilled(value: Value) -> Self {
        Self {
            state: PromiseState::Fulfilled(value),
            then_handlers: Vec::new(),
            catch_handlers: Vec::new(),
            finally_handlers: Vec::new(),
        }
    }

    pub fn rejected(reason: Value) -> Self {
        Self {
            state: PromiseState::Rejected(reason),
            then_handlers: Vec::new(),
            catch_handlers: Vec::new(),
            finally_handlers: Vec::new(),
        }
    }

    pub fn resolve(&mut self, value: Value) {
        if self.state == PromiseState::Pending {
            self.state = PromiseState::Fulfilled(value);
        }
    }

    pub fn reject(&mut self, reason: Value) {
        if self.state == PromiseState::Pending {
            self.state = PromiseState::Rejected(reason);
        }
    }

    pub fn is_pending(&self) -> bool {
        self.state == PromiseState::Pending
    }

    pub fn is_fulfilled(&self) -> bool {
        matches!(self.state, PromiseState::Fulfilled(_))
    }

    pub fn is_rejected(&self) -> bool {
        matches!(self.state, PromiseState::Rejected(_))
    }

    pub fn value(&self) -> Option<&Value> {
        match &self.state {
            PromiseState::Fulfilled(value) => Some(value),
            _ => None,
        }
    }

    pub fn reason(&self) -> Option<&Value> {
        match &self.state {
            PromiseState::Rejected(reason) => Some(reason),
            _ => None,
        }
    }

    pub fn then(&mut self, callback: usize, chained_promise: usize) {
        self.then_handlers.push(PromiseHandler {
            callback,
            resolve: true,
            chained_promise,
        });
    }

    pub fn catch(&mut self, callback: usize, chained_promise: usize) {
        self.catch_handlers.push(PromiseHandler {
            callback,
            resolve: false,
            chained_promise,
        });
    }

    pub fn finally(&mut self, callback: usize, chained_promise: usize) {
        self.finally_handlers.push(PromiseHandler {
            callback,
            resolve: true,
            chained_promise,
        });
    }

    pub fn resolve_all(promises: &[JsPromise]) -> JsPromise {
        let mut result = JsPromise::new();
        let mut values = Vec::new();

        for promise in promises {
            match &promise.state {
                PromiseState::Fulfilled(value) => values.push(value.clone()),
                PromiseState::Rejected(reason) => {
                    result.reject(reason.clone());
                    return result;
                }
                PromiseState::Pending => {
                    return result;
                }
            }
        }

        result.resolve(Value::Undefined);
        result
    }

    pub fn race(promises: &[JsPromise]) -> JsPromise {
        let mut result = JsPromise::new();

        for promise in promises {
            match &promise.state {
                PromiseState::Fulfilled(value) => {
                    result.resolve(value.clone());
                    return result;
                }
                PromiseState::Rejected(reason) => {
                    result.reject(reason.clone());
                    return result;
                }
                PromiseState::Pending => {}
            }
        }

        result
    }
}

impl Default for JsPromise {
    fn default() -> Self {
        Self::new()
    }
}
