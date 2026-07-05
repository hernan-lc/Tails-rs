use crate::errors::Result;
use crate::objects::Value;
use crate::vm::interpreter::Interpreter;

use super::helpers::to_f64;

/// Macro for generating unary math functions that operate on the first argument.
///
/// # Example
///
/// ```ignore
/// unary_math_fn!(pub(super) fn native_math_abs, |n: f64| n.abs());
/// ```
///
/// Expands to:
/// ```ignore
/// pub(super) fn native_math_abs(...) -> Result<Value> {
///     let n = args.first().map(to_f64).unwrap_or(0.0);
///     Ok(Value::Float((n.abs())))
/// }
/// ```
macro_rules! unary_math_fn {
    (
        $vis:vis fn $name:ident,
        $op:expr
    ) => {
        $vis fn $name(
            _interp: &mut Interpreter,
            _this: &Value,
            args: &[Value],
        ) -> Result<Value> {
            let n = args.first().map(to_f64).unwrap_or(0.0);
            Ok(Value::Float($op(n)))
        }
    };
}

// Unary math functions (9 functions reduced to 9 one-line macro invocations)
unary_math_fn!(pub(super) fn native_math_abs, |n: f64| n.abs());
unary_math_fn!(pub(super) fn native_math_floor, |n: f64| n.floor());
unary_math_fn!(pub(super) fn native_math_ceil, |n: f64| n.ceil());
unary_math_fn!(pub(super) fn native_math_round, |n: f64| n.round());

pub(super) fn native_math_min(
    _interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let mut result = f64::INFINITY;
    for arg in args {
        let n = to_f64(arg);
        if n < result {
            result = n;
        }
    }
    Ok(Value::Float(result))
}

pub(super) fn native_math_max(
    _interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let mut result = f64::NEG_INFINITY;
    for arg in args {
        let n = to_f64(arg);
        if n > result {
            result = n;
        }
    }
    Ok(Value::Float(result))
}

pub(super) fn native_math_random(
    _interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    let s = RandomState::new();
    let mut hasher = s.build_hasher();
    hasher.write_u64(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64,
    );
    let bits = hasher.finish();
    Ok(Value::Float((bits as f64) / (u64::MAX as f64)))
}

pub(super) fn native_math_pow(
    _interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let base = args.first().map(to_f64).unwrap_or(0.0);
    let exp = args.get(1).map(to_f64).unwrap_or(0.0);
    Ok(Value::Float(base.powf(exp)))
}

unary_math_fn!(pub(super) fn native_math_sqrt, |n: f64| n.sqrt());
unary_math_fn!(pub(super) fn native_math_log, |n: f64| n.ln());
unary_math_fn!(pub(super) fn native_math_sin, |n: f64| n.sin());
unary_math_fn!(pub(super) fn native_math_cos, |n: f64| n.cos());
unary_math_fn!(pub(super) fn native_math_tan, |n: f64| n.tan());
unary_math_fn!(pub(super) fn native_math_trunc, |n: f64| n.trunc());

pub(super) fn native_math_sign(
    _interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let n = args.first().map(to_f64).unwrap_or(f64::NAN);
    if n == 0.0 {
        Ok(Value::Float(0.0))
    } else {
        Ok(Value::Float(n.signum()))
    }
}

pub(super) fn native_math_hypot(
    _interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let mut sum_sq = 0.0_f64;
    for arg in args {
        let n = to_f64(arg);
        sum_sq += n * n;
    }
    Ok(Value::Float(sum_sq.sqrt()))
}
