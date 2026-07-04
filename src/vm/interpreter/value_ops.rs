use super::Interpreter;
use crate::errors::{Error, Result};
use crate::objects::{ConsString, Value};

macro_rules! compare_values {
    ($name:ident, $op:tt) => {
        pub(super) fn $name(&self, left: &Value, right: &Value) -> Result<bool> {
            match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => Ok(a $op b),
                (Value::String(a), Value::String(b)) => Ok(a $op b),
                (Value::String(a), Value::Cons(c)) => Ok(a.as_str() $op c.flatten().as_str()),
                (Value::Cons(c), Value::String(b)) => Ok(c.flatten().as_str() $op b.as_str()),
                (Value::Cons(a), Value::Cons(b)) => Ok(a.flatten() $op b.flatten()),
                (Value::BigInt(a), Value::BigInt(b)) => Ok(a $op b),
                _ => {
                    let l = self.to_number(left)?;
                    let r = self.to_number(right)?;
                    Ok(l $op r)
                }
            }
        }
    };
}

impl Interpreter {
    pub(super) fn add(&self, left: Value, right: Value) -> Result<Value> {
        match (&left, &right) {
            (Value::Integer(a), Value::Integer(b)) => {
                if let Some(result) = a.checked_add(*b) {
                    Ok(Value::Integer(result))
                } else {
                    Ok(Value::Float(*a as f64 + *b as f64))
                }
            }
            // Phase 1.7: String + String — O(1) ConsString rope node.
            // Avoids allocating a fresh `String` of `a.len() + b.len()` bytes.
            // The actual flat string is only produced when consumed by an
            // operation that needs contiguous bytes (comparisons, display, etc.).
            (Value::String(a), Value::String(b)) => Ok(Value::Cons(ConsString::new(
                Value::String(a.clone()),
                Value::String(b.clone()),
            ))),
            (Value::String(a), Value::Cons(c)) => Ok(Value::Cons(ConsString::new(
                Value::String(a.clone()),
                Value::Cons(c.clone()),
            ))),
            (Value::Cons(c), Value::String(b)) => Ok(Value::Cons(ConsString::new(
                Value::Cons(c.clone()),
                Value::String(b.clone()),
            ))),
            (Value::Cons(a), Value::Cons(b)) => Ok(Value::Cons(ConsString::new(
                Value::Cons(a.clone()),
                Value::Cons(b.clone()),
            ))),
            // Phase 1.7: String + Number — coerce number to string, then
            // build a Cons node. The number-to-string conversion is cheap
            // (format! on a small primitive) and avoids the larger
            // `String::with_capacity` + two `push_str` allocation.
            (Value::String(a), Value::Integer(b)) => {
                let b_str = Value::String(b.to_string());
                Ok(Value::Cons(ConsString::new(
                    Value::String(a.clone()),
                    b_str,
                )))
            }
            (Value::Integer(a), Value::String(b)) => {
                let a_str = Value::String(a.to_string());
                Ok(Value::Cons(ConsString::new(
                    a_str,
                    Value::String(b.clone()),
                )))
            }
            (Value::String(a), Value::Float(b)) => {
                let b_str = if b.is_finite() && *b == (*b as i64) as f64 {
                    Value::String((*b as i64).to_string())
                } else {
                    Value::String(b.to_string())
                };
                Ok(Value::Cons(ConsString::new(
                    Value::String(a.clone()),
                    b_str,
                )))
            }
            (Value::Float(a), Value::String(b)) => {
                let a_str = if a.is_finite() && *a == (*a as i64) as f64 {
                    Value::String((*a as i64).to_string())
                } else {
                    Value::String(a.to_string())
                };
                Ok(Value::Cons(ConsString::new(
                    a_str,
                    Value::String(b.clone()),
                )))
            }
            // Cons + Number: flatten the Cons first, then build a new Cons
            (Value::Cons(c), Value::Integer(b)) => {
                let left = Value::Cons(c.clone());
                let right = Value::String(b.to_string());
                Ok(Value::Cons(ConsString::new(left, right)))
            }
            (Value::Integer(a), Value::Cons(c)) => {
                let left = Value::String(a.to_string());
                let right = Value::Cons(c.clone());
                Ok(Value::Cons(ConsString::new(left, right)))
            }
            (Value::Cons(c), Value::Float(b)) => {
                let right = if b.is_finite() && *b == (*b as i64) as f64 {
                    Value::String((*b as i64).to_string())
                } else {
                    Value::String(b.to_string())
                };
                Ok(Value::Cons(ConsString::new(Value::Cons(c.clone()), right)))
            }
            (Value::Float(a), Value::Cons(c)) => {
                let left = if a.is_finite() && *a == (*a as i64) as f64 {
                    Value::String((*a as i64).to_string())
                } else {
                    Value::String(a.to_string())
                };
                Ok(Value::Cons(ConsString::new(left, Value::Cons(c.clone()))))
            }
            // Cold path: coerce left operand to string if needed, then
            // build a Cons node.
            (Value::String(a), r) => {
                let coerced = self.to_string_coerce_value(r).flatten();
                Ok(Value::Cons(ConsString::new(
                    Value::String(a.clone()),
                    Value::String(coerced),
                )))
            }
            (l, Value::String(b)) => {
                let coerced = self.to_string_coerce_value(l).flatten();
                Ok(Value::Cons(ConsString::new(
                    Value::String(coerced),
                    Value::String(b.clone()),
                )))
            }
            (Value::Cons(c), r) => {
                let coerced = self.to_string_coerce_value(r).flatten();
                Ok(Value::Cons(ConsString::new(
                    Value::Cons(c.clone()),
                    Value::String(coerced),
                )))
            }
            (l, Value::Cons(c)) => {
                let coerced = self.to_string_coerce_value(l).flatten();
                Ok(Value::Cons(ConsString::new(
                    Value::String(coerced),
                    Value::Cons(c.clone()),
                )))
            }
            (Value::BigInt(a), Value::BigInt(b)) => Ok(Value::BigInt(a + b)),
            _ => {
                let l = self.to_number(&left)?;
                let r = self.to_number(&right)?;
                Ok(Value::Float(l + r))
            }
        }
    }

