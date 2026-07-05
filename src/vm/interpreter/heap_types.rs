use crate::compiler::CompiledModule;
use crate::objects::js_array::TypedArray;
use crate::objects::js_collections::{JsMap, JsSet, JsWeakMap, JsWeakSet};
use crate::objects::js_date::JsDate;
use crate::objects::js_promise::JsPromise;
use crate::objects::Value;
use rustc_hash::FxHashMap;
use std::cell::RefCell;
use std::rc::Rc;

const INLINE_CAP: usize = 8;

/// Phase 8.2 — Inline cache for property access.  The `last_index` field
/// caches the slot index of the most recently looked-up property so that
/// repeated accesses to the same key (common in OO patterns) skip the
/// linear scan entirely.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum PropertyStorage {
    Inline(u8, [Option<(String, Value)>; INLINE_CAP], bool, u8),
    Map(FxHashMap<String, Value>),
}

impl Default for PropertyStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyStorage {
    pub fn new() -> Self {
        Self::Inline(0, Default::default(), false, u8::MAX)
    }

    #[allow(dead_code)]
    pub(crate) fn upgrade(map: FxHashMap<String, Value>) -> Self {
        Self::Map(map)
    }

    /// Returns true if this property storage contains any getter/setter accessors.
    /// Used to skip the find_accessor linear scan in the common case.
    pub fn has_accessors(&self) -> bool {
        match self {
            Self::Inline(_, _, has_acc, _) => *has_acc,
            Self::Map(m) => m
                .keys()
                .any(|k| k.starts_with("__getter_") || k.starts_with("__setter_")),
        }
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Self::Inline(len, slots, _has_acc, last_idx) => {
                let n = *len as usize;
                // Phase 8.2: Check inline cache only for objects with >2
                // properties — for smaller objects the linear scan is faster
                // than the cache-probe overhead.
                if n > 2 && *last_idx < n as u8 {
                    if let Some((k, v)) = &slots[*last_idx as usize] {
                        if k.as_str() == key {
                            return Some(v);
                        }
                    }
                }
                // Linear scan
                for slot in slots.iter().take(n).flatten() {
                    if slot.0 == key {
                        return Some(&slot.1);
                    }
                }
                None
            }
            Self::Map(m) => m.get(key),
        }
    }

    /// Phase 8.2: Like `get()` but updates the inline cache on hit.
    /// Use this in hot property-access paths where the same key is
    /// looked up repeatedly on the same object.
    pub fn get_cached(&mut self, key: &str) -> Option<&Value> {
        match self {
            Self::Inline(len, slots, _has_acc, last_idx) => {
                let n = *len as usize;
                // Check inline cache only for larger objects
                if n > 2 && *last_idx < n as u8 {
                    if let Some((k, v)) = &slots[*last_idx as usize] {
                        if k.as_str() == key {
                            return Some(v);
                        }
                    }
                }
                // Cache miss: linear scan, then update cache
                for (i, slot) in slots.iter().enumerate().take(n) {
                    if let Some((k, v)) = slot {
                        if k.as_str() == key {
                            if n > 2 {
                                *last_idx = i as u8;
                            }
                            return Some(v);
                        }
                    }
                }
                None
            }
            Self::Map(m) => m.get(key),
        }
    }

    pub fn insert(&mut self, key: String, value: Value) {
        let is_accessor = key.starts_with("__getter_") || key.starts_with("__setter_");
        match self {
            Self::Inline(len, slots, has_acc, last_idx) => {
                let n = *len as usize;
                // Update existing
                for (i, slot) in slots.iter_mut().enumerate().take(n) {
                    if let Some((k, _)) = slot {
                        if *k == key {
                            *slot = Some((key, value));
                            *last_idx = i as u8;
                            if is_accessor {
                                *has_acc = true;
                            }
                            return;
                        }
                    }
                }
                // Insert new
                if n < INLINE_CAP {
                    slots[n] = Some((key, value));
                    *len += 1;
                    *last_idx = n as u8;
                    if is_accessor {
                        *has_acc = true;
                    }
                } else {
                    // Upgrade to map
                    let mut map = FxHashMap::default();
                    for slot in slots.iter_mut().take(INLINE_CAP) {
                        if let Some((k, v)) = slot.take() {
                            map.insert(k, v);
                        }
                    }
                    map.insert(key, value);
                    *self = Self::Map(map);
                }
            }
            Self::Map(m) => {
                m.insert(key, value);
            }
        }
    }

    pub fn remove(&mut self, key: &str) -> Option<Value> {
        match self {
            Self::Inline(len, slots, _has_acc, last_idx) => {
                let n = *len as usize;
                for i in 0..n {
                    if let Some((k, _)) = &slots[i] {
                        if k == key {
                            let slot = slots[i].take();
                            // Shift remaining slots
                            for j in i..n - 1 {
                                slots[j] = slots[j + 1].take();
                            }
                            slots[n - 1] = None;
                            *len -= 1;
                            // Invalidate cache if it pointed to or past the removed slot
                            if *last_idx as usize >= n - 1 {
                                *last_idx = u8::MAX;
                            } else if *last_idx as usize >= i {
                                // The slot index shifted; invalidate to be safe
                                *last_idx = u8::MAX;
                            }
                            return slot.map(|(_, v)| v);
                        }
                    }
                }
                None
            }
            Self::Map(m) => m.remove(key),
        }
    }

    pub fn contains_key(&self, key: &str) -> bool {
        match self {
            Self::Inline(len, slots, _has_acc, last_idx) => {
                let n = *len as usize;
                // Phase 8.2: Check inline cache first
                if *last_idx < n as u8 {
                    if let Some((k, _)) = &slots[*last_idx as usize] {
                        if k.as_str() == key {
                            return true;
                        }
                    }
                }
                for slot in slots.iter().take(n).flatten() {
                    if slot.0 == key {
                        return true;
                    }
                }
                false
            }
            Self::Map(m) => m.contains_key(key),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Inline(len, _, _has_acc, _) => *len as usize,
            Self::Map(m) => m.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &Value)> {
        match self {
            Self::Inline(len, slots, _has_acc, _) => {
                let n = *len as usize;
                slots
                    .iter()
                    .take(n)
                    .filter_map(|slot| slot.as_ref().map(|(k, v)| (k.as_str(), v)))
                    .collect::<Vec<_>>()
                    .into_iter()
            }
            Self::Map(m) => m
                .iter()
                .map(|(k, v)| (k.as_str(), v))
                .collect::<Vec<_>>()
                .into_iter(),
        }
    }

    pub fn values(&self) -> impl Iterator<Item = &Value> {
        match self {
            Self::Inline(len, slots, _has_acc, _) => {
                let n = *len as usize;
                slots
                    .iter()
                    .take(n)
                    .filter_map(|slot| slot.as_ref().map(|(_, v)| v))
                    .collect::<Vec<_>>()
                    .into_iter()
            }
            Self::Map(m) => m.values().collect::<Vec<_>>().into_iter(),
        }
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&str, &mut Value)> {
        match self {
            Self::Inline(len, slots, _has_acc, _) => {
                let n = *len as usize;
                slots
                    .iter_mut()
                    .take(n)
                    .filter_map(|slot| slot.as_mut().map(|(k, v)| (k.as_str(), v)))
                    .collect::<Vec<_>>()
                    .into_iter()
            }
            Self::Map(m) => m
                .iter_mut()
                .map(|(k, v)| (k.as_str(), v))
                .collect::<Vec<_>>()
                .into_iter(),
        }
    }
}

