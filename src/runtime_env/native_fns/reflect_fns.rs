use crate::errors::{Error, Result};
use crate::objects::Value;
use crate::props;
use crate::vm::interpreter::Interpreter;
use crate::well_known as wk;

/// Converts an internal property key string back to the appropriate Value.
/// Keys like `"__sym_123"` are converted to `Value::Symbol(123)`.
/// All other keys remain as `Value::String(key)`.
fn prop_key_to_value(key: &str) -> Value {
    if let Some(id_str) = key.strip_prefix("__sym_") {
        if let Ok(id) = id_str.parse::<u64>() {
            return Value::Symbol(id);
        }
    }
    Value::from_string(key.to_string())
}

pub(super) fn native_reflect_get(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let target = args.first().cloned().unwrap_or(Value::Undefined);
    let property = args.get(1).cloned().unwrap_or(Value::Undefined);
    let receiver = args.get(2).cloned().unwrap_or(target.clone());
    interp.get_property_with_this(&target, &property, &receiver)
}

pub(super) fn native_reflect_set(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let target = args.first().cloned().unwrap_or(Value::Undefined);
    let property = args.get(1).cloned().unwrap_or(Value::Undefined);
    let value = args.get(2).cloned().unwrap_or(Value::Undefined);
    let _receiver = args.get(3).cloned().unwrap_or(target.clone());

    // Delegate to the shared set_property path so arrays, accessors, and
    // length updates all behave consistently.
    match interp.set_property(&target, &property, value) {
        Ok(()) => Ok(Value::Boolean(true)),
        Err(_) => Ok(Value::Boolean(false)),
    }
}

pub(super) fn native_reflect_has(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let target = args.first().cloned().unwrap_or(Value::Undefined);
    let property = args.get(1).cloned().unwrap_or(Value::Undefined);
    interp.in_check_mut(&property, &target)
}

pub(super) fn native_reflect_delete_property(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let target = args.first().cloned().unwrap_or(Value::Undefined);
    let property = args.get(1).cloned().unwrap_or(Value::Undefined);
    Ok(interp.delete_property(&target, &property))
}

pub(super) fn native_reflect_apply(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let target = args.first().cloned().unwrap_or(Value::Undefined);
    let this_arg = args.get(1).cloned().unwrap_or(Value::Undefined);
    let arguments_list = match args.get(2) {
        Some(Value::Array(arr_idx)) => {
            if let crate::vm::interpreter::HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                arr.elements.clone()
            } else {
                Vec::new()
            }
        }
        _ => Vec::new(),
    };
    interp.call_value(&target, &this_arg, &arguments_list)
}