    pub(crate) fn to_string_coerce(&self, value: &Value) -> String {
        match value {
            Value::Undefined => "undefined".to_string(),
            Value::Null => "null".to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::Integer(n) => n.to_string(),
            Value::Float(n) => {
                if *n == (*n as i64) as f64 && n.is_finite() {
                    (*n as i64).to_string()
                } else {
                    n.to_string()
                }
            }
            Value::String(s) => s.clone(),
            Value::Cons(c) => c.flatten(),
            Value::BigInt(n) => format!("{}", n),
            Value::Function(idx) => {
                if let crate::vm::interpreter::HeapValue::Function(f) = &self.heap[*idx] {
                    let name = f.name.as_deref().unwrap_or("");
                    if f.prototype.is_some() && f.super_class.is_some() {
                        format!("class {} {{}}", name)
                    } else if !name.is_empty() {
                        format!("function {}() {{ [native code] }}", name)
                    } else {
                        "function () {{ [native code] }}".to_string()
                    }
                } else {
                    "function () {{ [native code] }}".to_string()
                }
            }
            Value::NativeFunction(_) => "function () {{ [native code] }}".to_string(),
            Value::Object(_) => "[object Object]".to_string(),
            Value::Array(_) => "[object Array]".to_string(),
            _ => value.to_string(),
        }
    }

    /// Phase 1.7: like `to_string_coerce` but returns a `Value::String`
    /// directly, avoiding an extra clone for ConsString construction.
    pub(crate) fn to_string_coerce_value(&self, value: &Value) -> Value {
        match value {
            Value::String(_) | Value::Cons(_) => value.clone(),
            _ => Value::String(self.to_string_coerce(value)),
        }
    }

    pub(super) fn sub(&self, left: Value, right: Value) -> Result<Value> {
        match (&left, &right) {
            (Value::Integer(a), Value::Integer(b)) => {
                if let Some(result) = a.checked_sub(*b) {
                    Ok(Value::Integer(result))
                } else {
                    Ok(Value::Float(*a as f64 - *b as f64))
                }
            }
            (Value::BigInt(a), Value::BigInt(b)) => Ok(Value::BigInt(a - b)),
            _ => {
                let l = self.to_number(&left)?;
                let r = self.to_number(&right)?;
                Ok(Value::Float(l - r))
            }
        }
    }

    pub(super) fn mul(&self, left: Value, right: Value) -> Result<Value> {
        match (&left, &right) {
            (Value::Integer(a), Value::Integer(b)) => {
                if let Some(result) = a.checked_mul(*b) {
                    Ok(Value::Integer(result))
                } else {
                    Ok(Value::Float(*a as f64 * *b as f64))
                }
            }
            (Value::BigInt(a), Value::BigInt(b)) => Ok(Value::BigInt(a * b)),
            _ => {
                let l = self.to_number(&left)?;
                let r = self.to_number(&right)?;
                Ok(Value::Float(l * r))
            }
        }
    }

