use crate::errors::Result;
use crate::objects::Value;
use crate::props;
use crate::vm::interpreter::Interpreter;

use super::helpers::{find_error_ctor_proto, find_error_proto, to_string_value};
use crate::well_known as wk;

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
                wk::MESSAGE => Value::from_string(message.clone().into()),
                wk::NAME => Value::string($type_name),
                wk::STACK => Value::from_string(stack.into()),
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

error_constructor!(native_error_constructor, wk::ERROR, find_error_ctor_proto);
error_constructor!(native_type_error_constructor, wk::TYPE_ERROR, |i| {
    find_error_proto(i, wk::TYPE_ERROR)
});
error_constructor!(
    native_reference_error_constructor,
    wk::REFERENCE_ERROR,
    |i| { find_error_proto(i, wk::REFERENCE_ERROR) }
);
error_constructor!(native_syntax_error_constructor, wk::SYNTAX_ERROR, |i| {
    find_error_proto(i, wk::SYNTAX_ERROR)
});
error_constructor!(native_range_error_constructor, wk::RANGE_ERROR, |i| {
    find_error_proto(i, wk::RANGE_ERROR)
});