pub(super) fn native_reflect_construct(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let target = args.first().cloned().unwrap_or(Value::Undefined);
    let arguments_list = match args.get(1) {
        Some(Value::Array(arr_idx)) => {
            if let crate::vm::interpreter::HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                arr.elements.clone()
            } else {
                Vec::new()
            }
        }
        _ => Vec::new(),
    };
    let _new_target = args.get(2).cloned().unwrap_or(target.clone());

    match &target {
        Value::Function(func_idx) => {
            let f = if let crate::vm::interpreter::HeapValue::Function(f) = &interp.heap[*func_idx]
            {
                f.clone()
            } else {
                return Err(Error::TypeError("Not a constructor".into()));
            };

            let proto_idx = f.prototype;
            let new_obj_heap_idx = interp.heap.len();
            interp.heap.push(crate::vm::interpreter::HeapValue::Object(
                crate::vm::interpreter::JsObject::with_prototype(proto_idx),
            ));
            let this_val = Value::Object(new_obj_heap_idx);

            if f.bytecode_index == usize::MAX {
                return Ok(this_val);
            }

            let return_address = interp
                .current_module
                .as_ref()
                .map(|m| m.instructions.len())
                .unwrap_or(0);
            let base_pointer = interp.stack.len();
            let closure_count = f.closure.borrow().len();

            interp.call_stack.push(crate::vm::interpreter::CallFrame {
                return_address,
                base_pointer,
                closure_var_count: closure_count,
                func_heap_idx: Some(*func_idx),
                this_value: Some(this_val.clone()),
                is_construct: true,
                source_name: None,
                generator_heap_idx: None,
                source_line: None,
                source_col: None,
                exception_handlers_snapshot: interp.exception_handlers.clone(),
                arguments: None,
            });

            for closure_var in f.closure.borrow().iter().cloned() {
                interp.stack.push(closure_var);
            }
            for arg in arguments_list {
                interp.stack.push(arg);
            }

            if let Some(module) = interp.current_module.clone() {
                interp.execute_from(&module, f.bytecode_index)?;
            }

            Ok(interp.stack.pop().unwrap_or(this_val))
        }
        Value::NativeFunction(native_idx) => {
            let proto_idx = interp.find_native_prototype(*native_idx);
            let new_obj_heap_idx = interp.heap.len();
            interp.heap.push(crate::vm::interpreter::HeapValue::Object(
                crate::vm::interpreter::JsObject::with_prototype(proto_idx),
            ));
            let this_val = Value::Object(new_obj_heap_idx);
            let result = interp.call_native(*native_idx, &this_val, &arguments_list)?;
            match result {
                Value::Object(_) | Value::Array(_) | Value::Function(_) | Value::Promise(_) => {
                    Ok(result)
                }
                _ => Ok(this_val),
            }
        }
        _ => Err(Error::TypeError("Not a constructor".into())),
    }
}

pub(super) fn native_reflect_own_keys(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let target = args.first().cloned().unwrap_or(Value::Undefined);
    let mut keys = Vec::new();
    match &target {
        Value::Object(obj_idx) => {
            if let crate::vm::interpreter::HeapValue::Object(obj) = &interp.heap[*obj_idx] {
                for k in super::helpers::collect_own_enumerable_keys(&obj.properties) {
                    keys.push(prop_key_to_value(&k));
                }
            }
        }
        Value::Array(arr_idx) => {
            if let crate::vm::interpreter::HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                for i in 0..arr.elements.len() {
                    keys.push(Value::from_string(i.to_string()));
                }
                keys.push(Value::from_string(wk::LENGTH.to_string()));
            }
        }
        Value::Function(func_idx) => {
            if let crate::vm::interpreter::HeapValue::Function(f) = &interp.heap[*func_idx] {
                for k in super::helpers::collect_own_enumerable_keys(&f.properties) {
                    keys.push(prop_key_to_value(&k));
                }
            }
        }
        _ => {}
    }
    let heap_idx = interp.heap.len();
    interp.heap.push(crate::vm::interpreter::HeapValue::Array(
        crate::vm::interpreter::JsArray { elements: keys },
    ));
    Ok(Value::Array(heap_idx))
}

