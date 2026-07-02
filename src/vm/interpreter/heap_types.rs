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
    pub rest_param: Option<String>,
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
    pub is_arrow: bool,
    pub captured_this: Option<Value>,
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
    pub generator_yielded: bool,
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
    pub(crate) compiled: Option<JsCompiledRegex>,
}

#[derive(Debug, Clone)]
pub enum JsCompiledRegex {
    Simple(regex::Regex),
    Advanced(fancy_regex::Regex),
}

fn has_advanced_features(pattern: &str) -> bool {
    // Check for lookaheads, lookbehinds, backreferences
    let bytes = pattern.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    while i < len {
        if bytes[i] == b'(' && i + 1 < len && bytes[i + 1] == b'?' {
            if i + 2 < len {
                match bytes[i + 2] {
                    b'=' | b'!' | b'<' => return true, // lookahead/lookbehind
                    _ => {}
                }
            }
        }
        if bytes[i] == b'\\' && i + 1 < len && bytes[i + 1].is_ascii_digit() {
            return true; // backreference
        }
        i += 1;
    }
    false
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

        let compiled = if has_advanced_features(pattern) {
            JsCompiledRegex::Advanced(
                fancy_regex::Regex::new(&regex_flags).map_err(|e| e.to_string())?
            )
        } else {
            JsCompiledRegex::Simple(
                regex::Regex::new(&regex_flags).map_err(|e| e.to_string())?
            )
        };

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
            compiled: Some(compiled),
        })
    }

    pub fn test(&self, input: &str) -> bool {
        if let Some(ref compiled) = self.compiled {
            match compiled {
                JsCompiledRegex::Simple(re) => re.is_match(input),
                JsCompiledRegex::Advanced(re) => re.is_match(input).unwrap_or(false),
            }
        } else {
            false
        }
    }

    pub fn exec_at(&self, input: &str, start: usize) -> Option<(Vec<String>, usize)> {
        let tail = &input[start..];
        let (results, match_end) = match self.compiled.as_ref()? {
            JsCompiledRegex::Simple(re) => {
                let caps = re.captures(tail)?;
                let mut results = Vec::with_capacity(caps.len());
                for i in 0..caps.len() {
                    results.push(
                        caps.get(i)
                            .map(|m| input[start + m.start()..start + m.end()].to_string())
                            .unwrap_or_default(),
                    );
                }
                let match_end = caps.get(0).map(|m| m.end()).unwrap_or(0);
                (results, match_end)
            }
            JsCompiledRegex::Advanced(re) => {
                let caps = re.captures(tail).ok()??;
                let mut results = Vec::with_capacity(caps.len());
                for i in 0..caps.len() {
                    results.push(
                        caps.get(i)
                            .map(|m| input[start + m.start()..start + m.end()].to_string())
                            .unwrap_or_default(),
                    );
                }
                let match_end = caps.get(0).map(|m| m.end()).unwrap_or(0);
                (results, match_end)
            }
        };
        Some((results, start + match_end))
    }

    pub fn exec(&self, input: &str) -> Option<Vec<String>> {
        self.exec_at(input, 0).map(|(v, _)| v)
    }

    pub fn find_all(&self, input: &str) -> Vec<String> {
        if let Some(ref compiled) = self.compiled {
            match compiled {
                JsCompiledRegex::Simple(re) => {
                    re.find_iter(input)
                        .map(|m| m.as_str().to_string())
                        .collect()
                }
                JsCompiledRegex::Advanced(re) => {
                    re.find_iter(input)
                        .filter_map(|m| m.ok())
                        .map(|m| m.as_str().to_string())
                        .collect()
                }
            }
        } else {
            Vec::new()
        }
    }

    pub fn replace(&self, input: &str, replacement: &str) -> String {
        if let Some(ref compiled) = self.compiled {
            match compiled {
                JsCompiledRegex::Simple(re) => re.replace_all(input, replacement).to_string(),
                JsCompiledRegex::Advanced(re) => re.replace_all(input, replacement).to_string(),
            }
        } else {
            input.to_string()
        }
    }

    pub fn search(&self, input: &str) -> i64 {
        if let Some(ref compiled) = self.compiled {
            match compiled {
                JsCompiledRegex::Simple(re) => {
                    re.find(input).map(|m| m.start() as i64).unwrap_or(-1)
                }
                JsCompiledRegex::Advanced(re) => {
                    re.find(input).ok().flatten().map(|m| m.start() as i64).unwrap_or(-1)
                }
            }
        } else {
            -1
        }
    }

    pub fn split(&self, input: &str) -> Vec<String> {
        if let Some(ref compiled) = self.compiled {
            match compiled {
                JsCompiledRegex::Simple(re) => {
                    re.split(input).map(|s| s.to_string()).collect()
                }
                JsCompiledRegex::Advanced(re) => {
                    re.split(input).filter_map(|s| s.ok()).map(|s| s.to_string()).collect()
                }
            }
        } else {
            vec![input.to_string()]
        }
    }
}