    pub(super) fn div(&self, left: Value, right: Value) -> Result<Value> {
        match (&left, &right) {
            (Value::BigInt(a), Value::BigInt(b)) => {
                if *b == 0 {
                    return Err(Error::RuntimeError(super::ERR_DIV_BY_ZERO.into()));
                }
                Ok(Value::BigInt(a / b))
            }
            _ => {
                let l = self.to_number(&left)?;
                let r = self.to_number(&right)?;
                if r == 0.0 {
                    return Err(Error::RuntimeError(super::ERR_DIV_BY_ZERO.into()));
                }
                Ok(Value::Float(l / r))
            }
        }
    }

    pub(super) fn modulo(&self, left: Value, right: Value) -> Result<Value> {
        match (&left, &right) {
            (Value::BigInt(a), Value::BigInt(b)) => {
                if *b == 0 {
                    return Err(Error::RuntimeError(super::ERR_DIV_BY_ZERO.into()));
                }
                Ok(Value::BigInt(a % b))
            }
            _ => {
                let l = self.to_number(&left)?;
                let r = self.to_number(&right)?;
                if r == 0.0 {
                    return Err(Error::RuntimeError(super::ERR_DIV_BY_ZERO.into()));
                }
                Ok(Value::Float(l % r))
            }
        }
    }

    pub(super) fn power(&self, left: Value, right: Value) -> Result<Value> {
        match (&left, &right) {
            (Value::BigInt(a), Value::BigInt(b)) => {
                if *b < 0 {
                    return Err(Error::TypeError(
                        "BigInt negative exponent not allowed".into(),
                    ));
                }
                Ok(Value::BigInt(a.pow(*b as u32)))
            }
            _ => {
                let l = self.to_number(&left)?;
                let r = self.to_number(&right)?;
                Ok(Value::Float(l.powf(r)))
            }
        }
    }

    pub(super) fn negate(&self, value: Value) -> Result<Value> {
        match &value {
            Value::BigInt(n) => Ok(Value::BigInt(-n)),
            _ => {
                let n = self.to_number(&value)?;
                Ok(Value::Float(-n))
            }
        }
    }

