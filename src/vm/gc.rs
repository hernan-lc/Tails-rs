use crate::objects::Value;
use crate::vm::interpreter::HeapValue;
use rustc_hash::FxHashMap;
use std::collections::VecDeque;

pub struct GarbageCollector {
    free_list: VecDeque<usize>,
    marked: Vec<bool>,
    pub allocation_count: usize,
    pub threshold: usize,
    pub collections_performed: usize,
    pub bytes_freed: usize,
}

impl GarbageCollector {
    pub fn new() -> Self {
        Self {
            free_list: VecDeque::new(),
            marked: Vec::new(),
            allocation_count: 0,
            // Initial threshold raised from 8192 → 16384 to reduce the number
            // of early-GC pauses on small/medium programs (the per-iteration
            // `pc & 127 == 0 && should_collect()` check fires the same
            // `Vec::clone` of the heap, which dominates the tail of micro
            // benchmarks). The threshold still auto-scales up to 1M via the
            // `* 3 / 2` heuristic in `sweep`, so long-running programs are
            // unaffected.
            threshold: 16384,
            collections_performed: 0,
            bytes_freed: 0,
        }
    }

    pub fn should_collect(&self) -> bool {
        self.allocation_count >= self.threshold
    }

    pub fn allocate(&mut self, heap: &mut Vec<HeapValue>, value: HeapValue) -> usize {
        self.allocation_count += 1;
        if let Some(idx) = self.free_list.pop_front() {
            heap[idx] = value;
            self.marked[idx] = false;
            idx
        } else {
            let idx = heap.len();
            heap.push(value);
            self.marked.push(false);
            idx
        }
    }

    pub fn reset_marks(&mut self) {
        for m in &mut self.marked {
            *m = false;
        }
    }

    pub fn mark(&mut self, idx: usize, heap_len: usize) {
        if idx < self.marked.len() && idx < heap_len {
            self.marked[idx] = true;
        }
    }

    pub fn sweep(&mut self, heap: &mut [HeapValue]) -> usize {
        let mut freed = 0;
        let mut new_free_list = VecDeque::new();

        for i in 0..heap.len().min(self.marked.len()) {
            if !self.marked[i] {
                let old = std::mem::replace(
                    &mut heap[i],
                    HeapValue::Object(crate::vm::interpreter::JsObject::new()),
                );
                drop(old);
                new_free_list.push_back(i);
                freed += 1;
            }
        }

        self.free_list = new_free_list;
        self.allocation_count = 0;
        self.bytes_freed += freed;
        self.collections_performed += 1;

        const MAX_THRESHOLD: usize = 1_000_000;
        if self.threshold < MAX_THRESHOLD {
            self.threshold = (self.threshold * 3 / 2).min(MAX_THRESHOLD);
        }

        freed
    }

    pub(crate) fn collect(
        &mut self,
        heap: &mut [HeapValue],
        globals: &FxHashMap<String, Value>,
        stack: &[Value],
        call_stack: &[crate::vm::interpreter::CallFrame],
    ) -> usize {
        self.reset_marks();
        self.mark_roots(globals, stack, call_stack, heap);
        self.sweep(heap)
    }

