use crate::objects::Value;

#[derive(Debug, Clone)]
pub struct JsProxy {
    pub target: Value,
    pub handler: ProxyHandler,
}

#[derive(Debug, Clone)]
pub struct ProxyHandler {
    pub get: Option<usize>,
    pub set: Option<usize>,
    pub has: Option<usize>,
    pub delete_property: Option<usize>,
    pub own_keys: Option<usize>,
    pub get_own_property_descriptor: Option<usize>,
    pub define_property: Option<usize>,
    pub get_prototype_of: Option<usize>,
    pub set_prototype_of: Option<usize>,
    pub is_extensible: Option<usize>,
    pub prevent_extensions: Option<usize>,
    pub apply: Option<usize>,
    pub construct: Option<usize>,
}

impl ProxyHandler {
    pub fn new() -> Self {
        Self {
            get: None,
            set: None,
            has: None,
            delete_property: None,
            own_keys: None,
            get_own_property_descriptor: None,
            define_property: None,
            get_prototype_of: None,
            set_prototype_of: None,
            is_extensible: None,
            prevent_extensions: None,
            apply: None,
            construct: None,
        }
    }
}

impl JsProxy {
    pub fn new(target: Value, handler: ProxyHandler) -> Self {
        Self { target, handler }
    }

    pub fn get_trap(&self) -> Option<usize> {
        self.handler.get
    }

    pub fn set_trap(&self) -> Option<usize> {
        self.handler.set
    }

    pub fn has_trap(&self) -> Option<usize> {
        self.handler.has
    }

    pub fn delete_property_trap(&self) -> Option<usize> {
        self.handler.delete_property
    }

    pub fn apply_trap(&self) -> Option<usize> {
        self.handler.apply
    }

    pub fn construct_trap(&self) -> Option<usize> {
        self.handler.construct
    }
}

impl Default for ProxyHandler {
    fn default() -> Self {
        Self::new()
    }
}
