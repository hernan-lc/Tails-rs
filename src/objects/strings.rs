use std::cell::{Cell, RefCell};
use std::sync::Arc;

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

/// Phase 1.7 + Phase 1.8: ConsString — a binary tree (rope) representation
/// for deferred string concatenation.
///
/// Children use `Arc<Value>` for O(1) clone (atomic refcount) instead of
/// deep-copying the rope. This replaces the previous custom `SharedValue`
/// raw pointer/refcount (which required `unsafe` and had a broken layout).
///
/// Other improvements:
/// - `cached` stores a lazily-computed flattened `String` so repeated
///   calls to `flatten()` are O(1) after the first.
/// - `cached_hash` stores a pre-computed `u64` hash so `Hash` impls
///   avoid repeated flatten+hash.
/// - Short concatenations (both children are flat strings <= 64 bytes
///   total) are eagerly flattened to avoid building a tree node that
///   would immediately need flattening anyway.
///
/// # Thread safety
///
/// `Arc` shares ownership safely; `RefCell`/`Cell` cache fields must only be
/// accessed from one thread at a time (the VM thread). `Send`/`Sync` are
/// asserted so `Value` can live in thread-safe containers.
pub struct ConsString {
    left: Arc<Value>,
    right: Arc<Value>,
    pub total_len: usize,
    cached: RefCell<Option<String>>,
    cached_hash: Cell<u64>,
}

// SAFETY: See struct docs. Cache cells are VM-thread-only.
unsafe impl Send for ConsString {}
unsafe impl Sync for ConsString {}

impl Clone for ConsString {
    fn clone(&self) -> Self {
        Self {
            left: Arc::clone(&self.left),
            right: Arc::clone(&self.right),
            total_len: self.total_len,
            cached: RefCell::new(None),
            cached_hash: Cell::new(0),
        }
    }
}

impl std::fmt::Debug for ConsString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConsString")
            .field("left", self.left.as_ref())
            .field("right", self.right.as_ref())
            .field("total_len", &self.total_len)
            .finish()
    }
}

impl ConsString {
    pub fn new(left: Value, right: Value) -> Self {
        let total_len = Self::value_len(&left) + Self::value_len(&right);
        Self {
            left: Arc::new(left),
            right: Arc::new(right),
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
                    left: Arc::new(Value::from_string(buf)),
                    right: Arc::new(Value::string("")),
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
        self.left.as_ref()
    }

    pub fn right(&self) -> &Value {
        self.right.as_ref()
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