pub(super) fn native_reflect_get_own_property_descriptor(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let target = args.first().cloned().unwrap_or(Value::Undefined);
    let property = match args.get(1) {
        Some(Value::String(s)) => s.to_string(),
        _ => return Ok(Value::Undefined),
    };

    match &target {
        Value::Object(obj_idx) => {
            if let crate::vm::interpreter::HeapValue::Object(obj) = &interp.heap[*obj_idx] {
                let getter_key = format!("__getter_{}", property);
                let setter_key = format!("__setter_{}", property);
                let getter = obj.properties.get(&getter_key).cloned();
                let setter = obj.properties.get(&setter_key).cloned();
                let mut data = obj.properties.get(&property).cloned();

                if let Some(data) = data.take() {
                    if getter.is_none() && setter.is_none() {
                        let descriptor = props! {
                            "value" => data,
                            "writable" => Value::Boolean(true),
                            "enumerable" => Value::Boolean(true),
                            "configurable" => Value::Boolean(true),
                        };
                        let desc_idx = interp.heap.len();
                        interp.heap.push(crate::vm::interpreter::HeapValue::Object(
                            crate::vm::interpreter::JsObject {
                                properties: descriptor,
                                prototype: None,
                                extensible: true,
                            },
                        ));
                        return Ok(Value::Object(desc_idx));
                    }
                    if getter.is_some() || setter.is_some() {
                        let descriptor = props! {
                            "get" => getter.unwrap_or(Value::Undefined),
                            "set" => setter.unwrap_or(Value::Undefined),
                            "enumerable" => Value::Boolean(true),
                            "configurable" => Value::Boolean(true),
                        };
                        let desc_idx = interp.heap.len();
                        interp.heap.push(crate::vm::interpreter::HeapValue::Object(
                            crate::vm::interpreter::JsObject {
                                properties: descriptor,
                                prototype: None,
                                extensible: true,
                            },
                        ));
                        return Ok(Value::Object(desc_idx));
                    }
                    // Data property even if somehow accessors linger — prefer data.
                    let descriptor = props! {
                        "value" => data,
                        "writable" => Value::Boolean(true),
                        "enumerable" => Value::Boolean(true),
                        "configurable" => Value::Boolean(true),
                    };
                    let desc_idx = interp.heap.len();
                    interp.heap.push(crate::vm::interpreter::HeapValue::Object(
                        crate::vm::interpreter::JsObject {
                            properties: descriptor,
                            prototype: None,
                            extensible: true,
                        },
                    ));
                    return Ok(Value::Object(desc_idx));
                }
                if getter.is_some() || setter.is_some() {
                    let descriptor = props! {
                        "get" => getter.unwrap_or(Value::Undefined),
                        "set" => setter.unwrap_or(Value::Undefined),
                        "enumerable" => Value::Boolean(true),
                        "configurable" => Value::Boolean(true),
                    };
                    let desc_idx = interp.heap.len();
                    interp.heap.push(crate::vm::interpreter::HeapValue::Object(
                        crate::vm::interpreter::JsObject {
                            properties: descriptor,
                            prototype: None,
                            extensible: true,
                        },
                    ));
                    return Ok(Value::Object(desc_idx));
                }
                if getter.is_some() || setter.is_some() {
                    let descriptor = props! {
                        "get" => getter.unwrap_or(Value::Undefined),
                        "set" => setter.unwrap_or(Value::Undefined),
                        "enumerable" => Value::Boolean(true),
                        "configurable" => Value::Boolean(true),
                    };
                    let desc_idx = interp.heap.len();
                    interp.heap.push(crate::vm::interpreter::HeapValue::Object(
                        crate::vm::interpreter::JsObject {
                            properties: descriptor,
                            prototype: None,
                            extensible: true,
                        },
                    ));
                    return Ok(Value::Object(desc_idx));
                }
                // Data property even if somehow accessors linger — prefer data.
                if let Some(val) = data {
                    let descriptor = props! {
                        "value" => val,
                        "writable" => Value::Boolean(true),
                        "enumerable" => Value::Boolean(true),
                        "configurable" => Value::Boolean(true),
                    };
                    let desc_idx = interp.heap.len();
                    interp.heap.push(crate::vm::interpreter::HeapValue::Object(
                        crate::vm::interpreter::JsObject {
                            properties: descriptor,
                            prototype: None,
                            extensible: true,
                        },
                    ));
                    return Ok(Value::Object(desc_idx));
                }
            }
        }
        Value::Array(arr_idx) => {
            if let crate::vm::interpreter::HeapValue::Array(arr) = &interp.heap[*arr_idx] {
                if property.as_str() == wk::LENGTH {
                    let descriptor = props! {
                        "value" => Value::Float(arr.elements.len() as f64),
                        "writable" => Value::Boolean(false),
                        "enumerable" => Value::Boolean(false),
                        "configurable" => Value::Boolean(false),
                    };
                    let desc_idx = interp.heap.len();
                    interp.heap.push(crate::vm::interpreter::HeapValue::Object(
                        crate::vm::interpreter::JsObject {
                            properties: descriptor,
                            prototype: None,
                            extensible: true,
                        },
                    ));
                    return Ok(Value::Object(desc_idx));
                }
                if let Ok(index) = property.parse::<usize>() {
                    if index < arr.elements.len() {
                        let descriptor = props! {
                            "value" => arr.elements[index].clone(),
                            "writable" => Value::Boolean(true),
                            "enumerable" => Value::Boolean(true),
                            "configurable" => Value::Boolean(true),
                        };
                        let desc_idx = interp.heap.len();
                        interp.heap.push(crate::vm::interpreter::HeapValue::Object(
                            crate::vm::interpreter::JsObject {
                                properties: descriptor,
                                prototype: None,
                                extensible: true,
                            },
                        ));
                        return Ok(Value::Object(desc_idx));
                    }
                }
            }
        }
        Value::Function(func_idx) => {
            if let crate::vm::interpreter::HeapValue::Function(f) = &interp.heap[*func_idx] {
                let getter_key = format!("__getter_{}", property);
                let setter_key = format!("__setter_{}", property);
                let getter = f.properties.get(&getter_key).cloned();
                let setter = f.properties.get(&setter_key).cloned();
                let data = f.properties.get(&property).cloned();

                if let Some(data) = data {
                    if getter.is_none() && setter.is_none() {
                        let descriptor = props! {
                            "value" => data,
                            "writable" => Value::Boolean(true),
                            "enumerable" => Value::Boolean(true),
                            "configurable" => Value::Boolean(true),
                        };
                        let desc_idx = interp.heap.len();
                        interp.heap.push(crate::vm::interpreter::HeapValue::Object(
                            crate::vm::interpreter::JsObject {
                                properties: descriptor,
                                prototype: None,
                                extensible: true,
                            },
                        ));
                        return Ok(Value::Object(desc_idx));
                    }
                }
                if getter.is_some() || setter.is_some() {
                    let descriptor = props! {
                        "get" => getter.unwrap_or(Value::Undefined),
                        "set" => setter.unwrap_or(Value::Undefined),
                        "enumerable" => Value::Boolean(true),
                        "configurable" => Value::Boolean(true),
                    };
                    let desc_idx = interp.heap.len();
                    interp.heap.push(crate::vm::interpreter::HeapValue::Object(
                        crate::vm::interpreter::JsObject {
                            properties: descriptor,
                            prototype: None,
                            extensible: true,
                        },
                    ));
                    return Ok(Value::Object(desc_idx));
                }
            }
        }
        _ => {}
    }
    Ok(Value::Undefined)
}

