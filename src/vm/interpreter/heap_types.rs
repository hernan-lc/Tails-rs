use std::collections::HashMap;
use std::rc::Rc;
use crate::compiler::CompiledModule;
use crate::objects::Value;
use crate::objects::js_promise::JsPromise;

#[derive(Debug, Clone)]
pub struct JsObject {
    pub properties: HashMap<String, Value>,
    pub prototype: Option<usize>,
}

impl JsObject {
    pub fn new() -> Self {
        Self {
            properties: HashMap::new(),
            prototype: None,
        }
    }

    pub fn with_prototype(prototype: Option<usize>) -> Self {
        Self {
            properties: HashMap::new(),
            prototype,
        }
    }
}

#[derive(Debug, Clone)]
pub struct JsArray {
    pub elements: Vec<Value>,
}

#[derive(Debug, Clone)]
pub struct JsFunction {
    pub name: Option<String>,
    pub params: Vec<String>,
    pub bytecode_index: usize,
    pub closure: Vec<Value>,
    pub prototype: Option<usize>,
    pub super_class: Option<Value>,
    pub properties: HashMap<String, Value>,
    pub owner_module: Option<Rc<CompiledModule>>,
    pub module_scope: Option<Rc<HashMap<String, Value>>>,
}

#[derive(Debug, Clone)]
pub enum HeapValue {
    String(String),
    Object(JsObject),
    Array(JsArray),
    Function(JsFunction),
    Promise(JsPromise),
    Proxy(JsProxyData),
}

#[derive(Debug, Clone)]
pub struct JsProxyData {
    pub target: Value,
    pub handler: Value,
}
