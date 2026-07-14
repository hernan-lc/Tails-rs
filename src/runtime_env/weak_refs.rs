use rustc_hash::FxHashMap;

pub struct WeakRefManager {
    refs: FxHashMap<usize, WeakRef>,
    finalizers: Vec<FinalizationRegistry>,
    next_id: usize,
}

pub struct WeakRef {
    pub id: usize,
    pub target: Option<usize>,
}

pub struct FinalizationRegistry {
    pub target: usize,
    pub callback: usize,
}

impl WeakRefManager {
    pub fn new() -> Self {
        Self {
            refs: FxHashMap::default(),
            finalizers: Vec::new(),
            next_id: 0,
        }
    }

    pub fn create_weak_ref(&mut self, target: usize) -> usize {
        let id = self.next_id;
        self.next_id += 1;

        self.refs.insert(
            id,
            WeakRef {
                id,
                target: Some(target),
            },
        );

        id
    }

    pub fn deref(&self, id: usize) -> Option<usize> {
        self.refs.get(&id).and_then(|r| r.target)
    }

    pub fn register_finalizer(&mut self, target: usize, callback: usize) {
        self.finalizers
            .push(FinalizationRegistry { target, callback });
    }

    pub fn cleanup(&mut self) {
        self.refs.retain(|_, r| r.target.is_some());
    }
}

impl Default for WeakRefManager {
    fn default() -> Self {
        Self::new()
    }
}

use crate::errors::Result;
use crate::objects::Value;
use crate::runtime_env::native_fns::constants as c;
use crate::vm::interpreter::{HeapValue, Interpreter, JsObject};

/// `new WeakRef(target)` — stores the target so `.deref()` returns it.
/// We don't implement GC-based clearing (the host has no generational GC
/// for JS objects), so the ref always resolves.
pub(super) fn native_weak_ref_constructor(
    interp: &mut Interpreter,
    this: &Value,
    args: &[Value],
) -> Result<Value> {
    let target = args.first().cloned().unwrap_or(Value::Undefined);
    let target_idx = match target {
        Value::Object(idx) => idx,
        other => {
            // Non-object targets (primitives) are held by value.
            let idx = interp
                .gc
                .allocate(&mut interp.heap, HeapValue::Object(JsObject::new()));
            if let HeapValue::Object(obj) = &mut interp.heap[idx] {
                obj.properties.insert("__value".into(), other);
            }
            idx
        }
    };
    let props = crate::props! {
        "deref" => Value::NativeFunction(c::WEAK_REF_DEREF),
        "__target" => Value::Object(target_idx),
    };
    let idx = interp.gc.allocate(
        &mut interp.heap,
        HeapValue::Object(JsObject {
            properties: props,
            prototype: None,
            extensible: true,
        }),
    );
    let _ = this;
    Ok(Value::Object(idx))
}

/// `weakRef.deref()` — returns the referenced target (or undefined if cleared).
pub(super) fn native_weak_ref_deref(
    interp: &mut Interpreter,
    this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let obj_idx = match this {
        Value::Object(idx) => *idx,
        _ => return Ok(Value::Undefined),
    };
    if let HeapValue::Object(obj) = &interp.heap[obj_idx] {
        if let Some(Value::Object(t)) = obj.properties.get("__target") {
            return Ok(Value::Object(*t));
        }
        if let Some(v) = obj.properties.get("__value") {
            return Ok(v.clone());
        }
    }
    Ok(Value::Undefined)
}

/// `new FinalizationRegistry(cb)` — no-op holder (no GC finalization hook).
pub(super) fn native_finalization_registry_constructor(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let cb = args.first().cloned().unwrap_or(Value::Undefined);
    let props = crate::props! {
        "register" => Value::NativeFunction(c::FINALIZATION_REGISTRY_REGISTER),
        "__cb" => cb,
    };
    let idx = interp.gc.allocate(
        &mut interp.heap,
        HeapValue::Object(JsObject {
            properties: props,
            prototype: None,
            extensible: true,
        }),
    );
    Ok(Value::Object(idx))
}

/// `registry.register(target, heldValue[, unregisterToken])` — no-op.
pub(super) fn native_finalization_registry_register(
    _interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    Ok(Value::Undefined)
}
