use crate::compiler::CompiledModule;
use crate::objects::js_array::TypedArray;
use crate::objects::js_collections::{JsMap, JsSet, JsWeakMap, JsWeakSet};
use crate::objects::js_date::JsDate;
use crate::objects::js_promise::JsPromise;
use crate::objects::Value;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct JsObject {
    pub properties: HashMap<String, Value>,
    pub prototype: Option<usize>,
    pub extensible: bool,
}

impl Default for JsObject {
    fn default() -> Self {
        Self::new()
    }
}

impl JsObject {
    pub fn new() -> Self {
        Self {
            properties: HashMap::new(),
            prototype: None,
            extensible: true,
        }
    }

    pub fn with_prototype(prototype: Option<usize>) -> Self {
        Self {
            properties: HashMap::new(),
            prototype,
            extensible: true,
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
    pub is_generator: bool,
    pub source_file: Option<String>,
    pub source_line: Option<usize>,
}

#[derive(Debug, Clone)]
pub enum HeapValue {
    String(String),
    Object(JsObject),
    Array(JsArray),
    Function(JsFunction),
    Promise(JsPromise),
    Proxy(JsProxyData),
    Generator(JsGenerator),
    TypedArray(TypedArray),
    Map(JsMap),
    Set(JsSet),
    WeakMap(JsWeakMap),
    WeakSet(JsWeakSet),
    Date(JsDate),
    RegExp(JsRegExp),
    Buffer(Vec<u8>),
}

#[derive(Debug, Clone)]
pub struct JsGenerator {
    pub yield_value: Value,
    pub resume_pc: usize,
    pub saved_stack: Vec<Value>,
    pub saved_block_scope_stack: Vec<usize>,
    pub func_heap_idx: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct JsProxyData {
    pub target: Value,
    pub handler: Value,
}

#[derive(Debug, Clone)]
pub struct JsRegExp {
    pub source: String,
    pub flags: String,
    pub global: bool,
    pub ignore_case: bool,
    pub multiline: bool,
    pub dot_all: bool,
    pub unicode: bool,
    pub sticky: bool,
    pub last_index: f64,
    compiled: Option<JsCompiledRegex>,
}

#[derive(Debug, Clone)]
pub struct JsCompiledRegex {
    pub pattern: regex::Regex,
}

impl JsRegExp {
    pub fn new(pattern: &str, flags: &str) -> Result<Self, String> {
        let mut regex_flags = String::new();
        let global = flags.contains('g');
        let ignore_case = flags.contains('i');
        let multiline = flags.contains('m');
        let dot_all = flags.contains('s');
        let unicode = flags.contains('u');
        let sticky = flags.contains('y');

        if ignore_case {
            regex_flags.push_str("(?i)");
        }
        if multiline {
            regex_flags.push_str("(?m)");
        }
        if dot_all {
            regex_flags.push_str("(?s)");
        }
        regex_flags.push_str(pattern);

        let compiled = regex::Regex::new(&regex_flags).map_err(|e| e.to_string())?;

        Ok(JsRegExp {
            source: pattern.to_string(),
            flags: flags.to_string(),
            global,
            ignore_case,
            multiline,
            dot_all,
            unicode,
            sticky,
            last_index: 0.0,
            compiled: Some(JsCompiledRegex { pattern: compiled }),
        })
    }

    pub fn test(&self, input: &str) -> bool {
        if let Some(ref compiled) = self.compiled {
            compiled.pattern.is_match(input)
        } else {
            false
        }
    }

    pub fn exec(&self, input: &str) -> Option<Vec<String>> {
        if let Some(ref compiled) = self.compiled {
            let caps = compiled.pattern.captures(input)?;
            let mut results = Vec::new();
            for i in 0..caps.len() {
                results.push(
                    caps.get(i)
                        .map(|m| m.as_str().to_string())
                        .unwrap_or_default(),
                );
            }
            Some(results)
        } else {
            None
        }
    }

    pub fn find_all(&self, input: &str) -> Vec<String> {
        if let Some(ref compiled) = self.compiled {
            compiled
                .pattern
                .find_iter(input)
                .map(|m| m.as_str().to_string())
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn replace(&self, input: &str, replacement: &str) -> String {
        if let Some(ref compiled) = self.compiled {
            compiled.pattern.replace_all(input, replacement).to_string()
        } else {
            input.to_string()
        }
    }

    pub fn search(&self, input: &str) -> i64 {
        if let Some(ref compiled) = self.compiled {
            compiled
                .pattern
                .find(input)
                .map(|m| m.start() as i64)
                .unwrap_or(-1)
        } else {
            -1
        }
    }

    pub fn split(&self, input: &str) -> Vec<String> {
        if let Some(ref compiled) = self.compiled {
            compiled
                .pattern
                .split(input)
                .map(|s| s.to_string())
                .collect()
        } else {
            vec![input.to_string()]
        }
    }
}