pub(super) fn native_reflect_define_property(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let target = args.first().cloned().unwrap_or(Value::Undefined);
    let property = match args.get(1) {
        Some(Value::String(s)) => s.to_string(),
        _ => return Ok(Value::Boolean(false)),
    };
    let descriptor = args.get(2).cloned().unwrap_or(Value::Undefined);

    let value = match &descriptor {
        Value::Object(obj_idx) => {
            if let crate::vm::interpreter::HeapValue::Object(obj) = &interp.heap[*obj_idx] {
                obj.properties.get("value").cloned()
            } else {
                None
            }
        }
        _ => None,
    };

    match &target {
        Value::Object(obj_idx) => {
            if let Some(val) = value {
                if let crate::vm::interpreter::HeapValue::Object(obj) = &mut interp.heap[*obj_idx] {
                    obj.properties.insert(property.to_string(), val);
                    return Ok(Value::Boolean(true));
                }
            }
            Ok(Value::Boolean(false))
        }
        Value::Function(func_idx) => {
            if let Some(val) = value {
                if let crate::vm::interpreter::HeapValue::Function(f) = &mut interp.heap[*func_idx]
                {
                    f.properties.insert(property.to_string(), val);
                    return Ok(Value::Boolean(true));
                }
            }
            Ok(Value::Boolean(false))
        }
        _ => Ok(Value::Boolean(false)),
    }
}

