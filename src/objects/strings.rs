use std::cell::{Cell, RefCell};

use crate::objects::Value;

/// Well-known symbol IDs (small numbers to distinguish from user symbols)
pub const SYMBOL_ITERATOR: u64 = 1;
pub const SYMBOL_TO_STRING_TAG: u64 = 2;
pub const SYMBOL_HAS_INSTANCE: u64 = 3;
pub const SYMBOL_TO_PRIMITIVE: u64 = 4;
pub const SYMBOL_SPECIES: u64 = 5;
pub const SYMBOL_UNSCOPABLES: u64 = 6;
pub const SYMBOL_ASYNC_ITERATOR: u64 = 7;
/// Starting ID for user-created symbols
pub const USER_SYMBOL_START: u64 = 1000;

/// Threshold in bytes: if both children are flat strings totaling at most
/// this many bytes, eagerly flatten instead of building a tree node.
const EAGER_FLATTEN_THRESHOLD: usize = 64;

/// A reference-counted pointer to a `Value` that is heap-allocated and
/// shared by reference. Unlike `Box`, cloning this pointer is O(1)
/// (just incrementing a refcount) instead of O(N) (deep copy).
///
/// # Safety
///
/// `SharedValue` is `!Send` and `!Sync` — it must only be used from
/// the single VM thread. The refcount is non-atomic for performance.
struct SharedValue {
    ptr: *mut Value,
    count: Cell<usize>,
}

impl SharedValue {
    fn new(val: Value) -> Self {
        let boxed = Box::new(val);
        Self {
            ptr: Box::into_raw(boxed),
            count: Cell::new(1),
        }
    }

    fn clone_ref(&self) -> SharedValue {
        self.count.set(self.count.get() + 1);
        SharedValue {
            ptr: self.ptr,
            count: Cell::new(self.count.get()),
        }
    }

    fn deref(&self) -> &Value {
        unsafe { &*self.ptr }
    }
}

impl Drop for SharedValue {
    fn drop(&mut self) {
        let c = self.count.get();
        if c == 1 {
            unsafe {
                drop(Box::from_raw(self.ptr));
            }
        } else {
            self.count.set(c - 1);
        }
    }
}

impl std::fmt::Debug for SharedValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.deref(), f)
    }
}

/// Phase 1.7 + Phase 1.8: ConsString — a binary tree (rope) representation
/// for deferred string concatenation.
///
/// Phase 1.8 improvements:
/// - Children use `SharedValue` (O(1) clone via refcount) instead of
///   `Box` (O(N) deep copy). This eliminates the quadratic blowup in
///   loops like `for (...) s = s + 'x'` where the GC snapshots the stack.
/// - `cached` stores a lazily-computed flattened `String` so repeated
///   calls to `flatten()` are O(1) after the first.
/// - `cached_hash` stores a pre-computed `u64` hash so `Hash` impls
///   avoid repeated flatten+hash.
/// - Short concatenations (both children are flat strings <= 64 bytes
///   total) are eagerly flattened to avoid building a tree node that
///   would immediately need flattening anyway.
pub struct ConsString {
    left: SharedValue,
    right: SharedValue,
    pub total_len: usize,
    cached: RefCell<Option<String>>,
    cached_hash: Cell<u64>,
}

// SAFETY: ConsString is only accessed from the single VM thread.
// The RefCell and Cell are used for lazy caching and are never
// accessed concurrently.
unsafe impl Sync for ConsString {}
unsafe impl Send for ConsString {}

impl Clone for ConsString {
    fn clone(&self) -> Self {
        Self {
            left: self.left.clone_ref(),
            right: self.right.clone_ref(),
            total_len: self.total_len,
            cached: RefCell::new(None),
            cached_hash: Cell::new(0),
        }
    }
}

impl std::fmt::Debug for ConsString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConsString")
            .field("left", self.left.deref())
            .field("right", self.right.deref())
            .field("total_len", &self.total_len)
            .finish()
    }
}