impl From<FxHashMap<String, Value>> for PropertyStorage {
    fn from(map: FxHashMap<String, Value>) -> Self {
        Self::Map(map)
    }
}

impl PropertyStorage {
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        match self {
            Self::Inline(len, slots, _has_acc, _) => {
                let n = *len as usize;
                slots
                    .iter()
                    .take(n)
                    .filter_map(|slot| slot.as_ref().map(|(k, _)| k.as_str()))
                    .collect::<Vec<_>>()
                    .into_iter()
            }
            Self::Map(m) => m.keys().map(|k| k.as_str()).collect::<Vec<_>>().into_iter(),
        }
    }

    pub fn clear(&mut self) {
        match self {
            Self::Inline(len, slots, _has_acc, last_idx) => {
                for slot in slots.iter_mut() {
                    *slot = None;
                }
                *len = 0;
                *last_idx = u8::MAX;
            }
            Self::Map(m) => m.clear(),
        }
    }
}

impl<'a> IntoIterator for &'a PropertyStorage {
    type Item = (&'a str, &'a Value);
    type IntoIter = std::vec::IntoIter<(&'a str, &'a Value)>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter().collect::<Vec<_>>().into_iter()
    }
}

impl<'a> IntoIterator for &'a mut PropertyStorage {
    type Item = (&'a str, &'a mut Value);
    type IntoIter = std::vec::IntoIter<(&'a str, &'a mut Value)>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut().collect::<Vec<_>>().into_iter()
    }
}

