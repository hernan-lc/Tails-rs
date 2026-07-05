use crate::errors::Result;
use crate::objects::Value;
use crate::props;
use crate::vm::interpreter::Interpreter;

use super::helpers::{find_error_ctor_proto, find_error_proto, to_string_value};

macro_rules! error_constructor {
    ($name:ident, $type_name:expr, $proto_finder:expr) => {
        pub(super) fn $name(
            interp: &mut Interpreter,
            _this: &Value,
            args: &[Value],
        ) -> Result<Value> {
            let message = args
                .first()
                .map(|v| to_string_value(interp, v))
                .unwrap_or_default();
            let obj_idx = interp.heap.len();
            let stack = interp.build_stack_trace($type_name, &message);
            let props = props! {
                "message" => Value::String(message.clone()),
                "name" => Value::String($type_name.into()),
                "stack" => Value::String(stack),
            };
            let proto_idx = $proto_finder(interp);
            interp.heap.push(crate::vm::interpreter::HeapValue::Object(
                crate::vm::interpreter::JsObject {
                    properties: props,
                    prototype: proto_idx,
                    extensible: true,
                },
            ));
            Ok(Value::Object(obj_idx))
        }
    };
}

error_constructor!(native_error_constructor, "Error", find_error_ctor_proto);
error_constructor!(native_type_error_constructor, "TypeError", |i| {
    find_error_proto(i, "TypeError")
});
error_constructor!(native_reference_error_constructor, "ReferenceError", |i| {
    find_error_proto(i, "ReferenceError")
});
error_constructor!(native_syntax_error_constructor, "SyntaxError", |i| {
    find_error_proto(i, "SyntaxError")
});
error_constructor!(native_range_error_constructor, "RangeError", |i| {
    find_error_proto(i, "RangeError")
});