    pub(crate) fn is_truthy(&self, value: &Value) -> bool {
        match value {
            Value::Undefined => false,
            Value::Null => false,
            Value::Boolean(b) => *b,
            Value::Integer(n) => *n != 0,
            Value::Float(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Cons(c) => c.total_len > 0,
            Value::BigInt(n) => *n != 0,
            _ => true,
        }
    }

    pub(super) fn is_equal(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Undefined, Value::Undefined) => true,
            (Value::Null, Value::Null) => true,
            (Value::Null, Value::Undefined) => true,
            (Value::Undefined, Value::Null) => true,
            (Value::Boolean(a), _) => {
                self.is_equal(&Value::Float(if *a { 1.0 } else { 0.0 }), right)
            }
            (_, Value::Boolean(b)) => {
                self.is_equal(left, &Value::Float(if *b { 1.0 } else { 0.0 }))
            }
            (Value::String(a), Value::String(b)) => a == b,
            (Value::String(a), Value::Cons(c)) => a.as_str() == c.flatten().as_str(),
            (Value::Cons(c), Value::String(b)) => c.flatten().as_str() == b.as_str(),
            (Value::Cons(a), Value::Cons(b)) => a.flatten() == b.flatten(),
            (Value::String(s), _) => {
                let num = s.parse::<f64>().unwrap_or(f64::NAN);
                self.is_equal(&Value::Float(num), right)
            }
            (Value::Cons(c), _) => {
                let flat = c.flatten();
                let num = flat.parse::<f64>().unwrap_or(f64::NAN);
                self.is_equal(&Value::Float(num), right)
            }
            (_, Value::String(s)) => {
                let num = s.parse::<f64>().unwrap_or(f64::NAN);
                self.is_equal(left, &Value::Float(num))
            }
            (_, Value::Cons(c)) => {
                let flat = c.flatten();
                let num = flat.parse::<f64>().unwrap_or(f64::NAN);
                self.is_equal(left, &Value::Float(num))
            }
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b && !a.is_nan() && !b.is_nan(),
            (Value::Integer(a), Value::Float(b)) => *a as f64 == *b && !b.is_nan(),
            (Value::Float(a), Value::Integer(b)) => *a == *b as f64 && !a.is_nan(),
            (Value::BigInt(a), Value::BigInt(b)) => a == b,
            _ => false,
        }
    }

    compare_values!(less_than, <);
    compare_values!(greater_than, >);
    compare_values!(less_than_or_equal, <=);
    compare_values!(greater_than_or_equal, >=);

    pub(super) fn to_number(&self, value: &Value) -> Result<f64> {
        match value {
            Value::Integer(n) => Ok(*n as f64),
            Value::Float(n) => Ok(*n),
            Value::Boolean(b) => Ok(if *b { 1.0 } else { 0.0 }),
            Value::Null => Ok(0.0),
            Value::Undefined => Ok(f64::NAN),
            Value::String(s) => {
                if s.is_empty() {
                    Ok(0.0)
                } else {
                    Ok(s.parse::<f64>().unwrap_or(f64::NAN))
                }
            }
            Value::Cons(c) => {
                let flat = c.flatten();
                if flat.is_empty() {
                    Ok(0.0)
                } else {
                    Ok(flat.parse::<f64>().unwrap_or(f64::NAN))
                }
            }
            _ => Ok(f64::NAN),
        }
    }

    fn format_function(&self, idx: &usize) -> String {
        if let crate::vm::interpreter::HeapValue::Function(f) = &self.heap[*idx] {
            let name = f.name.as_deref().unwrap_or("");
            if f.prototype.is_some() && f.super_class.is_some() {
                format!("[class {}]", name)
            } else if !name.is_empty() {
                format!("[Function: {}]", name)
            } else {
                "[Function]".to_string()
            }
        } else {
            "[Function]".to_string()
        }
    }

    pub(super) fn value_to_string_raw(&self, value: &Value) -> String {
        match value {
            Value::Undefined => "undefined".to_string(),
            Value::Null => "null".to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::Integer(n) => n.to_string(),
            Value::Float(n) => {
                if *n == (*n as i64) as f64 {
                    (*n as i64).to_string()
                } else {
                    n.to_string()
                }
            }
            Value::String(s) => s.clone(),
            Value::Cons(c) => c.flatten(),
            Value::BigInt(n) => format!("{}n", n),
            Value::Symbol(id) => format!("Symbol({})", id),
            Value::Function(idx) => self.format_function(idx),
            Value::NativeFunction(_) => "[NativeFunction]".to_string(),
            Value::Object(_) => "[Object]".to_string(),
            Value::Array(_) => "[Array]".to_string(),
            Value::Promise(_) => "[Promise]".to_string(),
            Value::Proxy(_) => "[Proxy]".to_string(),
            Value::Generator(_) => "[Generator]".to_string(),
            Value::TypedArray(_) => "[TypedArray]".to_string(),
            Value::Map(_) => "[Map]".to_string(),
            Value::Set(_) => "[Set]".to_string(),
            Value::WeakMap(_) => "[WeakMap]".to_string(),
            Value::WeakSet(_) => "[WeakSet]".to_string(),
            Value::Date(_) => "[Date]".to_string(),
            Value::RegExp(_) => "[RegExp]".to_string(),
            Value::Buffer(_) => "[Buffer]".to_string(),
            Value::NativeObject(_) => "[NativeObject]".to_string(),
        }
    }

    pub(super) fn value_to_string(&self, value: &Value) -> String {
        match value {
            Value::Undefined => "undefined".to_string(),
            Value::Null => "null".to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::Integer(n) => n.to_string(),
            Value::Float(n) => n.to_string(),
            Value::String(s) => format!("\"{}\"", s),
            Value::Cons(c) => format!("\"{}\"", c.flatten()),
            Value::BigInt(n) => format!("{}n", n),
            Value::Symbol(id) => format!("Symbol({})", id),
            Value::Function(idx) => self.format_function(idx),
            Value::NativeFunction(_) => "[NativeFunction]".to_string(),
            Value::Object(_) => "[Object]".to_string(),
            Value::Array(_) => "[Array]".to_string(),
            Value::Promise(_) => "[Promise]".to_string(),
            Value::Proxy(_) => "[Proxy]".to_string(),
            Value::Generator(_) => "[Generator]".to_string(),
            Value::TypedArray(_) => "[TypedArray]".to_string(),
            Value::Map(_) => "[Map]".to_string(),
            Value::Set(_) => "[Set]".to_string(),
            Value::WeakMap(_) => "[WeakMap]".to_string(),
            Value::WeakSet(_) => "[WeakSet]".to_string(),
            Value::Date(_) => "[Date]".to_string(),
            Value::RegExp(_) => "[RegExp]".to_string(),
            Value::Buffer(_) => "[Buffer]".to_string(),
            Value::NativeObject(_) => "[NativeObject]".to_string(),
        }
    }
}
