use super::Interpreter;
use crate::errors::{Error, Result};
use crate::objects::Value;

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
            // Hot path: String + String — skip the `to_string_coerce`
            // round-trip and the 24-byte String clone that the general
            // `(Value::String(a), r)` arm would do, since the right
            // operand is already a `String`. This is the single most
            // common `+` pattern in benchmark `eval_string_concat_*`
            // (`s = s + "x"` loops, where `"x"` is a LoadConst).
            (Value::String(a), Value::String(b)) => {
                let mut result = String::with_capacity(a.len() + b.len());
                result.push_str(a);
                result.push_str(b);
                Ok(Value::String(result))
            }
            (Value::String(a), r) => {
                let coerced = self.to_string_coerce(r);
                let mut result = String::with_capacity(a.len() + coerced.len());
                result.push_str(a);
                result.push_str(&coerced);
                Ok(Value::String(result))
            }
            (l, Value::String(b)) => {
                let coerced = self.to_string_coerce(l);
                let mut result = String::with_capacity(coerced.len() + b.len());
                result.push_str(&coerced);
                result.push_str(b);
                Ok(Value::String(result))
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
                    return Err(Error::RuntimeError("Division by zero".into()));
                }
                Ok(Value::BigInt(a / b))
            }
            _ => {
                let l = self.to_number(&left)?;
                let r = self.to_number(&right)?;
                if r == 0.0 {
                    return Err(Error::RuntimeError("Division by zero".into()));
                }
                Ok(Value::Float(l / r))
            }
        }
    }

    pub(super) fn modulo(&self, left: Value, right: Value) -> Result<Value> {
        match (&left, &right) {
            (Value::BigInt(a), Value::BigInt(b)) => {
                if *b == 0 {
                    return Err(Error::RuntimeError("Division by zero".into()));
                }
                Ok(Value::BigInt(a % b))
            }
            _ => {
                let l = self.to_number(&left)?;
                let r = self.to_number(&right)?;
                if r == 0.0 {
                    return Err(Error::RuntimeError("Division by zero".into()));
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
            (Value::String(s), _) => {
                let num = s.parse::<f64>().unwrap_or(f64::NAN);
                self.is_equal(&Value::Float(num), right)
            }
            (_, Value::String(s)) => {
                let num = s.parse::<f64>().unwrap_or(f64::NAN);
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

    pub(super) fn less_than(&self, left: &Value, right: &Value) -> Result<bool> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(a < b),
            (Value::String(a), Value::String(b)) => Ok(a < b),
            (Value::BigInt(a), Value::BigInt(b)) => Ok(a < b),
            _ => {
                let l = self.to_number(left)?;
                let r = self.to_number(right)?;
                Ok(l < r)
            }
        }
    }

    pub(super) fn greater_than(&self, left: &Value, right: &Value) -> Result<bool> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(a > b),
            (Value::String(a), Value::String(b)) => Ok(a > b),
            (Value::BigInt(a), Value::BigInt(b)) => Ok(a > b),
            _ => {
                let l = self.to_number(left)?;
                let r = self.to_number(right)?;
                Ok(l > r)
            }
        }
    }

    pub(super) fn less_than_or_equal(&self, left: &Value, right: &Value) -> Result<bool> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(a <= b),
            (Value::String(a), Value::String(b)) => Ok(a <= b),
            (Value::BigInt(a), Value::BigInt(b)) => Ok(a <= b),
            _ => {
                let l = self.to_number(left)?;
                let r = self.to_number(right)?;
                Ok(l <= r)
            }
        }
    }

    pub(super) fn greater_than_or_equal(&self, left: &Value, right: &Value) -> Result<bool> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(a >= b),
            (Value::String(a), Value::String(b)) => Ok(a >= b),
            (Value::BigInt(a), Value::BigInt(b)) => Ok(a >= b),
            _ => {
                let l = self.to_number(left)?;
                let r = self.to_number(right)?;
                Ok(l >= r)
            }
        }
    }

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
            _ => Ok(f64::NAN),
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
            Value::BigInt(n) => format!("{}n", n),
            Value::Symbol(id) => format!("Symbol({})", id),
            Value::Function(idx) => {
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
            Value::BigInt(n) => format!("{}n", n),
            Value::Symbol(id) => format!("Symbol({})", id),
            Value::Function(idx) => {
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
