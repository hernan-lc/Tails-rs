use super::{HeapValue, Interpreter, JsObject};
use crate::objects::js_array::{TypedArray, TypedArrayType};
use crate::objects::Value;
use crate::runtime_env::native_fns::constants as c;
use std::collections::HashMap;

impl Interpreter {
    pub(super) fn init_builtins(&mut self) {
        // Global constants
        self.globals
            .insert("Infinity".into(), Value::Float(f64::INFINITY));

        // globalThis — an object that represents the global scope
        let global_this_idx = self
            .gc
            .allocate(&mut self.heap, HeapValue::Object(JsObject::new()));
        self.globals
            .insert("globalThis".into(), Value::Object(global_this_idx));
        self.globals
            .insert("-Infinity".into(), Value::Float(f64::NEG_INFINITY));

        // Global functions
        self.globals
            .insert("parseInt".into(), Value::NativeFunction(c::PARSE_INT));
        self.globals
            .insert("parseFloat".into(), Value::NativeFunction(c::PARSE_FLOAT));
        self.globals
            .insert("isNaN".into(), Value::NativeFunction(c::IS_NAN));
        self.globals
            .insert("isFinite".into(), Value::NativeFunction(c::IS_FINITE));

        // Timer stubs
        self.globals
            .insert("setTimeout".into(), Value::NativeFunction(c::SET_TIMEOUT));
        self.globals
            .insert("setInterval".into(), Value::NativeFunction(c::SET_INTERVAL));
        self.globals.insert(
            "clearTimeout".into(),
            Value::NativeFunction(c::CLEAR_TIMEOUT),
        );
        self.globals.insert(
            "clearInterval".into(),
            Value::NativeFunction(c::CLEAR_INTERVAL),
        );

        // CommonJS require() — NativeFunction(c::REQUIRE)
        self.globals
            .insert("require".into(), Value::NativeFunction(c::REQUIRE));

        // console object
        let mut console_props = HashMap::new();
        console_props.insert("log".into(), Value::NativeFunction(c::CONSOLE_LOG));
        console_props.insert("warn".into(), Value::NativeFunction(c::CONSOLE_WARN));
        console_props.insert("error".into(), Value::NativeFunction(c::CONSOLE_ERROR));
        console_props.insert("info".into(), Value::NativeFunction(c::CONSOLE_INFO));
        console_props.insert("table".into(), Value::NativeFunction(c::CONSOLE_TABLE));
        console_props.insert("dir".into(), Value::NativeFunction(c::CONSOLE_DIR));
        console_props.insert("group".into(), Value::NativeFunction(c::CONSOLE_GROUP));
        console_props.insert(
            "groupEnd".into(),
            Value::NativeFunction(c::CONSOLE_GROUP_END),
        );
        console_props.insert(
            "groupCollapsed".into(),
            Value::NativeFunction(c::CONSOLE_GROUP_COLLAPSED),
        );
        console_props.insert("time".into(), Value::NativeFunction(c::CONSOLE_TIME));
        console_props.insert("timeEnd".into(), Value::NativeFunction(c::CONSOLE_TIME_END));
        console_props.insert("assert".into(), Value::NativeFunction(c::CONSOLE_ASSERT));
        console_props.insert("clear".into(), Value::NativeFunction(c::CONSOLE_CLEAR));
        console_props.insert("trace".into(), Value::NativeFunction(c::CONSOLE_INFO)); // Use info for now
        console_props.insert("count".into(), Value::NativeFunction(c::CONSOLE_INFO)); // Use info for now
        console_props.insert("countReset".into(), Value::NativeFunction(c::CONSOLE_INFO)); // Use info for now
        console_props.insert("debug".into(), Value::NativeFunction(c::CONSOLE_LOG)); // Use log for now
        console_props.insert("profile".into(), Value::NativeFunction(c::CONSOLE_INFO)); // Use info for now
        console_props.insert("profileEnd".into(), Value::NativeFunction(c::CONSOLE_INFO)); // Use info for now
        console_props.insert("timeLog".into(), Value::NativeFunction(c::CONSOLE_TIME_END)); // Use timeEnd for now
        let console_obj_idx = self.gc.allocate(
            &mut self.heap,
            HeapValue::Object(JsObject {
                properties: console_props,
                prototype: None,
                extensible: true,
            }),
        );
        self.globals
            .insert("console".into(), Value::Object(console_obj_idx));

        // Object
        let mut object_props = HashMap::new();
        object_props.insert("keys".into(), Value::NativeFunction(c::OBJECT_KEYS));
        object_props.insert("values".into(), Value::NativeFunction(c::OBJECT_VALUES));
        object_props.insert("entries".into(), Value::NativeFunction(c::OBJECT_ENTRIES));
        object_props.insert("assign".into(), Value::NativeFunction(c::OBJECT_ASSIGN));
        object_props.insert(
            "defineProperty".into(),
            Value::NativeFunction(c::OBJECT_DEFINE_PROPERTY),
        );
        object_props.insert(
            "getOwnPropertyDescriptor".into(),
            Value::NativeFunction(c::OBJECT_GET_OWN_PROPERTY_DESCRIPTOR),
        );
        object_props.insert("freeze".into(), Value::NativeFunction(c::OBJECT_FREEZE));
        object_props.insert("is".into(), Value::NativeFunction(c::OBJECT_IS));
        object_props.insert(
            "preventExtensions".into(),
            Value::NativeFunction(c::OBJECT_PREVENT_EXTENSIONS),
        );
        object_props.insert(
            "isExtensible".into(),
            Value::NativeFunction(c::OBJECT_IS_EXTENSIBLE),
        );
        object_props.insert(
            "isSealed".into(),
            Value::NativeFunction(c::OBJECT_IS_SEALED),
        );
        object_props.insert(
            "isFrozen".into(),
            Value::NativeFunction(c::OBJECT_IS_FROZEN),
        );
        object_props.insert("seal".into(), Value::NativeFunction(c::OBJECT_SEAL));
        object_props.insert(
            "getPrototypeOf".into(),
            Value::NativeFunction(c::REFLECT_GET_PROTOTYPE_OF),
        );
        object_props.insert(
            "setPrototypeOf".into(),
            Value::NativeFunction(c::REFLECT_SET_PROTOTYPE_OF),
        );

        // Object.prototype with hasOwnProperty
        let mut object_proto_props = HashMap::new();
        object_proto_props.insert(
            "hasOwnProperty".into(),
            Value::NativeFunction(c::OBJECT_HAS_OWN_PROPERTY),
        );
        let object_proto_idx = self.gc.allocate(
            &mut self.heap,
            HeapValue::Object(JsObject {
                properties: object_proto_props,
                prototype: None,
                extensible: true,
            }),
        );
        object_props.insert("prototype".into(), Value::Object(object_proto_idx));

        let object_obj_idx = self.gc.allocate(
            &mut self.heap,
            HeapValue::Object(JsObject {
                properties: object_props,
                prototype: None,
                extensible: true,
            }),
        );
        self.globals
            .insert("Object".into(), Value::Object(object_obj_idx));

        // Proxy
        self.globals
            .insert("Proxy".into(), Value::NativeFunction(c::PROXY_CONSTRUCTOR));

        // Reflect
        let mut reflect_props = HashMap::new();
        reflect_props.insert("get".into(), Value::NativeFunction(c::REFLECT_GET));
        reflect_props.insert("set".into(), Value::NativeFunction(c::REFLECT_SET));
        reflect_props.insert("has".into(), Value::NativeFunction(c::REFLECT_HAS));
        reflect_props.insert(
            "deleteProperty".into(),
            Value::NativeFunction(c::REFLECT_DELETE_PROPERTY),
        );
        reflect_props.insert("apply".into(), Value::NativeFunction(c::REFLECT_APPLY));
        reflect_props.insert(
            "construct".into(),
            Value::NativeFunction(c::REFLECT_CONSTRUCT),
        );
        reflect_props.insert("ownKeys".into(), Value::NativeFunction(c::REFLECT_OWN_KEYS));
        reflect_props.insert(
            "getOwnPropertyDescriptor".into(),
            Value::NativeFunction(c::REFLECT_GET_OWN_PROPERTY_DESCRIPTOR),
        );
        reflect_props.insert(
            "defineProperty".into(),
            Value::NativeFunction(c::REFLECT_DEFINE_PROPERTY),
        );
        reflect_props.insert(
            "getPrototypeOf".into(),
            Value::NativeFunction(c::REFLECT_GET_PROTOTYPE_OF),
        );
        reflect_props.insert(
            "setPrototypeOf".into(),
            Value::NativeFunction(c::REFLECT_SET_PROTOTYPE_OF),
        );
        reflect_props.insert(
            "isExtensible".into(),
            Value::NativeFunction(c::REFLECT_IS_EXTENSIBLE),
        );
        reflect_props.insert(
            "preventExtensions".into(),
            Value::NativeFunction(c::REFLECT_PREVENT_EXTENSIONS),
        );
        let reflect_obj_idx = self.gc.allocate(
            &mut self.heap,
            HeapValue::Object(JsObject {
                properties: reflect_props,
                prototype: None,
                extensible: true,
            }),
        );
        self.globals
            .insert("Reflect".into(), Value::Object(reflect_obj_idx));

        // Symbol - registered as NativeFunction(c::SYMBOL_CONSTRUCTOR) with well-known symbols accessible via GetProperty
        self.globals.insert(
            "Symbol".into(),
            Value::NativeFunction(c::SYMBOL_CONSTRUCTOR),
        );

        // JSON
        let mut json_props = HashMap::new();
        json_props.insert("parse".into(), Value::NativeFunction(c::JSON_PARSE));
        json_props.insert("stringify".into(), Value::NativeFunction(c::JSON_STRINGIFY));
        let json_obj_idx = self.gc.allocate(
            &mut self.heap,
            HeapValue::Object(JsObject {
                properties: json_props,
                prototype: None,
                extensible: true,
            }),
        );
        self.globals
            .insert("JSON".into(), Value::Object(json_obj_idx));

        // Array
        let mut array_props = HashMap::new();
        array_props.insert("isArray".into(), Value::NativeFunction(c::ARRAY_IS_ARRAY));
        array_props.insert("from".into(), Value::NativeFunction(c::ARRAY_FROM));
        array_props.insert("of".into(), Value::NativeFunction(c::ARRAY_OF));
        let array_obj_idx = self.gc.allocate(
            &mut self.heap,
            HeapValue::Object(JsObject {
                properties: array_props,
                prototype: None,
                extensible: true,
            }),
        );
        self.globals
            .insert("Array".into(), Value::Object(array_obj_idx));

        // BigInt
        self.globals.insert(
            "BigInt".into(),
            Value::NativeFunction(c::BIGINT_CONSTRUCTOR),
        );

        // Encoding
        self.globals
            .insert("atob".into(), Value::NativeFunction(c::ATOB));
        self.globals
            .insert("btoa".into(), Value::NativeFunction(c::BTOA));

        // URL object with static methods
        let mut url_props = HashMap::new();
        url_props.insert("canParse".into(), Value::NativeFunction(c::URL_CAN_PARSE));
        url_props.insert("parse".into(), Value::NativeFunction(c::URL_PARSE));
        let _url_obj_idx = self.gc.allocate(
            &mut self.heap,
            HeapValue::Object(JsObject {
                properties: url_props,
                prototype: None,
                extensible: true,
            }),
        );
        // URL is a factory function (NativeFunction(c::URL_CONSTRUCTOR)) used as `new URL(...)`
        // Static methods are accessed via the native function's own properties
        self.globals
            .insert("URL".into(), Value::NativeFunction(c::URL_CONSTRUCTOR));

        // URLSearchParams constructor
        self.globals.insert(
            "URLSearchParams".into(),
            Value::NativeFunction(c::URL_SEARCH_PARAMS_CONSTRUCTOR),
        );

        // Headers constructor
        self.globals.insert(
            "Headers".into(),
            Value::NativeFunction(c::HEADERS_CONSTRUCTOR),
        );

        // Request constructor
        self.globals.insert(
            "Request".into(),
            Value::NativeFunction(c::REQUEST_CONSTRUCTOR),
        );

        // Response constructor
        self.globals.insert(
            "Response".into(),
            Value::NativeFunction(c::RESPONSE_CONSTRUCTOR),
        );

        // fetch
        self.globals
            .insert("fetch".into(), Value::NativeFunction(c::FETCH));

        // Date
        let mut date_proto_props = HashMap::new();
        date_proto_props.insert("getTime".into(), Value::NativeFunction(c::DATE_GET_TIME));
        date_proto_props.insert(
            "getFullYear".into(),
            Value::NativeFunction(c::DATE_GET_FULL_YEAR),
        );
        date_proto_props.insert("getMonth".into(), Value::NativeFunction(c::DATE_GET_MONTH));
        date_proto_props.insert("getDate".into(), Value::NativeFunction(c::DATE_GET_DATE));
        date_proto_props.insert("getDay".into(), Value::NativeFunction(c::DATE_GET_DAY));
        date_proto_props.insert("getHours".into(), Value::NativeFunction(c::DATE_GET_HOURS));
        date_proto_props.insert(
            "getMinutes".into(),
            Value::NativeFunction(c::DATE_GET_MINUTES),
        );
        date_proto_props.insert(
            "getSeconds".into(),
            Value::NativeFunction(c::DATE_GET_SECONDS),
        );
        date_proto_props.insert(
            "getMilliseconds".into(),
            Value::NativeFunction(c::DATE_GET_MILLISECONDS),
        );
        date_proto_props.insert(
            "getTimezoneOffset".into(),
            Value::NativeFunction(c::DATE_GET_TIMEZONE_OFFSET),
        );
        date_proto_props.insert(
            "getUTCFullYear".into(),
            Value::NativeFunction(c::DATE_GET_UTC_FULL_YEAR),
        );
        date_proto_props.insert(
            "getUTCMonth".into(),
            Value::NativeFunction(c::DATE_GET_UTC_MONTH),
        );
        date_proto_props.insert(
            "getUTCDate".into(),
            Value::NativeFunction(c::DATE_GET_UTC_DATE),
        );
        date_proto_props.insert(
            "getUTCDay".into(),
            Value::NativeFunction(c::DATE_GET_UTC_DAY),
        );
        date_proto_props.insert(
            "getUTCHours".into(),
            Value::NativeFunction(c::DATE_GET_UTC_HOURS),
        );
        date_proto_props.insert(
            "getUTCMinutes".into(),
            Value::NativeFunction(c::DATE_GET_UTC_MINUTES),
        );
        date_proto_props.insert(
            "getUTCSeconds".into(),
            Value::NativeFunction(c::DATE_GET_UTC_SECONDS),
        );
        date_proto_props.insert(
            "getUTCMilliseconds".into(),
            Value::NativeFunction(c::DATE_GET_UTC_MILLISECONDS),
        );
        date_proto_props.insert("setTime".into(), Value::NativeFunction(c::DATE_SET_TIME));
        date_proto_props.insert(
            "setFullYear".into(),
            Value::NativeFunction(c::DATE_SET_FULL_YEAR),
        );
        date_proto_props.insert("setMonth".into(), Value::NativeFunction(c::DATE_SET_MONTH));
        date_proto_props.insert("setDate".into(), Value::NativeFunction(c::DATE_SET_DATE));
        date_proto_props.insert("setHours".into(), Value::NativeFunction(c::DATE_SET_HOURS));
        date_proto_props.insert(
            "setMinutes".into(),
            Value::NativeFunction(c::DATE_SET_MINUTES),
        );
        date_proto_props.insert(
            "setSeconds".into(),
            Value::NativeFunction(c::DATE_SET_SECONDS),
        );
        date_proto_props.insert(
            "setMilliseconds".into(),
            Value::NativeFunction(c::DATE_SET_MILLISECONDS),
        );
        date_proto_props.insert(
            "setUTCFullYear".into(),
            Value::NativeFunction(c::DATE_SET_UTC_FULL_YEAR),
        );
        date_proto_props.insert(
            "setUTCMonth".into(),
            Value::NativeFunction(c::DATE_SET_UTC_MONTH),
        );
        date_proto_props.insert(
            "setUTCDate".into(),
            Value::NativeFunction(c::DATE_SET_UTC_DATE),
        );
        date_proto_props.insert(
            "setUTCHours".into(),
            Value::NativeFunction(c::DATE_SET_UTC_HOURS),
        );
        date_proto_props.insert(
            "setUTCMinutes".into(),
            Value::NativeFunction(c::DATE_SET_UTC_MINUTES),
        );
        date_proto_props.insert(
            "setUTCSeconds".into(),
            Value::NativeFunction(c::DATE_SET_UTC_SECONDS),
        );
        date_proto_props.insert(
            "setUTCMilliseconds".into(),
            Value::NativeFunction(c::DATE_SET_UTC_MILLISECONDS),
        );
        date_proto_props.insert("toString".into(), Value::NativeFunction(c::DATE_TO_STRING));
        date_proto_props.insert(
            "toISOString".into(),
            Value::NativeFunction(c::DATE_TO_ISO_STRING),
        );
        date_proto_props.insert(
            "toUTCString".into(),
            Value::NativeFunction(c::DATE_TO_UTC_STRING),
        );
        date_proto_props.insert(
            "toDateString".into(),
            Value::NativeFunction(c::DATE_TO_DATE_STRING),
        );
        date_proto_props.insert(
            "toTimeString".into(),
            Value::NativeFunction(c::DATE_TO_TIME_STRING),
        );
        date_proto_props.insert("toJSON".into(), Value::NativeFunction(c::DATE_TO_JSON));
        date_proto_props.insert("valueOf".into(), Value::NativeFunction(c::DATE_VALUE_OF));
        let date_proto_idx = self.gc.allocate(
            &mut self.heap,
            HeapValue::Object(JsObject {
                properties: date_proto_props,
                prototype: None,
                extensible: true,
            }),
        );
        // Register Date as a NativeFunction for constructor
        self.globals
            .insert("Date".into(), Value::NativeFunction(c::DATE_CONSTRUCTOR));
        // Store the prototype index for Date constructor
        self.date_proto_idx = Some(date_proto_idx);

        // RegExp
        let mut regexp_proto_props = HashMap::new();
        regexp_proto_props.insert("test".into(), Value::NativeFunction(c::REGEXP_TEST));
        regexp_proto_props.insert("exec".into(), Value::NativeFunction(c::REGEXP_EXEC));
        regexp_proto_props.insert(
            "toString".into(),
            Value::NativeFunction(c::REGEXP_TO_STRING),
        );
        regexp_proto_props.insert("source".into(), Value::NativeFunction(c::REGEXP_SOURCE));
        regexp_proto_props.insert("flags".into(), Value::NativeFunction(c::REGEXP_FLAGS));
        regexp_proto_props.insert("global".into(), Value::NativeFunction(c::REGEXP_GLOBAL));
        regexp_proto_props.insert(
            "ignoreCase".into(),
            Value::NativeFunction(c::REGEXP_IGNORE_CASE),
        );
        regexp_proto_props.insert(
            "multiline".into(),
            Value::NativeFunction(c::REGEXP_MULTILINE),
        );
        regexp_proto_props.insert("dotAll".into(), Value::NativeFunction(c::REGEXP_DOT_ALL));
        regexp_proto_props.insert("unicode".into(), Value::NativeFunction(c::REGEXP_UNICODE));
        regexp_proto_props.insert("sticky".into(), Value::NativeFunction(c::REGEXP_STICKY));
        regexp_proto_props.insert(
            "lastIndex".into(),
            Value::NativeFunction(c::REGEXP_LAST_INDEX),
        );
        let regexp_proto_idx = self.gc.allocate(
            &mut self.heap,
            HeapValue::Object(JsObject {
                properties: regexp_proto_props,
                prototype: None,
                extensible: true,
            }),
        );
        // Register RegExp as a NativeFunction for constructor
        self.globals.insert(
            "RegExp".into(),
            Value::NativeFunction(c::REGEXP_CONSTRUCTOR),
        );
        // Store the prototype index for RegExp constructor
        self.regexp_proto_idx = Some(regexp_proto_idx);

        // Math
        let mut math_props = HashMap::new();
        math_props.insert("PI".into(), Value::Float(std::f64::consts::PI));
        math_props.insert("E".into(), Value::Float(std::f64::consts::E));
        math_props.insert("abs".into(), Value::NativeFunction(c::MATH_ABS));
        math_props.insert("floor".into(), Value::NativeFunction(c::MATH_FLOOR));
        math_props.insert("ceil".into(), Value::NativeFunction(c::MATH_CEIL));
        math_props.insert("round".into(), Value::NativeFunction(c::MATH_ROUND));
        math_props.insert("min".into(), Value::NativeFunction(c::MATH_MIN));
        math_props.insert("max".into(), Value::NativeFunction(c::MATH_MAX));
        math_props.insert("random".into(), Value::NativeFunction(c::MATH_RANDOM));
        math_props.insert("pow".into(), Value::NativeFunction(c::MATH_POW));
        math_props.insert("sqrt".into(), Value::NativeFunction(c::MATH_SQRT));
        math_props.insert("log".into(), Value::NativeFunction(c::MATH_LOG));
        math_props.insert("sin".into(), Value::NativeFunction(c::MATH_SIN));
        math_props.insert("cos".into(), Value::NativeFunction(c::MATH_COS));
        math_props.insert("tan".into(), Value::NativeFunction(c::MATH_TAN));
        let math_obj_idx = self.gc.allocate(
            &mut self.heap,
            HeapValue::Object(JsObject {
                properties: math_props,
                prototype: None,
                extensible: true,
            }),
        );
        self.globals
            .insert("Math".into(), Value::Object(math_obj_idx));

        // Number constructor
        let mut number_props = HashMap::new();
        number_props.insert("isFinite".into(), Value::NativeFunction(c::IS_FINITE));
        number_props.insert("isNaN".into(), Value::NativeFunction(c::IS_NAN));
        number_props.insert("parseFloat".into(), Value::NativeFunction(c::PARSE_FLOAT));
        number_props.insert("parseInt".into(), Value::NativeFunction(c::PARSE_INT));
        number_props.insert(
            "isInteger".into(),
            Value::NativeFunction(c::NUMBER_IS_INTEGER),
        );
        number_props.insert(
            "isSafeInteger".into(),
            Value::NativeFunction(c::NUMBER_IS_SAFE_INTEGER),
        );
        let number_obj_idx = self.gc.allocate(
            &mut self.heap,
            HeapValue::Object(JsObject {
                properties: number_props,
                prototype: None,
                extensible: true,
            }),
        );
        self.globals
            .insert("Number".into(), Value::Object(number_obj_idx));

        // Promise constructor and prototype
        let mut promise_proto_props = HashMap::new();
        promise_proto_props.insert("then".into(), Value::NativeFunction(c::PROMISE_THEN));
        promise_proto_props.insert("catch".into(), Value::NativeFunction(c::PROMISE_CATCH));
        promise_proto_props.insert("finally".into(), Value::NativeFunction(c::PROMISE_FINALLY));
        let promise_proto_idx = self.gc.allocate(
            &mut self.heap,
            HeapValue::Object(JsObject {
                properties: promise_proto_props,
                prototype: None,
                extensible: true,
            }),
        );

        let mut promise_ctor_props = HashMap::new();
        promise_ctor_props.insert("prototype".into(), Value::Object(promise_proto_idx));
        promise_ctor_props.insert("resolve".into(), Value::NativeFunction(c::PROMISE_RESOLVE));
        promise_ctor_props.insert("reject".into(), Value::NativeFunction(c::PROMISE_REJECT));
        promise_ctor_props.insert("all".into(), Value::NativeFunction(c::PROMISE_ALL));
        promise_ctor_props.insert("race".into(), Value::NativeFunction(c::PROMISE_RACE));
        self.gc.allocate(
            &mut self.heap,
            HeapValue::Object(JsObject {
                properties: promise_ctor_props,
                prototype: None,
                extensible: true,
            }),
        );
        self.globals.insert(
            "Promise".into(),
            Value::NativeFunction(c::PROMISE_CONSTRUCTOR),
        );

        // Error constructor
        let error_proto_idx = self
            .gc
            .allocate(&mut self.heap, HeapValue::Object(JsObject::new()));
        let mut error_ctor_props = HashMap::new();
        error_ctor_props.insert("prototype".into(), Value::Object(error_proto_idx));
        self.gc.allocate(
            &mut self.heap,
            HeapValue::Object(JsObject {
                properties: error_ctor_props,
                prototype: None,
                extensible: true,
            }),
        );
        self.globals
            .insert("Error".into(), Value::NativeFunction(c::ERROR_CONSTRUCTOR));

        // TypeError constructor
        let mut type_error_proto_props = HashMap::new();
        type_error_proto_props.insert("name".into(), Value::String("TypeError".into()));
        let type_error_proto_idx = self.gc.allocate(
            &mut self.heap,
            HeapValue::Object(JsObject {
                properties: type_error_proto_props,
                prototype: Some(error_proto_idx),
                extensible: true,
            }),
        );
        let mut type_error_ctor_props = HashMap::new();
        type_error_ctor_props.insert("prototype".into(), Value::Object(type_error_proto_idx));
        self.gc.allocate(
            &mut self.heap,
            HeapValue::Object(JsObject {
                properties: type_error_ctor_props,
                prototype: None,
                extensible: true,
            }),
        );
        self.globals.insert(
            "TypeError".into(),
            Value::NativeFunction(c::TYPE_ERROR_CONSTRUCTOR),
        );

        // ReferenceError constructor
        let mut ref_error_proto_props = HashMap::new();
        ref_error_proto_props.insert("name".into(), Value::String("ReferenceError".into()));
        let ref_error_proto_idx = self.gc.allocate(
            &mut self.heap,
            HeapValue::Object(JsObject {
                properties: ref_error_proto_props,
                prototype: Some(error_proto_idx),
                extensible: true,
            }),
        );
        let mut ref_error_ctor_props = HashMap::new();
        ref_error_ctor_props.insert("prototype".into(), Value::Object(ref_error_proto_idx));
        self.gc.allocate(
            &mut self.heap,
            HeapValue::Object(JsObject {
                properties: ref_error_ctor_props,
                prototype: None,
                extensible: true,
            }),
        );
        self.globals.insert(
            "ReferenceError".into(),
            Value::NativeFunction(c::REFERENCE_ERROR_CONSTRUCTOR),
        );

        // SyntaxError constructor
        let mut syntax_error_proto_props = HashMap::new();
        syntax_error_proto_props.insert("name".into(), Value::String("SyntaxError".into()));
        let syntax_error_proto_idx = self.gc.allocate(
            &mut self.heap,
            HeapValue::Object(JsObject {
                properties: syntax_error_proto_props,
                prototype: Some(error_proto_idx),
                extensible: true,
            }),
        );
        let mut syntax_error_ctor_props = HashMap::new();
        syntax_error_ctor_props.insert("prototype".into(), Value::Object(syntax_error_proto_idx));
        self.gc.allocate(
            &mut self.heap,
            HeapValue::Object(JsObject {
                properties: syntax_error_ctor_props,
                prototype: None,
                extensible: true,
            }),
        );
        self.globals.insert(
            "SyntaxError".into(),
            Value::NativeFunction(c::SYNTAX_ERROR_CONSTRUCTOR),
        );

        // RangeError constructor
        let mut range_error_proto_props = HashMap::new();
        range_error_proto_props.insert("name".into(), Value::String("RangeError".into()));
        let range_error_proto_idx = self.gc.allocate(
            &mut self.heap,
            HeapValue::Object(JsObject {
                properties: range_error_proto_props,
                prototype: Some(error_proto_idx),
                extensible: true,
            }),
        );
        let mut range_error_ctor_props = HashMap::new();
        range_error_ctor_props.insert("prototype".into(), Value::Object(range_error_proto_idx));
        self.gc.allocate(
            &mut self.heap,
            HeapValue::Object(JsObject {
                properties: range_error_ctor_props,
                prototype: None,
                extensible: true,
            }),
        );
        self.globals.insert(
            "RangeError".into(),
            Value::NativeFunction(c::RANGE_ERROR_CONSTRUCTOR),
        );

        // TypedArray constructors
        let typed_array_constructors = [
            ("Int8Array", 301usize),
            ("Uint8Array", 302),
            ("Uint8ClampedArray", 303),
            ("Int16Array", 304),
            ("Uint16Array", 305),
            ("Int32Array", 306),
            ("Uint32Array", 307),
            ("Float32Array", 308),
            ("Float64Array", 309),
            ("BigInt64Array", 310),
            ("BigUint64Array", 311),
        ];

        for (name, ctor_idx) in typed_array_constructors.iter() {
            // Create prototype
            let mut proto_props = HashMap::new();
            proto_props.insert(
                "BYTES_PER_ELEMENT".into(),
                Value::Integer(TypedArray::element_size(&parse_typed_array_type(name)) as i64),
            );
            proto_props.insert(
                "length".into(),
                Value::NativeFunction(c::TYPED_ARRAY_LENGTH),
            );
            proto_props.insert("get".into(), Value::NativeFunction(c::TYPED_ARRAY_GET));
            proto_props.insert("set".into(), Value::NativeFunction(c::TYPED_ARRAY_SET));
            proto_props.insert(
                "subarray".into(),
                Value::NativeFunction(c::TYPED_ARRAY_SUBARRAY),
            );
            proto_props.insert("slice".into(), Value::NativeFunction(c::TYPED_ARRAY_SLICE));
            let proto_idx = self.gc.allocate(
                &mut self.heap,
                HeapValue::Object(JsObject {
                    properties: proto_props,
                    prototype: None,
                    extensible: true,
                }),
            );

            // Create constructor
            let mut ctor_props = HashMap::new();
            ctor_props.insert("prototype".into(), Value::Object(proto_idx));
            ctor_props.insert(
                "BYTES_PER_ELEMENT".into(),
                Value::Integer(TypedArray::element_size(&parse_typed_array_type(name)) as i64),
            );
            ctor_props.insert("from".into(), Value::NativeFunction(c::TYPED_ARRAY_FROM));
            ctor_props.insert("of".into(), Value::NativeFunction(c::TYPED_ARRAY_OF));
            let _ctor_obj_idx = self.gc.allocate(
                &mut self.heap,
                HeapValue::Object(JsObject {
                    properties: ctor_props,
                    prototype: None,
                    extensible: true,
                }),
            );
            self.globals
                .insert((*name).into(), Value::NativeFunction(*ctor_idx));
        }

        // Map
        let mut map_proto_props = HashMap::new();
        map_proto_props.insert("get".into(), Value::NativeFunction(c::MAP_GET));
        map_proto_props.insert("set".into(), Value::NativeFunction(c::MAP_SET));
        map_proto_props.insert("has".into(), Value::NativeFunction(c::MAP_HAS));
        map_proto_props.insert("delete".into(), Value::NativeFunction(c::MAP_DELETE));
        map_proto_props.insert("clear".into(), Value::NativeFunction(c::MAP_CLEAR));
        map_proto_props.insert("size".into(), Value::NativeFunction(c::MAP_SIZE));
        map_proto_props.insert("forEach".into(), Value::NativeFunction(c::MAP_FOR_EACH));
        map_proto_props.insert("keys".into(), Value::NativeFunction(c::MAP_KEYS));
        map_proto_props.insert("values".into(), Value::NativeFunction(c::MAP_VALUES));
        map_proto_props.insert("entries".into(), Value::NativeFunction(c::MAP_ENTRIES));
        let map_proto_idx = self.gc.allocate(
            &mut self.heap,
            HeapValue::Object(JsObject {
                properties: map_proto_props,
                prototype: None,
                extensible: true,
            }),
        );

        let mut map_ctor_props = HashMap::new();
        map_ctor_props.insert("prototype".into(), Value::Object(map_proto_idx));
        let _map_ctor_idx = self.gc.allocate(
            &mut self.heap,
            HeapValue::Object(JsObject {
                properties: map_ctor_props,
                prototype: None,
                extensible: true,
            }),
        );
        self.globals
            .insert("Map".into(), Value::NativeFunction(c::MAP_CONSTRUCTOR));

        // Set
        let mut set_proto_props = HashMap::new();
        set_proto_props.insert("add".into(), Value::NativeFunction(c::SET_ADD));
        set_proto_props.insert("has".into(), Value::NativeFunction(c::SET_HAS));
        set_proto_props.insert("delete".into(), Value::NativeFunction(c::SET_DELETE));
        set_proto_props.insert("clear".into(), Value::NativeFunction(c::SET_CLEAR));
        set_proto_props.insert("size".into(), Value::NativeFunction(c::SET_SIZE));
        set_proto_props.insert("forEach".into(), Value::NativeFunction(c::SET_FOR_EACH));
        set_proto_props.insert("values".into(), Value::NativeFunction(c::SET_VALUES));
        set_proto_props.insert("keys".into(), Value::NativeFunction(c::SET_KEYS));
        set_proto_props.insert("entries".into(), Value::NativeFunction(c::SET_ENTRIES));
        let set_proto_idx = self.gc.allocate(
            &mut self.heap,
            HeapValue::Object(JsObject {
                properties: set_proto_props,
                prototype: None,
                extensible: true,
            }),
        );

        let mut set_ctor_props = HashMap::new();
        set_ctor_props.insert("prototype".into(), Value::Object(set_proto_idx));
        let _set_ctor_idx = self.gc.allocate(
            &mut self.heap,
            HeapValue::Object(JsObject {
                properties: set_ctor_props,
                prototype: None,
                extensible: true,
            }),
        );
        self.globals
            .insert("Set".into(), Value::NativeFunction(c::SET_CONSTRUCTOR));

        // WeakMap
        let mut weakmap_proto_props = HashMap::new();
        weakmap_proto_props.insert("get".into(), Value::NativeFunction(c::WEAKMAP_GET));
        weakmap_proto_props.insert("set".into(), Value::NativeFunction(c::WEAKMAP_SET));
        weakmap_proto_props.insert("has".into(), Value::NativeFunction(c::WEAKMAP_HAS));
        weakmap_proto_props.insert("delete".into(), Value::NativeFunction(c::WEAKMAP_DELETE));
        let weakmap_proto_idx = self.gc.allocate(
            &mut self.heap,
            HeapValue::Object(JsObject {
                properties: weakmap_proto_props,
                prototype: None,
                extensible: true,
            }),
        );

        let mut weakmap_ctor_props = HashMap::new();
        weakmap_ctor_props.insert("prototype".into(), Value::Object(weakmap_proto_idx));
        let _weakmap_ctor_idx = self.gc.allocate(
            &mut self.heap,
            HeapValue::Object(JsObject {
                properties: weakmap_ctor_props,
                prototype: None,
                extensible: true,
            }),
        );
        self.globals.insert(
            "WeakMap".into(),
            Value::NativeFunction(c::WEAKMAP_CONSTRUCTOR),
        );

        // WeakSet
        let mut weakset_proto_props = HashMap::new();
        weakset_proto_props.insert("add".into(), Value::NativeFunction(c::WEAKSET_ADD));
        weakset_proto_props.insert("has".into(), Value::NativeFunction(c::WEAKSET_HAS));
        weakset_proto_props.insert("delete".into(), Value::NativeFunction(c::WEAKSET_DELETE));
        let weakset_proto_idx = self.gc.allocate(
            &mut self.heap,
            HeapValue::Object(JsObject {
                properties: weakset_proto_props,
                prototype: None,
                extensible: true,
            }),
        );

        let mut weakset_ctor_props = HashMap::new();
        weakset_ctor_props.insert("prototype".into(), Value::Object(weakset_proto_idx));
        let _weakset_ctor_idx = self.gc.allocate(
            &mut self.heap,
            HeapValue::Object(JsObject {
                properties: weakset_ctor_props,
                prototype: None,
                extensible: true,
            }),
        );
        self.globals.insert(
            "WeakSet".into(),
            Value::NativeFunction(c::WEAKSET_CONSTRUCTOR),
        );

        // Generator
        let mut generator_proto_props = HashMap::new();
        generator_proto_props.insert("next".into(), Value::NativeFunction(c::GENERATOR_NEXT));
        generator_proto_props.insert("return".into(), Value::NativeFunction(c::GENERATOR_RETURN));
        generator_proto_props.insert("throw".into(), Value::NativeFunction(c::GENERATOR_THROW));
        generator_proto_props.insert(
            "Symbol.iterator".into(),
            Value::NativeFunction(c::GENERATOR_SYMBOL_ITERATOR),
        );
        let generator_proto_idx = self.gc.allocate(
            &mut self.heap,
            HeapValue::Object(JsObject {
                properties: generator_proto_props,
                prototype: None,
                extensible: true,
            }),
        );
        self.generator_proto_idx = Some(generator_proto_idx);

        let mut generator_ctor_props = HashMap::new();
        generator_ctor_props.insert("prototype".into(), Value::Object(generator_proto_idx));
        let generator_ctor_idx = self.gc.allocate(
            &mut self.heap,
            HeapValue::Object(JsObject {
                properties: generator_ctor_props,
                prototype: None,
                extensible: true,
            }),
        );
        self.globals
            .insert("Generator".into(), Value::Object(generator_ctor_idx));

        // WebSocket constructor
        self.globals.insert(
            "WebSocket".into(),
            Value::NativeFunction(c::WEBSOCKET_CONSTRUCTOR),
        );
    }
}

fn parse_typed_array_type(name: &str) -> TypedArrayType {
    match name {
        "Int8Array" => TypedArrayType::Int8Array,
        "Uint8Array" => TypedArrayType::Uint8Array,
        "Uint8ClampedArray" => TypedArrayType::Uint8ClampedArray,
        "Int16Array" => TypedArrayType::Int16Array,
        "Uint16Array" => TypedArrayType::Uint16Array,
        "Int32Array" => TypedArrayType::Int32Array,
        "Uint32Array" => TypedArrayType::Uint32Array,
        "Float32Array" => TypedArrayType::Float32Array,
        "Float64Array" => TypedArrayType::Float64Array,
        "BigInt64Array" => TypedArrayType::BigInt64Array,
        "BigUint64Array" => TypedArrayType::BigUint64Array,
        _ => TypedArrayType::Int8Array,
    }
}
