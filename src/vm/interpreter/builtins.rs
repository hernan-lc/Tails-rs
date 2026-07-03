use super::{HeapValue, Interpreter, JsObject};
use crate::objects::js_array::{TypedArray, TypedArrayType};
use crate::objects::Value;
use crate::props;
use crate::runtime_env::native_fns::constants as c;

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
        self.globals.insert(
            "setImmediate".into(),
            Value::NativeFunction(c::SET_IMMEDIATE),
        );
        self.globals.insert(
            "clearImmediate".into(),
            Value::NativeFunction(c::CLEAR_IMMEDIATE),
        );

        // CommonJS require() — NativeFunction(c::REQUIRE)
        self.globals
            .insert("require".into(), Value::NativeFunction(c::REQUIRE));

        // console object
        let console_props = props! {
            "log" => Value::NativeFunction(c::CONSOLE_LOG),
            "warn" => Value::NativeFunction(c::CONSOLE_WARN),
            "error" => Value::NativeFunction(c::CONSOLE_ERROR),
            "info" => Value::NativeFunction(c::CONSOLE_INFO),
            "table" => Value::NativeFunction(c::CONSOLE_TABLE),
            "dir" => Value::NativeFunction(c::CONSOLE_DIR),
            "group" => Value::NativeFunction(c::CONSOLE_GROUP),
            "groupEnd" => Value::NativeFunction(c::CONSOLE_GROUP_END),
            "groupCollapsed" => Value::NativeFunction(c::CONSOLE_GROUP_COLLAPSED),
            "time" => Value::NativeFunction(c::CONSOLE_TIME),
            "timeEnd" => Value::NativeFunction(c::CONSOLE_TIME_END),
            "assert" => Value::NativeFunction(c::CONSOLE_ASSERT),
            "clear" => Value::NativeFunction(c::CONSOLE_CLEAR),
            "trace" => Value::NativeFunction(c::CONSOLE_INFO),
            "count" => Value::NativeFunction(c::CONSOLE_INFO),
            "countReset" => Value::NativeFunction(c::CONSOLE_INFO),
            "debug" => Value::NativeFunction(c::CONSOLE_LOG),
            "profile" => Value::NativeFunction(c::CONSOLE_INFO),
            "profileEnd" => Value::NativeFunction(c::CONSOLE_INFO),
            "timeLog" => Value::NativeFunction(c::CONSOLE_TIME_END),
        };
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
        let object_proto_props = props! {
            "hasOwnProperty" => Value::NativeFunction(c::OBJECT_HAS_OWN_PROPERTY),
        };
        let object_proto_idx = self.gc.allocate(
            &mut self.heap,
            HeapValue::Object(JsObject {
                properties: object_proto_props,
                prototype: None,
                extensible: true,
            }),
        );

        let object_props = props! {
            "keys" => Value::NativeFunction(c::OBJECT_KEYS),
            "values" => Value::NativeFunction(c::OBJECT_VALUES),
            "entries" => Value::NativeFunction(c::OBJECT_ENTRIES),
            "assign" => Value::NativeFunction(c::OBJECT_ASSIGN),
            "defineProperty" => Value::NativeFunction(c::OBJECT_DEFINE_PROPERTY),
            "getOwnPropertyDescriptor" => Value::NativeFunction(c::OBJECT_GET_OWN_PROPERTY_DESCRIPTOR),
            "freeze" => Value::NativeFunction(c::OBJECT_FREEZE),
            "is" => Value::NativeFunction(c::OBJECT_IS),
            "preventExtensions" => Value::NativeFunction(c::OBJECT_PREVENT_EXTENSIONS),
            "isExtensible" => Value::NativeFunction(c::OBJECT_IS_EXTENSIBLE),
            "isSealed" => Value::NativeFunction(c::OBJECT_IS_SEALED),
            "isFrozen" => Value::NativeFunction(c::OBJECT_IS_FROZEN),
            "seal" => Value::NativeFunction(c::OBJECT_SEAL),
            "getPrototypeOf" => Value::NativeFunction(c::REFLECT_GET_PROTOTYPE_OF),
            "setPrototypeOf" => Value::NativeFunction(c::REFLECT_SET_PROTOTYPE_OF),
            "prototype" => Value::Object(object_proto_idx),
        };

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
        let reflect_props = props! {
            "get" => Value::NativeFunction(c::REFLECT_GET),
            "set" => Value::NativeFunction(c::REFLECT_SET),
            "has" => Value::NativeFunction(c::REFLECT_HAS),
            "deleteProperty" => Value::NativeFunction(c::REFLECT_DELETE_PROPERTY),
            "apply" => Value::NativeFunction(c::REFLECT_APPLY),
            "construct" => Value::NativeFunction(c::REFLECT_CONSTRUCT),
            "ownKeys" => Value::NativeFunction(c::REFLECT_OWN_KEYS),
            "getOwnPropertyDescriptor" => Value::NativeFunction(c::REFLECT_GET_OWN_PROPERTY_DESCRIPTOR),
            "defineProperty" => Value::NativeFunction(c::REFLECT_DEFINE_PROPERTY),
            "getPrototypeOf" => Value::NativeFunction(c::REFLECT_GET_PROTOTYPE_OF),
            "setPrototypeOf" => Value::NativeFunction(c::REFLECT_SET_PROTOTYPE_OF),
            "isExtensible" => Value::NativeFunction(c::REFLECT_IS_EXTENSIBLE),
            "preventExtensions" => Value::NativeFunction(c::REFLECT_PREVENT_EXTENSIONS),
        };
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
        let json_props = props! {
            "parse" => Value::NativeFunction(c::JSON_PARSE),
            "stringify" => Value::NativeFunction(c::JSON_STRINGIFY),
        };
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
        let array_props = props! {
            "isArray" => Value::NativeFunction(c::ARRAY_IS_ARRAY),
            "from" => Value::NativeFunction(c::ARRAY_FROM),
            "of" => Value::NativeFunction(c::ARRAY_OF),
        };
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
        let url_props = props! {
            "canParse" => Value::NativeFunction(c::URL_CAN_PARSE),
            "parse" => Value::NativeFunction(c::URL_PARSE),
        };
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
        let date_proto_props = props! {
            "getTime" => Value::NativeFunction(c::DATE_GET_TIME),
            "getFullYear" => Value::NativeFunction(c::DATE_GET_FULL_YEAR),
            "getMonth" => Value::NativeFunction(c::DATE_GET_MONTH),
            "getDate" => Value::NativeFunction(c::DATE_GET_DATE),
            "getDay" => Value::NativeFunction(c::DATE_GET_DAY),
            "getHours" => Value::NativeFunction(c::DATE_GET_HOURS),
            "getMinutes" => Value::NativeFunction(c::DATE_GET_MINUTES),
            "getSeconds" => Value::NativeFunction(c::DATE_GET_SECONDS),
            "getMilliseconds" => Value::NativeFunction(c::DATE_GET_MILLISECONDS),
            "getTimezoneOffset" => Value::NativeFunction(c::DATE_GET_TIMEZONE_OFFSET),
            "getUTCFullYear" => Value::NativeFunction(c::DATE_GET_UTC_FULL_YEAR),
            "getUTCMonth" => Value::NativeFunction(c::DATE_GET_UTC_MONTH),
            "getUTCDate" => Value::NativeFunction(c::DATE_GET_UTC_DATE),
            "getUTCDay" => Value::NativeFunction(c::DATE_GET_UTC_DAY),
            "getUTCHours" => Value::NativeFunction(c::DATE_GET_UTC_HOURS),
            "getUTCMinutes" => Value::NativeFunction(c::DATE_GET_UTC_MINUTES),
            "getUTCSeconds" => Value::NativeFunction(c::DATE_GET_UTC_SECONDS),
            "getUTCMilliseconds" => Value::NativeFunction(c::DATE_GET_UTC_MILLISECONDS),
            "setTime" => Value::NativeFunction(c::DATE_SET_TIME),
            "setFullYear" => Value::NativeFunction(c::DATE_SET_FULL_YEAR),
            "setMonth" => Value::NativeFunction(c::DATE_SET_MONTH),
            "setDate" => Value::NativeFunction(c::DATE_SET_DATE),
            "setHours" => Value::NativeFunction(c::DATE_SET_HOURS),
            "setMinutes" => Value::NativeFunction(c::DATE_SET_MINUTES),
            "setSeconds" => Value::NativeFunction(c::DATE_SET_SECONDS),
            "setMilliseconds" => Value::NativeFunction(c::DATE_SET_MILLISECONDS),
            "setUTCFullYear" => Value::NativeFunction(c::DATE_SET_UTC_FULL_YEAR),
            "setUTCMonth" => Value::NativeFunction(c::DATE_SET_UTC_MONTH),
            "setUTCDate" => Value::NativeFunction(c::DATE_SET_UTC_DATE),
            "setUTCHours" => Value::NativeFunction(c::DATE_SET_UTC_HOURS),
            "setUTCMinutes" => Value::NativeFunction(c::DATE_SET_UTC_MINUTES),
            "setUTCSeconds" => Value::NativeFunction(c::DATE_SET_UTC_SECONDS),
            "setUTCMilliseconds" => Value::NativeFunction(c::DATE_SET_UTC_MILLISECONDS),
            "toString" => Value::NativeFunction(c::DATE_TO_STRING),
            "toISOString" => Value::NativeFunction(c::DATE_TO_ISO_STRING),
            "toUTCString" => Value::NativeFunction(c::DATE_TO_UTC_STRING),
            "toDateString" => Value::NativeFunction(c::DATE_TO_DATE_STRING),
            "toTimeString" => Value::NativeFunction(c::DATE_TO_TIME_STRING),
            "toJSON" => Value::NativeFunction(c::DATE_TO_JSON),
            "valueOf" => Value::NativeFunction(c::DATE_VALUE_OF),
        };
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
        let regexp_proto_props = props! {
            "test" => Value::NativeFunction(c::REGEXP_TEST),
            "exec" => Value::NativeFunction(c::REGEXP_EXEC),
            "toString" => Value::NativeFunction(c::REGEXP_TO_STRING),
            "source" => Value::NativeFunction(c::REGEXP_SOURCE),
            "flags" => Value::NativeFunction(c::REGEXP_FLAGS),
            "global" => Value::NativeFunction(c::REGEXP_GLOBAL),
            "ignoreCase" => Value::NativeFunction(c::REGEXP_IGNORE_CASE),
            "multiline" => Value::NativeFunction(c::REGEXP_MULTILINE),
            "dotAll" => Value::NativeFunction(c::REGEXP_DOT_ALL),
            "unicode" => Value::NativeFunction(c::REGEXP_UNICODE),
            "sticky" => Value::NativeFunction(c::REGEXP_STICKY),
            "lastIndex" => Value::NativeFunction(c::REGEXP_LAST_INDEX),
        };
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
        let math_props = props! {
            "PI" => Value::Float(std::f64::consts::PI),
            "E" => Value::Float(std::f64::consts::E),
            "abs" => Value::NativeFunction(c::MATH_ABS),
            "floor" => Value::NativeFunction(c::MATH_FLOOR),
            "ceil" => Value::NativeFunction(c::MATH_CEIL),
            "round" => Value::NativeFunction(c::MATH_ROUND),
            "min" => Value::NativeFunction(c::MATH_MIN),
            "max" => Value::NativeFunction(c::MATH_MAX),
            "random" => Value::NativeFunction(c::MATH_RANDOM),
            "pow" => Value::NativeFunction(c::MATH_POW),
            "sqrt" => Value::NativeFunction(c::MATH_SQRT),
            "log" => Value::NativeFunction(c::MATH_LOG),
            "sin" => Value::NativeFunction(c::MATH_SIN),
            "cos" => Value::NativeFunction(c::MATH_COS),
            "tan" => Value::NativeFunction(c::MATH_TAN),
        };
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
        let number_props = props! {
            "isFinite" => Value::NativeFunction(c::IS_FINITE),
            "isNaN" => Value::NativeFunction(c::IS_NAN),
            "parseFloat" => Value::NativeFunction(c::PARSE_FLOAT),
            "parseInt" => Value::NativeFunction(c::PARSE_INT),
            "isInteger" => Value::NativeFunction(c::NUMBER_IS_INTEGER),
            "isSafeInteger" => Value::NativeFunction(c::NUMBER_IS_SAFE_INTEGER),
        };
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
        let promise_proto_props = props! {
            "then" => Value::NativeFunction(c::PROMISE_THEN),
            "catch" => Value::NativeFunction(c::PROMISE_CATCH),
            "finally" => Value::NativeFunction(c::PROMISE_FINALLY),
        };
        let promise_proto_idx = self.gc.allocate(
            &mut self.heap,
            HeapValue::Object(JsObject {
                properties: promise_proto_props,
                prototype: None,
                extensible: true,
            }),
        );

        let promise_ctor_props = props! {
            "prototype" => Value::Object(promise_proto_idx),
            "resolve" => Value::NativeFunction(c::PROMISE_RESOLVE),
            "reject" => Value::NativeFunction(c::PROMISE_REJECT),
            "all" => Value::NativeFunction(c::PROMISE_ALL),
            "race" => Value::NativeFunction(c::PROMISE_RACE),
        };
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
        let error_ctor_props = props! {
            "prototype" => Value::Object(error_proto_idx),
        };
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
        let type_error_proto_props = props! {
            "name" => Value::String("TypeError".into()),
        };
        let type_error_proto_idx = self.gc.allocate(
            &mut self.heap,
            HeapValue::Object(JsObject {
                properties: type_error_proto_props,
                prototype: Some(error_proto_idx),
                extensible: true,
            }),
        );
        let type_error_ctor_props = props! {
            "prototype" => Value::Object(type_error_proto_idx),
        };
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
        let ref_error_proto_props = props! {
            "name" => Value::String("ReferenceError".into()),
        };
        let ref_error_proto_idx = self.gc.allocate(
            &mut self.heap,
            HeapValue::Object(JsObject {
                properties: ref_error_proto_props,
                prototype: Some(error_proto_idx),
                extensible: true,
            }),
        );
        let ref_error_ctor_props = props! {
            "prototype" => Value::Object(ref_error_proto_idx),
        };
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
        let syntax_error_proto_props = props! {
            "name" => Value::String("SyntaxError".into()),
        };
        let syntax_error_proto_idx = self.gc.allocate(
            &mut self.heap,
            HeapValue::Object(JsObject {
                properties: syntax_error_proto_props,
                prototype: Some(error_proto_idx),
                extensible: true,
            }),
        );
        let syntax_error_ctor_props = props! {
            "prototype" => Value::Object(syntax_error_proto_idx),
        };
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
        let range_error_proto_props = props! {
            "name" => Value::String("RangeError".into()),
        };
        let range_error_proto_idx = self.gc.allocate(
            &mut self.heap,
            HeapValue::Object(JsObject {
                properties: range_error_proto_props,
                prototype: Some(error_proto_idx),
                extensible: true,
            }),
        );
        let range_error_ctor_props = props! {
            "prototype" => Value::Object(range_error_proto_idx),
        };
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
            let bytes_per_element = TypedArray::element_size(&parse_typed_array_type(name)) as i64;
            let proto_props = props! {
                "BYTES_PER_ELEMENT" => Value::Integer(bytes_per_element),
                "length" => Value::NativeFunction(c::TYPED_ARRAY_LENGTH),
                "get" => Value::NativeFunction(c::TYPED_ARRAY_GET),
                "set" => Value::NativeFunction(c::TYPED_ARRAY_SET),
                "subarray" => Value::NativeFunction(c::TYPED_ARRAY_SUBARRAY),
                "slice" => Value::NativeFunction(c::TYPED_ARRAY_SLICE),
            };
            let proto_idx = self.gc.allocate(
                &mut self.heap,
                HeapValue::Object(JsObject {
                    properties: proto_props,
                    prototype: None,
                    extensible: true,
                }),
            );

            // Create constructor
            let ctor_props = props! {
                "prototype" => Value::Object(proto_idx),
                "BYTES_PER_ELEMENT" => Value::Integer(bytes_per_element),
                "from" => Value::NativeFunction(c::TYPED_ARRAY_FROM),
                "of" => Value::NativeFunction(c::TYPED_ARRAY_OF),
            };
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
        let map_proto_props = props! {
            "get" => Value::NativeFunction(c::MAP_GET),
            "set" => Value::NativeFunction(c::MAP_SET),
            "has" => Value::NativeFunction(c::MAP_HAS),
            "delete" => Value::NativeFunction(c::MAP_DELETE),
            "clear" => Value::NativeFunction(c::MAP_CLEAR),
            "size" => Value::NativeFunction(c::MAP_SIZE),
            "forEach" => Value::NativeFunction(c::MAP_FOR_EACH),
            "keys" => Value::NativeFunction(c::MAP_KEYS),
            "values" => Value::NativeFunction(c::MAP_VALUES),
            "entries" => Value::NativeFunction(c::MAP_ENTRIES),
        };
        let map_proto_idx = self.gc.allocate(
            &mut self.heap,
            HeapValue::Object(JsObject {
                properties: map_proto_props,
                prototype: None,
                extensible: true,
            }),
        );

        let map_ctor_props = props! {
            "prototype" => Value::Object(map_proto_idx),
        };
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
        let set_proto_props = props! {
            "add" => Value::NativeFunction(c::SET_ADD),
            "has" => Value::NativeFunction(c::SET_HAS),
            "delete" => Value::NativeFunction(c::SET_DELETE),
            "clear" => Value::NativeFunction(c::SET_CLEAR),
            "size" => Value::NativeFunction(c::SET_SIZE),
            "forEach" => Value::NativeFunction(c::SET_FOR_EACH),
            "values" => Value::NativeFunction(c::SET_VALUES),
            "keys" => Value::NativeFunction(c::SET_KEYS),
            "entries" => Value::NativeFunction(c::SET_ENTRIES),
        };
        let set_proto_idx = self.gc.allocate(
            &mut self.heap,
            HeapValue::Object(JsObject {
                properties: set_proto_props,
                prototype: None,
                extensible: true,
            }),
        );

        let set_ctor_props = props! {
            "prototype" => Value::Object(set_proto_idx),
        };
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
        let weakmap_proto_props = props! {
            "get" => Value::NativeFunction(c::WEAKMAP_GET),
            "set" => Value::NativeFunction(c::WEAKMAP_SET),
            "has" => Value::NativeFunction(c::WEAKMAP_HAS),
            "delete" => Value::NativeFunction(c::WEAKMAP_DELETE),
        };
        let weakmap_proto_idx = self.gc.allocate(
            &mut self.heap,
            HeapValue::Object(JsObject {
                properties: weakmap_proto_props,
                prototype: None,
                extensible: true,
            }),
        );

        let weakmap_ctor_props = props! {
            "prototype" => Value::Object(weakmap_proto_idx),
        };
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
        let weakset_proto_props = props! {
            "add" => Value::NativeFunction(c::WEAKSET_ADD),
            "has" => Value::NativeFunction(c::WEAKSET_HAS),
            "delete" => Value::NativeFunction(c::WEAKSET_DELETE),
        };
        let weakset_proto_idx = self.gc.allocate(
            &mut self.heap,
            HeapValue::Object(JsObject {
                properties: weakset_proto_props,
                prototype: None,
                extensible: true,
            }),
        );

        let weakset_ctor_props = props! {
            "prototype" => Value::Object(weakset_proto_idx),
        };
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
        let generator_proto_props = props! {
            "next" => Value::NativeFunction(c::GENERATOR_NEXT),
            "return" => Value::NativeFunction(c::GENERATOR_RETURN),
            "throw" => Value::NativeFunction(c::GENERATOR_THROW),
            "Symbol.iterator" => Value::NativeFunction(c::GENERATOR_SYMBOL_ITERATOR),
        };
        let generator_proto_idx = self.gc.allocate(
            &mut self.heap,
            HeapValue::Object(JsObject {
                properties: generator_proto_props,
                prototype: None,
                extensible: true,
            }),
        );
        self.generator_proto_idx = Some(generator_proto_idx);

        let generator_ctor_props = props! {
            "prototype" => Value::Object(generator_proto_idx),
        };
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

        // -------------------------------------------------------------------
        // Buffer (Node-compatible global).
        // -------------------------------------------------------------------
        //
        // `Buffer` is reachable two ways in this runtime:
        //   1. As a native module via `import Buffer from "./buffer.native"`
        //      (see `discover_module` / `create_buffer_module`).
        //   2. As a plain global identifier, matching Node.js and the
        //      v0.5.0 API-completeness tests.
        //
        // The native-module registry already builds a property map for the
        // `buffer` module; we just hoist that map into the global scope so
        // scripts can call `Buffer.from(...)`, `Buffer.byteLength(...)`, etc.
        // without an import.
        {
            use super::native_loader::create_buffer_module;
            let buffer_props = create_buffer_module(&mut self.heap, &mut self.gc);
            // Capture the prototype index before moving `buffer_props`
            // into the heap, so subsequent `Value::Buffer(_)` property
            // lookups (`b.toString(...)`, `b.length`, etc.) can find
            // their methods. Without this, `buffer_proto_idx` would
            // only be set when the module is imported explicitly, and
            // bare `Buffer.from(x).toString("utf8")` would crash with
            // "undefined is not a function".
            if let Some(Value::Object(proto_idx)) = buffer_props.get("prototype") {
                self.buffer_proto_idx = Some(*proto_idx);
            }
            let buffer_idx = self.gc.allocate(
                &mut self.heap,
                HeapValue::Object(JsObject {
                    properties: buffer_props,
                    prototype: None,
                    extensible: true,
                }),
            );
            self.globals
                .insert("Buffer".into(), Value::Object(buffer_idx));
        }

        // -------------------------------------------------------------------
        // process (Node-compatible global, when the `process` feature is on).
        // -------------------------------------------------------------------
        //
        // Mirrors the Buffer registration above. With the `process` feature
        // enabled, the built-in `process` module is exposed as a global so
        // scripts that follow Node's conventions (e.g. `process.pid()`,
        // `process.kill(...)`, `process.uptime()`) work without an
        // explicit import.
        #[cfg(feature = "process")]
        {
            use super::native_loader::create_process_module;
            let process_props = create_process_module(&mut self.heap, &mut self.gc);
            let process_idx = self.gc.allocate(
                &mut self.heap,
                HeapValue::Object(JsObject {
                    properties: process_props,
                    prototype: None,
                    extensible: true,
                }),
            );
            self.globals
                .insert("process".into(), Value::Object(process_idx));
        }
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