    pub(crate) fn mark_roots(
        &mut self,
        globals: &FxHashMap<String, Value>,
        stack: &[Value],
        call_stack: &[crate::vm::interpreter::CallFrame],
        heap: &[HeapValue],
    ) {
        for value in globals.values() {
            self.mark_value(value);
        }

        for value in stack {
            self.mark_value(value);
        }

        for frame in call_stack {
            if let Some(func_idx) = frame.func_heap_idx {
                self.mark(func_idx, heap.len());
                if let Some(HeapValue::Function(f)) = heap.get(func_idx) {
                    for closure_val in f.closure.borrow().iter() {
                        self.mark_value(closure_val);
                    }
                    if let Some(ref super_class) = f.super_class {
                        self.mark_value(super_class);
                    }
                }
            }
            if let Some(ref this) = frame.this_value {
                self.mark_value(this);
            }
        }

        let mut worklist: Vec<usize> = Vec::new();
        for i in 0..self.marked.len().min(heap.len()) {
            if self.marked[i] {
                worklist.push(i);
            }
        }

        while let Some(idx) = worklist.pop() {
            if let Some(hv) = heap.get(idx) {
                match hv {
                    HeapValue::String(_) => {}
                    HeapValue::Object(obj) => {
                        for val in obj.properties.values() {
                            if let Some(child_idx) = heap_value_to_index(val) {
                                if !self.is_marked(child_idx, heap.len()) {
                                    self.mark(child_idx, heap.len());
                                    worklist.push(child_idx);
                                }
                            }
                        }
                        if let Some(proto) = obj.prototype {
                            if !self.is_marked(proto, heap.len()) {
                                self.mark(proto, heap.len());
                                worklist.push(proto);
                            }
                        }
                    }
                    HeapValue::Array(arr) => {
                        for val in &arr.elements {
                            if let Some(child_idx) = heap_value_to_index(val) {
                                if !self.is_marked(child_idx, heap.len()) {
                                    self.mark(child_idx, heap.len());
                                    worklist.push(child_idx);
                                }
                            }
                        }
                    }
                    HeapValue::Function(f) => {
                        for val in f.closure.borrow().iter() {
                            if let Some(child_idx) = heap_value_to_index(val) {
                                if !self.is_marked(child_idx, heap.len()) {
                                    self.mark(child_idx, heap.len());
                                    worklist.push(child_idx);
                                }
                            }
                        }
                        if let Some(ref ct) = f.captured_this {
                            if let Some(child_idx) = heap_value_to_index(ct) {
                                if !self.is_marked(child_idx, heap.len()) {
                                    self.mark(child_idx, heap.len());
                                    worklist.push(child_idx);
                                }
                            }
                        }
                        if let Some(proto) = f.prototype {
                            if !self.is_marked(proto, heap.len()) {
                                self.mark(proto, heap.len());
                                worklist.push(proto);
                            }
                        }
                        if let Some(ref sc) = f.super_class {
                            if let Some(child_idx) = heap_value_to_index(sc) {
                                if !self.is_marked(child_idx, heap.len()) {
                                    self.mark(child_idx, heap.len());
                                    worklist.push(child_idx);
                                }
                            }
                        }
                        for val in f.properties.values() {
                            if let Some(child_idx) = heap_value_to_index(val) {
                                if !self.is_marked(child_idx, heap.len()) {
                                    self.mark(child_idx, heap.len());
                                    worklist.push(child_idx);
                                }
                            }
                        }
                    }
                    HeapValue::Generator(g) => {
                        for val in &g.saved_stack {
                            if let Some(child_idx) = heap_value_to_index(val) {
                                if !self.is_marked(child_idx, heap.len()) {
                                    self.mark(child_idx, heap.len());
                                    worklist.push(child_idx);
                                }
                            }
                        }
                        if let Some(child_idx) = heap_value_to_index(&g.yield_value) {
                            if !self.is_marked(child_idx, heap.len()) {
                                self.mark(child_idx, heap.len());
                                worklist.push(child_idx);
                            }
                        }
                        if let Some(func_idx) = g.func_heap_idx {
                            if !self.is_marked(func_idx, heap.len()) {
                                self.mark(func_idx, heap.len());
                                worklist.push(func_idx);
                            }
                        }
                    }
                    HeapValue::Promise(p) => match &p.state {
                        crate::objects::js_promise::PromiseState::Fulfilled(v)
                        | crate::objects::js_promise::PromiseState::Rejected(v) => {
                            if let Some(child_idx) = heap_value_to_index(v) {
                                if !self.is_marked(child_idx, heap.len()) {
                                    self.mark(child_idx, heap.len());
                                    worklist.push(child_idx);
                                }
                            }
                        }
                        _ => {}
                    },
                    HeapValue::Proxy(proxy) => {
                        if let Some(child_idx) = heap_value_to_index(&proxy.target) {
                            if !self.is_marked(child_idx, heap.len()) {
                                self.mark(child_idx, heap.len());
                                worklist.push(child_idx);
                            }
                        }
                        if let Some(child_idx) = heap_value_to_index(&proxy.handler) {
                            if !self.is_marked(child_idx, heap.len()) {
                                self.mark(child_idx, heap.len());
                                worklist.push(child_idx);
                            }
                        }
                    }
                    HeapValue::TypedArray(_) => {
                        // TypedArray holds a `Vec<u8>` (the raw buffer) which is
                        // a value type, not a heap object. Nothing to trace.
                    }
                    HeapValue::Map(m) => {
                        // Trace keys (Vec<Value>) and values (Vec<Value>) so that
                        // objects referenced only by Map entries are not collected.
                        for val in &m.keys {
                            if let Some(child_idx) = heap_value_to_index(val) {
                                if !self.is_marked(child_idx, heap.len()) {
                                    self.mark(child_idx, heap.len());
                                    worklist.push(child_idx);
                                }
                            }
                        }
                        for val in &m.values {
                            if let Some(child_idx) = heap_value_to_index(val) {
                                if !self.is_marked(child_idx, heap.len()) {
                                    self.mark(child_idx, heap.len());
                                    worklist.push(child_idx);
                                }
                            }
                        }
                    }
                    HeapValue::Set(s) => {
                        // Trace values (Vec<Value>) so that objects referenced
                        // only by Set entries are not collected.
                        for val in &s.values {
                            if let Some(child_idx) = heap_value_to_index(val) {
                                if !self.is_marked(child_idx, heap.len()) {
                                    self.mark(child_idx, heap.len());
                                    worklist.push(child_idx);
                                }
                            }
                        }
                    }
                    HeapValue::WeakMap(_) => {}
                    HeapValue::WeakSet(_) => {}
                    HeapValue::Date(_) => {
                        // JsDate stores `utc_ms: f64` (value type) — nothing to trace.
                    }
                    HeapValue::RegExp(_) => {
                        // JsRegExp holds Strings and flags (value types) — nothing to trace.
                    }
                    HeapValue::Buffer(_) => {}
                    HeapValue::Iterator(iter) => {
                        if let Some(ref target) = iter.target {
                            if let Some(child_idx) = heap_value_to_index(target) {
                                if !self.is_marked(child_idx, heap.len()) {
                                    self.mark(child_idx, heap.len());
                                    worklist.push(child_idx);
                                }
                            }
                        }
                        if let Some(ref data) = iter.data {
                            if let Some(child_idx) = heap_value_to_index(data) {
                                if !self.is_marked(child_idx, heap.len()) {
                                    self.mark(child_idx, heap.len());
                                    worklist.push(child_idx);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn is_marked(&self, idx: usize, heap_len: usize) -> bool {
        idx < self.marked.len() && self.marked[idx] && idx < heap_len
    }

    pub fn mark_value(&mut self, value: &Value) {
        match value {
            Value::Object(idx)
            | Value::Array(idx)
            | Value::Function(idx)
            | Value::Promise(idx)
            | Value::Proxy(idx)
            | Value::TypedArray(idx)
            | Value::Map(idx)
            | Value::Set(idx)
            | Value::WeakMap(idx)
            | Value::WeakSet(idx)
            | Value::Date(idx)
            | Value::RegExp(idx)
            | Value::Buffer(idx)
                if *idx < self.marked.len() =>
            {
                self.marked[*idx] = true;
            }
            Value::Generator(idx) if *idx < self.marked.len() => {
                self.marked[*idx] = true;
            }
            // Phase 1.7: ConsString contains Value children (String/Cons),
            // not heap indices. No tracing needed — the children are either
            // inline strings or more Cons nodes.
            Value::Cons(_) => {}
            _ => {}
        }
    }

    pub fn set_threshold(&mut self, threshold: usize) {
        self.threshold = threshold;
    }

    pub fn live_count(&self, heap_len: usize) -> usize {
        heap_len - self.free_list.len()
    }

    pub fn free_count(&self) -> usize {
        self.free_list.len()
    }
}

/// Extract the heap index from any `Value` variant that points into the
/// heap. Returns `Some(idx)` for every variant whose `usize` payload is
/// a `HeapValue` index.
///
/// **Important:** this list must be kept in lockstep with
/// [`mark_value`](Self::mark_value), the `HeapValue::X` arms in
/// [`mark_roots`](Self::mark_roots), and the inverse `Value` enum in
/// `src/objects/mod.rs`. When a new `Value::X(idx)` variant is added
/// that wraps a `HeapValue::X` slot, it must be added here **and** in
/// `mark_value`, otherwise the GC will collect live objects held only
/// inside collections (Maps, Sets, Arrays, Object properties, …).
fn heap_value_to_index(value: &Value) -> Option<usize> {
    match value {
        Value::Object(idx)
        | Value::Array(idx)
        | Value::Function(idx)
        | Value::Promise(idx)
        | Value::Proxy(idx)
        | Value::Generator(idx)
        | Value::TypedArray(idx)
        | Value::Map(idx)
        | Value::Set(idx)
        | Value::WeakMap(idx)
        | Value::WeakSet(idx)
        | Value::Date(idx)
        | Value::RegExp(idx)
        | Value::Buffer(idx) => Some(*idx),
        // These do not index into `Interpreter.heap` and have no
        // heap-traceable payload:
        //   - Undefined, Null, Boolean, Integer, Float, BigInt, Symbol:
        //     immediate values
        //   - NativeFunction: index into `NATIVE_TABLE`, not `heap`
        //   - NativeObject: handle into the FFI `tails_abi` registry
        //   - Cons: rope tree containing only String/Cons children
        Value::Undefined
        | Value::Null
        | Value::Boolean(_)
        | Value::Integer(_)
        | Value::Float(_)
        | Value::BigInt(_)
        | Value::Symbol(_)
        | Value::String(_)
        | Value::Cons(_)
        | Value::NativeFunction(_)
        | Value::NativeObject(_) => None,
    }
}

impl Default for GarbageCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::interpreter::{JsArray, JsObject};

    fn make_obj() -> HeapValue {
        HeapValue::Object(JsObject::new())
    }

    fn make_arr() -> HeapValue {
        HeapValue::Array(JsArray { elements: vec![] })
    }

    #[test]
    fn test_gc_new() {
        let gc = GarbageCollector::new();
        assert_eq!(gc.allocation_count, 0);
        assert_eq!(gc.collections_performed, 0);
    }

    #[test]
    fn test_gc_allocate_reuses_free_slot() {
        let mut gc = GarbageCollector::new();
        let mut heap = Vec::new();

        let idx0 = gc.allocate(&mut heap, make_obj());
        let idx1 = gc.allocate(&mut heap, make_obj());
        assert_eq!(idx0, 0);
        assert_eq!(idx1, 1);

        gc.mark(0, heap.len());
        let freed = gc.sweep(&mut heap);
        assert_eq!(freed, 1);
        assert_eq!(gc.free_count(), 1);

        let idx2 = gc.allocate(&mut heap, make_obj());
        assert_eq!(idx2, 1);
        assert_eq!(gc.free_count(), 0);
    }

    #[test]
    fn test_gc_collect_preserves_reachable() {
        let mut gc = GarbageCollector::new();
        let mut heap = Vec::new();
        let globals = FxHashMap::default();

        let idx0 = gc.allocate(&mut heap, make_obj());
        let idx1 = gc.allocate(&mut heap, make_obj());
        let idx2 = gc.allocate(&mut heap, make_obj());

        heap[idx0] = HeapValue::Array(JsArray {
            elements: vec![Value::Object(idx1)],
        });
        heap[idx1] = HeapValue::Array(JsArray {
            elements: vec![Value::Object(idx2)],
        });

        let stack = vec![Value::Object(idx0)];

        gc.collect(&mut heap, &globals, &stack, &[]);

        assert!(gc.is_marked(idx0, heap.len()));
        assert!(gc.is_marked(idx1, heap.len()));
        assert!(gc.is_marked(idx2, heap.len()));
    }

    #[test]
    fn test_gc_collect_frees_unreachable() {
        let mut gc = GarbageCollector::new();
        let mut heap = Vec::new();
        let globals = FxHashMap::default();
        let stack = vec![];

        gc.allocate(&mut heap, make_obj());
        gc.allocate(&mut heap, make_obj());
        gc.allocate(&mut heap, make_arr());

        gc.collect(&mut heap, &globals, &stack, &[]);

        assert_eq!(gc.free_count(), 3);
        assert_eq!(heap.len(), 3);
    }

    #[test]
    fn test_gc_should_collect() {
        let mut gc = GarbageCollector::new();
        gc.set_threshold(3);
        assert!(!gc.should_collect());

        gc.allocation_count = 3;
        assert!(gc.should_collect());
    }

    #[test]
    fn test_gc_multiple_collections() {
        let mut gc = GarbageCollector::new();
        let mut heap = Vec::new();
        let globals = FxHashMap::default();
        let stack = vec![];

        gc.allocate(&mut heap, make_obj());
        gc.allocate(&mut heap, make_obj());
        gc.collect(&mut heap, &globals, &stack, &[]);
        assert_eq!(gc.collections_performed, 1);

        gc.allocate(&mut heap, make_obj());
        gc.allocate(&mut heap, make_obj());
        gc.collect(&mut heap, &globals, &stack, &[]);
        assert_eq!(gc.collections_performed, 2);
    }

    #[test]
    fn test_gc_chain_of_references() {
        let mut gc = GarbageCollector::new();
        let mut heap = Vec::new();
        let globals = FxHashMap::default();

        let idx0 = gc.allocate(&mut heap, make_obj());
        let idx1 = gc.allocate(&mut heap, make_obj());
        let idx2 = gc.allocate(&mut heap, make_obj());

        heap[idx0] = HeapValue::Array(JsArray {
            elements: vec![Value::Object(idx1)],
        });
        heap[idx1] = HeapValue::Array(JsArray {
            elements: vec![Value::Object(idx2)],
        });

        let stack = vec![Value::Object(idx0)];

        gc.collect(&mut heap, &globals, &stack, &[]);

        assert!(gc.is_marked(idx0, heap.len()));
        assert!(gc.is_marked(idx1, heap.len()));
        assert!(gc.is_marked(idx2, heap.len()));
    }

    #[test]
    fn test_gc_closure_references() {
        let mut gc = GarbageCollector::new();
        let mut heap = Vec::new();
        let globals = FxHashMap::default();

        let inner_obj_idx = gc.allocate(&mut heap, make_obj());
        let func_idx = gc.allocate(
            &mut heap,
            HeapValue::Function(crate::vm::interpreter::JsFunction {
                name: Some("test".into()),
                params: vec![],
                rest_param: None,
                bytecode_index: 0,
                closure: std::rc::Rc::new(std::cell::RefCell::new(vec![Value::Object(
                    inner_obj_idx,
                )])),
                prototype: None,
                super_class: None,
                properties: FxHashMap::default(),
                owner_module: None,
                module_scope: None,
                is_generator: false,
                source_file: None,
                source_line: None,
                is_arrow: false,
                captured_this: None,
            }),
        );

        let stack = vec![Value::Function(func_idx)];

        gc.collect(&mut heap, &globals, &stack, &[]);

        assert!(gc.is_marked(func_idx, heap.len()));
        assert!(gc.is_marked(inner_obj_idx, heap.len()));
    }
}