impl ConsString {
    pub fn new(left: Value, right: Value) -> Self {
        let total_len = Self::value_len(&left) + Self::value_len(&right);
        Self {
            left: SharedValue::new(left),
            right: SharedValue::new(right),
            total_len,
            cached: RefCell::new(None),
            cached_hash: Cell::new(0),
        }
    }

    /// Build a ConsString, but if both children are short flat strings,
    /// eagerly flatten to avoid tree depth explosion.
    pub fn new_smart(left: Value, right: Value) -> Self {
        match (&left, &right) {
            (Value::String(a), Value::String(b))
                if a.len() + b.len() <= EAGER_FLATTEN_THRESHOLD =>
            {
                let mut buf = String::with_capacity(a.len() + b.len());
                buf.push_str(a);
                buf.push_str(b);
                let total_len = buf.len();
                Self {
                    left: SharedValue::new(Value::String(buf)),
                    right: SharedValue::new(Value::String(String::new())),
                    total_len,
                    cached: RefCell::new(None),
                    cached_hash: Cell::new(0),
                }
            }
            _ => Self::new(left, right),
        }
    }

    fn value_len(v: &Value) -> usize {
        match v {
            Value::String(s) => s.len(),
            Value::Cons(c) => c.total_len,
            _ => 0,
        }
    }

    pub fn left(&self) -> &Value {
        self.left.deref()
    }

    pub fn right(&self) -> &Value {
        self.right.deref()
    }

    /// Flatten the tree into a single `String`. Memoized: the first call
    /// computes and caches the result; subsequent calls return the cached
    /// value in O(1).
    pub fn flatten(&self) -> String {
        if let Some(ref cached) = *self.cached.borrow() {
            return cached.clone();
        }
        let flat = self.flatten_uncached();
        *self.cached.borrow_mut() = Some(flat.clone());
        flat
    }

    /// Compute the flattened string without checking the cache.
    fn flatten_uncached(&self) -> String {
        let mut result = String::with_capacity(self.total_len);
        let mut stack: Vec<&Value> = vec![self.right(), self.left()];
        while let Some(node) = stack.pop() {
            match node {
                Value::String(s) => result.push_str(s),
                Value::Cons(c) => {
                    stack.push(c.right());
                    stack.push(c.left());
                }
                _ => {}
            }
        }
        result
    }

    /// Write the flattened string into an existing buffer without
    /// allocating a new String. Uses the cache if available.
    pub fn flatten_into(&self, buf: &mut String) {
        if let Some(ref cached) = *self.cached.borrow() {
            buf.push_str(cached);
            return;
        }
        let mut stack: Vec<&Value> = vec![self.right(), self.left()];
        while let Some(node) = stack.pop() {
            match node {
                Value::String(s) => buf.push_str(s),
                Value::Cons(c) => {
                    stack.push(c.right());
                    stack.push(c.left());
                }
                _ => {}
            }
        }
    }

    /// Compute and cache a 64-bit hash of the flattened string.
    pub fn hash_cons_string(&self) -> u64 {
        let h = self.cached_hash.get();
        if h != 0 {
            return h;
        }
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let flat = self.flatten_uncached();
        let mut hasher = DefaultHasher::new();
        flat.hash(&mut hasher);
        let h = hasher.finish();
        let h = if h == 0 { 1 } else { h };
        self.cached_hash.set(h);
        *self.cached.borrow_mut() = Some(flat);
        h
    }

    /// Compare with a flat string: check lengths first, then content.
    pub fn eq_flat(&self, other_flat: &str) -> bool {
        if self.total_len != other_flat.len() {
            return false;
        }
        self.flatten().as_str() == other_flat
    }

    pub fn eq_cons(&self, other: &ConsString) -> bool {
        if self.total_len != other.total_len {
            return false;
        }
        self.flatten() == other.flatten()
    }

    pub fn lt_flat(&self, other_flat: &str) -> bool {
        self.flatten().as_str() < other_flat
    }

    pub fn lt_cons(&self, other: &ConsString) -> bool {
        self.flatten() < other.flatten()
    }
}

impl Drop for ConsString {
    fn drop(&mut self) {
        // SharedValue handles its own refcount-based deallocation.
        // Nothing extra needed here.
    }
}
