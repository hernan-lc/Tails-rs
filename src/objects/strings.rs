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

/// Phase 1.7: ConsString — a binary tree (rope) representation for
/// deferred string concatenation. Instead of allocating a fresh
/// `String` of `a.len() + b.len()` bytes on every `+`, we build a
/// lazy tree node. The actual flat `String` is only produced when the
/// value is consumed by an operation that requires contiguous bytes
/// (comparisons, display, `to_number`, etc.).
///
/// `total_len` is cached for O(1) `is_truthy` / `length` checks.
#[derive(Debug, Clone)]
pub struct ConsString {
    pub left: Box<Value>,
    pub right: Box<Value>,
    pub total_len: usize,
}

impl ConsString {
    pub fn new(left: Value, right: Value) -> Self {
        let total_len = Self::value_len(&left) + Self::value_len(&right);
        Self {
            left: Box::new(left),
            right: Box::new(right),
            total_len,
        }
    }

    fn value_len(v: &Value) -> usize {
        match v {
            Value::String(s) => s.len(),
            Value::Cons(c) => c.total_len,
            _ => 0,
        }
    }

    /// Flatten the tree into a single `String`. Uses a stack-based
    /// iterative approach to avoid stack overflow on deep trees.
    pub fn flatten(&self) -> String {
        let mut result = String::with_capacity(self.total_len);
        let mut stack: Vec<&Value> = vec![&self.right, &self.left];
        while let Some(node) = stack.pop() {
            match node {
                Value::String(s) => result.push_str(s),
                Value::Cons(c) => {
                    stack.push(&c.right);
                    stack.push(&c.left);
                }
                _ => {}
            }
        }
        result
    }

    /// Write the flattened string into an existing buffer without
    /// allocating a new String.
    pub fn flatten_into(&self, buf: &mut String) {
        let mut stack: Vec<&Value> = vec![&self.right, &self.left];
        while let Some(node) = stack.pop() {
            match node {
                Value::String(s) => buf.push_str(s),
                Value::Cons(c) => {
                    stack.push(&c.right);
                    stack.push(&c.left);
                }
                _ => {}
            }
        }
    }
}