/// Creates a `PropertyStorage` from key-value pairs.
///
/// # Example
/// ```ignore
/// let props = props! {
///     "href" => Value::String(url),
///     "toString" => Value::NativeFunction(c::URL_TO_STRING),
/// };
/// ```
#[macro_export]
macro_rules! props {
    ($($key:expr => $value:expr),* $(,)?) => {{
        let mut map = ::rustc_hash::FxHashMap::default();
        $(map.insert($key.to_string(), $value);)*
        $crate::vm::interpreter::PropertyStorage::from(map)
    }};
}

#[derive(Debug, Clone)]
pub struct JsObject {
    pub properties: PropertyStorage,
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
            properties: PropertyStorage::new(),
            prototype: None,
            extensible: true,
        }
    }

    pub fn with_prototype(prototype: Option<usize>) -> Self {
        Self {
            properties: PropertyStorage::new(),
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
    pub closure: Rc<RefCell<Vec<Value>>>,
    pub prototype: Option<usize>,
    pub super_class: Option<Value>,
    pub properties: PropertyStorage,
    pub owner_module: Option<Rc<CompiledModule>>,
    pub module_scope: Option<Rc<FxHashMap<String, Value>>>,
    pub is_generator: bool,
    pub source_file: Option<String>,
    pub source_line: Option<usize>,
    pub is_arrow: bool,
    pub captured_this: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct JsIterator {
    pub kind: String,
    pub index: usize,
    pub target: Option<Value>,
    pub data: Option<Value>,
}

#[allow(clippy::large_enum_variant)]
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
    Iterator(JsIterator),
    DeferredResolve(usize),
    DeferredReject(usize),
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
    /// Phase 3.4 — Lazy result cache for repeated matches on the same input.
    /// When `last_input` matches the cached string, reuse the cached `last_match_start`
    /// and `last_match_end` to skip regex matching entirely for `test()` calls.
    pub(crate) last_input: Option<String>,
    pub(crate) last_match_start: Option<usize>,
    pub(crate) last_match_end: Option<usize>,
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
        if bytes[i] == b'(' && i + 1 < len && bytes[i + 1] == b'?' && i + 2 < len {
            match bytes[i + 2] {
                b'=' | b'!' | b'<' => return true, // lookahead/lookbehind
                _ => {}
            }
        }
        if bytes[i] == b'\\' && i + 1 < len && bytes[i + 1].is_ascii_digit() {
            return true; // backreference
        }
        if bytes[i] == b'\\' && i + 1 < len && (bytes[i + 1] == b'p' || bytes[i + 1] == b'P') {
            return true; // Unicode property escape
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
                fancy_regex::Regex::new(&regex_flags).map_err(|e| e.to_string())?,
            )
        } else {
            JsCompiledRegex::Simple(regex::Regex::new(&regex_flags).map_err(|e| e.to_string())?)
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
            last_input: None,
            last_match_start: None,
            last_match_end: None,
        })
    }

    pub fn test(&self, input: &str) -> bool {
        // Phase 3.4 — Fast-path: for literal patterns (no metacharacters), use
        // str::contains directly instead of the regex engine.
        if self.is_literal_pattern() {
            return if self.ignore_case {
                input.to_lowercase().contains(&self.source.to_lowercase())
            } else {
                input.contains(self.source.as_str())
            };
        }
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
        // Phase 3.4 — Fast-path: for literal patterns, use str::find instead of
        // the regex engine. Returns the first match as a single capture group.
        if self.is_literal_pattern() {
            let tail = &input[start..];
            let found = if self.ignore_case {
                let needle_lower: Vec<char> = self
                    .source
                    .chars()
                    .map(|c| c.to_lowercase().next().unwrap_or(c))
                    .collect();
                let haystack_lower: Vec<char> = tail
                    .chars()
                    .map(|c| c.to_lowercase().next().unwrap_or(c))
                    .collect();
                let needle_len = needle_lower.len();
                haystack_lower
                    .windows(needle_len)
                    .position(|w| w == needle_lower.as_slice())
                    .map(|pos| {
                        let byte_pos = tail.char_indices().nth(pos).map(|(i, _)| i).unwrap_or(0);
                        let byte_end = tail[byte_pos..]
                            .char_indices()
                            .nth(needle_len)
                            .map(|(i, _)| byte_pos + i)
                            .unwrap_or(tail.len());
                        (tail[byte_pos..byte_end].to_string(), start + byte_end)
                    })
            } else {
                tail.find(self.source.as_str()).map(|pos| {
                    let end = pos + self.source.len();
                    (self.source.clone(), start + end)
                })
            };
            return found.map(|(m, e)| (vec![m], e));
        }

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
        // Phase 3.4 — Fast-path for literal patterns
        if self.is_literal_pattern() {
            return if self.ignore_case {
                let needle_lower: Vec<char> = self
                    .source
                    .chars()
                    .map(|c| c.to_lowercase().next().unwrap_or(c))
                    .collect();
                let haystack_lower: Vec<char> = input
                    .chars()
                    .map(|c| c.to_lowercase().next().unwrap_or(c))
                    .collect();
                let needle_len = needle_lower.len();
                haystack_lower
                    .windows(needle_len)
                    .position(|w| w == needle_lower.as_slice())
                    .map(|pos| {
                        let byte_pos = input.char_indices().nth(pos).map(|(i, _)| i).unwrap_or(0);
                        let byte_end = input[byte_pos..]
                            .char_indices()
                            .nth(needle_len)
                            .map(|(i, _)| byte_pos + i)
                            .unwrap_or(input.len());
                        vec![input[byte_pos..byte_end].to_string()]
                    })
            } else {
                input
                    .find(self.source.as_str())
                    .map(|pos| vec![input[pos..pos + self.source.len()].to_string()])
            };
        }
        self.exec_at(input, 0).map(|(v, _)| v)
    }

    /// Like `exec`, but also extracts named capture groups.
    /// Returns `(positional_groups, named_groups, match_start)`.
    pub fn exec_with_groups(
        &self,
        input: &str,
    ) -> Option<(Vec<String>, FxHashMap<String, String>, usize)> {
        if self.is_literal_pattern() {
            return self.exec(input).map(|v| (v, FxHashMap::default(), 0));
        }

        let tail = input;
        match self.compiled.as_ref()? {
            JsCompiledRegex::Simple(re) => {
                let caps = re.captures(tail)?;
                let match_start = caps.get(0).map(|m| m.start()).unwrap_or(0);
                let mut results = Vec::with_capacity(caps.len());
                for i in 0..caps.len() {
                    results.push(
                        caps.get(i)
                            .map(|m| m.as_str().to_string())
                            .unwrap_or_default(),
                    );
                }
                let mut groups = FxHashMap::default();
                for name in re.capture_names().flatten() {
                    if let Some(m) = caps.name(name) {
                        groups.insert(name.to_string(), m.as_str().to_string());
                    }
                }
                Some((results, groups, match_start))
            }
            JsCompiledRegex::Advanced(re) => {
                let caps = re.captures(tail).ok()??;
                let match_start = caps.get(0).map(|m| m.start()).unwrap_or(0);
                let mut results = Vec::with_capacity(caps.len());
                for i in 0..caps.len() {
                    results.push(
                        caps.get(i)
                            .map(|m| m.as_str().to_string())
                            .unwrap_or_default(),
                    );
                }
                let mut groups = FxHashMap::default();
                for name in re.capture_names().flatten() {
                    if let Some(m) = caps.name(name) {
                        groups.insert(name.to_string(), m.as_str().to_string());
                    }
                }
                Some((results, groups, match_start))
            }
        }
    }

    pub fn find_all(&self, input: &str) -> Vec<String> {
        if let Some(ref compiled) = self.compiled {
            match compiled {
                JsCompiledRegex::Simple(re) => re
                    .find_iter(input)
                    .map(|m| m.as_str().to_string())
                    .collect(),
                JsCompiledRegex::Advanced(re) => re
                    .find_iter(input)
                    .filter_map(|m| m.ok())
                    .map(|m| m.as_str().to_string())
                    .collect(),
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
                JsCompiledRegex::Advanced(re) => re
                    .find(input)
                    .ok()
                    .flatten()
                    .map(|m| m.start() as i64)
                    .unwrap_or(-1),
            }
        } else {
            -1
        }
    }

    pub fn split(&self, input: &str) -> Vec<String> {
        if let Some(ref compiled) = self.compiled {
            match compiled {
                JsCompiledRegex::Simple(re) => re.split(input).map(|s| s.to_string()).collect(),
                JsCompiledRegex::Advanced(re) => re
                    .split(input)
                    .filter_map(|s| s.ok())
                    .map(|s| s.to_string())
                    .collect(),
            }
        } else {
            vec![input.to_string()]
        }
    }

    /// Phase 3.4 - Fast-path: detect simple literal patterns (no regex
    /// metacharacters) and indicate they can use str::find directly,
    /// bypassing the regex crate entirely.
    pub fn is_literal_pattern(&self) -> bool {
        !self.source.contains('.')
            && !self.source.contains('^')
            && !self.source.contains('$')
            && !self.source.contains('*')
            && !self.source.contains('+')
            && !self.source.contains('?')
            && !self.source.contains('(')
            && !self.source.contains(')')
            && !self.source.contains('[')
            && !self.source.contains(']')
            && !self.source.contains('{')
            && !self.source.contains('}')
            && !self.source.contains('\\')
            && !self.source.contains('|')
    }

    /// Phase 3.4 — Lazy result cache hit for repeated `test()` calls on the
    /// same input string. If the input matches the cached string, reuse the
    /// cached match result instead of running the regex engine again.
    pub fn test_cached(&mut self, input: &str) -> bool {
        // Check if we have a cache hit
        if let Some(ref cached_input) = self.last_input {
            if cached_input.as_str() == input {
                if let (Some(start), Some(end)) = (self.last_match_start, self.last_match_end) {
                    // Cache hit: return whether a match was found
                    return end > start || (end == start && start == 0 && !input.is_empty());
                }
            }
        }

        // Cache miss: run the actual test
        let result = self.test(input);

        // Update cache
        if !self.global && !self.sticky {
            // Only cache for non-global, non-sticky regexps (test() semantics)
            self.last_input = Some(input.to_string());
            if result {
                // Find match positions for caching
                if let Some(ref compiled) = self.compiled {
                    let (match_start, match_end) = match compiled {
                        JsCompiledRegex::Simple(re) => re
                            .find(input)
                            .map(|m| (m.start(), m.end()))
                            .unwrap_or((0, 0)),
                        JsCompiledRegex::Advanced(re) => re
                            .find(input)
                            .ok()
                            .flatten()
                            .map(|m| (m.start(), m.end()))
                            .unwrap_or((0, 0)),
                    };
                    // Cache the match positions if we found a match
                    // (match_end > match_start) or if it's a zero-length match at position 0
                    if match_end > match_start || (match_start == 0 && result) {
                        self.last_match_start = Some(match_start);
                        self.last_match_end = Some(match_end);
                    } else {
                        self.last_match_start = None;
                        self.last_match_end = None;
                    }
                }
            } else {
                self.last_match_start = None;
                self.last_match_end = None;
            }
        }

        result
    }
}
