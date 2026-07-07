use tails::{TailsRuntime, Value};
use std::path::Path;

pub struct TailsTestHarness {
    pub rt: TailsRuntime,
}

impl TailsTestHarness {
    pub fn new() -> Self {
        Self {
            rt: TailsRuntime::default(),
        }
    }

    pub fn eval(&mut self, source: &str) -> Value {
        self.rt.eval(source).expect("Eval failed")
    }

    #[allow(dead_code)]
    pub fn eval_module(&mut self, source: &str, path: &str) -> Value {
        self.rt.eval_module(source, Path::new(path)).expect("Module eval failed")
    }

    pub fn assert_eq(&self, actual: Value, expected: Value) {
        assert_eq!(actual, expected);
    }

    #[allow(dead_code)]
    pub fn assert_true(&self, val: Value) {
        assert_eq!(val, Value::Boolean(true));
    }
}