pub(super) fn native_reflect_get_prototype_of(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let target = args.first().cloned().unwrap_or(Value::Undefined);
    match &target {
        Value::Object(obj_idx) => {
            if let crate::vm::interpreter::HeapValue::Object(obj) = &interp.heap[*obj_idx] {
                match obj.prototype {
                    Some(proto_idx) => Ok(Value::Object(proto_idx)),
                    None => Ok(Value::Null),
                }
            } else {
                Ok(Value::Null)
            }
        }
        Value::Function(func_idx) => {
            if let crate::vm::interpreter::HeapValue::Function(f) = &interp.heap[*func_idx] {
                if let Some(proto) = f.properties.get("__[[Prototype]]__") {
                    return Ok(proto.clone());
                }
                if let Some(proto_idx) = interp.function_proto_idx {
                    return Ok(Value::Object(proto_idx));
                }
            }
            Ok(Value::Null)
        }
        _ => Ok(Value::Null),
    }
}

pub(super) fn native_reflect_set_prototype_of(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let target = args.first().cloned().unwrap_or(Value::Undefined);
    let proto = args.get(1).cloned().unwrap_or(Value::Null);

    match &target {
        Value::Object(obj_idx) => {
            let new_proto = match &proto {
                Value::Object(p_idx) => Some(*p_idx),
                Value::Null => None,
                _ => return Ok(Value::Boolean(false)),
            };
            if let crate::vm::interpreter::HeapValue::Object(obj) = &mut interp.heap[*obj_idx] {
                obj.prototype = new_proto;
                return Ok(Value::Boolean(true));
            }
            Ok(Value::Boolean(false))
        }
        Value::Function(func_idx) => {
            // Functions also have an internal [[Prototype]] chain (used by
            // Express Router: `Object.setPrototypeOf(router, this)`).
            let new_proto = match &proto {
                Value::Object(p_idx) => Some(Value::Object(*p_idx)),
                Value::Function(p_idx) => Some(Value::Function(*p_idx)),
                Value::Null => None,
                _ => return Ok(Value::Boolean(false)),
            };
            if let crate::vm::interpreter::HeapValue::Function(f) = &mut interp.heap[*func_idx] {
                match new_proto {
                    Some(p) => {
                        f.properties.insert("__[[Prototype]]__".into(), p);
                    }
                    None => {
                        f.properties.remove("__[[Prototype]]__");
                    }
                }
                return Ok(Value::Boolean(true));
            }
            Ok(Value::Boolean(false))
        }
        _ => Ok(Value::Boolean(false)),
    }
}

pub(super) fn native_reflect_is_extensible(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let target = args.first().cloned().unwrap_or(Value::Undefined);
    match &target {
        Value::Object(obj_idx) => {
            if let crate::vm::interpreter::HeapValue::Object(obj) = &interp.heap[*obj_idx] {
                Ok(Value::Boolean(obj.extensible))
            } else {
                Ok(Value::Boolean(false))
            }
        }
        Value::Array(arr_idx) => {
            if let crate::vm::interpreter::HeapValue::Array(_) = &interp.heap[*arr_idx] {
                Ok(Value::Boolean(true))
            } else {
                Ok(Value::Boolean(false))
            }
        }
        Value::Function(func_idx) => {
            if let crate::vm::interpreter::HeapValue::Function(f) = &interp.heap[*func_idx] {
                // Functions are extensible by default; we don't track extensibility on functions yet
                let _ = f;
                Ok(Value::Boolean(true))
            } else {
                Ok(Value::Boolean(false))
            }
        }
        _ => Ok(Value::Boolean(false)),
    }
}

pub(super) fn native_reflect_prevent_extensions(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let target = args.first().cloned().unwrap_or(Value::Undefined);
    match &target {
        Value::Object(obj_idx) => {
            if let crate::vm::interpreter::HeapValue::Object(obj) = &mut interp.heap[*obj_idx] {
                obj.extensible = false;
                return Ok(Value::Boolean(true));
            }
            Ok(Value::Boolean(false))
        }
        _ => Ok(Value::Boolean(false)),
    }
}
