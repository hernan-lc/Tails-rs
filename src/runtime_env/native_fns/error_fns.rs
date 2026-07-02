use rustc_hash::FxHashMap;
use crate::errors::Result;
use crate::objects::Value;
use crate::vm::interpreter::Interpreter;

use super::helpers::{find_error_ctor_proto, find_error_proto, to_string_value};

pub(super) fn native_error_constructor(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let message = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    let obj_idx = interp.heap.len();
    let mut props = FxHashMap::default();
    props.insert("message".into(), Value::String(message.clone()));
    props.insert("name".into(), Value::String("Error".into()));

    let stack = interp.build_stack_trace("Error", &message);
    props.insert("stack".into(), Value::String(stack));

    let proto_idx = find_error_ctor_proto(interp);

    interp.heap.push(crate::vm::interpreter::HeapValue::Object(
        crate::vm::interpreter::JsObject {
            properties: props,
            prototype: proto_idx,
            extensible: true,
        },
    ));
    Ok(Value::Object(obj_idx))
}

pub(super) fn native_type_error_constructor(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let message = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    let obj_idx = interp.heap.len();
    let mut props = FxHashMap::default();
    props.insert("message".into(), Value::String(message.clone()));
    props.insert("name".into(), Value::String("TypeError".into()));

    let stack = interp.build_stack_trace("TypeError", &message);
    props.insert("stack".into(), Value::String(stack));

    let proto_idx = find_error_proto(interp, "TypeError");
    interp.heap.push(crate::vm::interpreter::HeapValue::Object(
        crate::vm::interpreter::JsObject {
            properties: props,
            prototype: proto_idx,
            extensible: true,
        },
    ));
    Ok(Value::Object(obj_idx))
}

pub(super) fn native_reference_error_constructor(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let message = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    let obj_idx = interp.heap.len();
    let mut props = FxHashMap::default();
    props.insert("message".into(), Value::String(message.clone()));
    props.insert("name".into(), Value::String("ReferenceError".into()));

    let stack = interp.build_stack_trace("ReferenceError", &message);
    props.insert("stack".into(), Value::String(stack));

    let proto_idx = find_error_proto(interp, "ReferenceError");
    interp.heap.push(crate::vm::interpreter::HeapValue::Object(
        crate::vm::interpreter::JsObject {
            properties: props,
            prototype: proto_idx,
            extensible: true,
        },
    ));
    Ok(Value::Object(obj_idx))
}

pub(super) fn native_syntax_error_constructor(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let message = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    let obj_idx = interp.heap.len();
    let mut props = FxHashMap::default();
    props.insert("message".into(), Value::String(message.clone()));
    props.insert("name".into(), Value::String("SyntaxError".into()));

    let stack = interp.build_stack_trace("SyntaxError", &message);
    props.insert("stack".into(), Value::String(stack));

    let proto_idx = find_error_proto(interp, "SyntaxError");
    interp.heap.push(crate::vm::interpreter::HeapValue::Object(
        crate::vm::interpreter::JsObject {
            properties: props,
            prototype: proto_idx,
            extensible: true,
        },
    ));
    Ok(Value::Object(obj_idx))
}

pub(super) fn native_range_error_constructor(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let message = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    let obj_idx = interp.heap.len();
    let mut props = FxHashMap::default();
    props.insert("message".into(), Value::String(message.clone()));
    props.insert("name".into(), Value::String("RangeError".into()));

    let stack = interp.build_stack_trace("RangeError", &message);
    props.insert("stack".into(), Value::String(stack));

    let proto_idx = find_error_proto(interp, "RangeError");
    interp.heap.push(crate::vm::interpreter::HeapValue::Object(
        crate::vm::interpreter::JsObject {
            properties: props,
            prototype: proto_idx,
            extensible: true,
        },
    ));
    Ok(Value::Object(obj_idx))
}
