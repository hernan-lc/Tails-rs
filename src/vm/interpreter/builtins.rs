use super::{HeapValue, Interpreter, JsObject};
use crate::objects::js_array::{TypedArray, TypedArrayType};
use crate::objects::Value;
use std::collections::HashMap;

impl Interpreter {
    pub(super) fn init_builtins(&mut self) {
        // Global constants
        self.globals
            .insert("Infinity".into(), Value::Float(f64::INFINITY));
        self.globals
            .insert("-Infinity".into(), Value::Float(f64::NEG_INFINITY));

        // Global functions
        self.globals
            .insert("parseInt".into(), Value::NativeFunction(10));
        self.globals
            .insert("parseFloat".into(), Value::NativeFunction(11));
        self.globals
            .insert("isNaN".into(), Value::NativeFunction(12));
        self.globals
            .insert("isFinite".into(), Value::NativeFunction(13));

        // Timer stubs
        self.globals
            .insert("setTimeout".into(), Value::NativeFunction(14));
        self.globals
            .insert("setInterval".into(), Value::NativeFunction(15));
        self.globals
            .insert("clearTimeout".into(), Value::NativeFunction(16));
        self.globals
            .insert("clearInterval".into(), Value::NativeFunction(17));

        // console object
        let mut console_props = HashMap::new();
        console_props.insert("log".into(), Value::NativeFunction(0));
        console_props.insert("warn".into(), Value::NativeFunction(1));
        console_props.insert("error".into(), Value::NativeFunction(2));
        console_props.insert("info".into(), Value::NativeFunction(3));
        console_props.insert("table".into(), Value::NativeFunction(341));
        console_props.insert("dir".into(), Value::NativeFunction(342));
        console_props.insert("group".into(), Value::NativeFunction(343));
        console_props.insert("groupEnd".into(), Value::NativeFunction(344));
        console_props.insert("groupCollapsed".into(), Value::NativeFunction(345));
        console_props.insert("time".into(), Value::NativeFunction(346));
        console_props.insert("timeEnd".into(), Value::NativeFunction(347));
        console_props.insert("assert".into(), Value::NativeFunction(348));
        console_props.insert("clear".into(), Value::NativeFunction(349));
        console_props.insert("trace".into(), Value::NativeFunction(3)); // Use info for now
        console_props.insert("count".into(), Value::NativeFunction(3)); // Use info for now
        console_props.insert("countReset".into(), Value::NativeFunction(3)); // Use info for now
        console_props.insert("debug".into(), Value::NativeFunction(0)); // Use log for now
        console_props.insert("profile".into(), Value::NativeFunction(3)); // Use info for now
        console_props.insert("profileEnd".into(), Value::NativeFunction(3)); // Use info for now
        console_props.insert("timeLog".into(), Value::NativeFunction(347)); // Use timeEnd for now
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
        object_props.insert("keys".into(), Value::NativeFunction(4));
        object_props.insert("values".into(), Value::NativeFunction(5));
        object_props.insert("entries".into(), Value::NativeFunction(6));
        object_props.insert("assign".into(), Value::NativeFunction(7));
        object_props.insert("defineProperty".into(), Value::NativeFunction(99));
        object_props.insert(
            "getOwnPropertyDescriptor".into(),
            Value::NativeFunction(100),
        );
        object_props.insert("freeze".into(), Value::NativeFunction(101));
        object_props.insert("is".into(), Value::NativeFunction(145));
        object_props.insert("preventExtensions".into(), Value::NativeFunction(146));
        object_props.insert("isExtensible".into(), Value::NativeFunction(147));
        object_props.insert("isSealed".into(), Value::NativeFunction(148));
        object_props.insert("isFrozen".into(), Value::NativeFunction(149));
        object_props.insert("seal".into(), Value::NativeFunction(150));
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
            .insert("Proxy".into(), Value::NativeFunction(85));

        // Reflect
        let mut reflect_props = HashMap::new();
        reflect_props.insert("get".into(), Value::NativeFunction(86));
        reflect_props.insert("set".into(), Value::NativeFunction(87));
        reflect_props.insert("has".into(), Value::NativeFunction(88));
        reflect_props.insert("deleteProperty".into(), Value::NativeFunction(89));
        reflect_props.insert("apply".into(), Value::NativeFunction(90));
        reflect_props.insert("construct".into(), Value::NativeFunction(91));
        reflect_props.insert("ownKeys".into(), Value::NativeFunction(92));
        reflect_props.insert("getOwnPropertyDescriptor".into(), Value::NativeFunction(93));
        reflect_props.insert("defineProperty".into(), Value::NativeFunction(94));
        reflect_props.insert("getPrototypeOf".into(), Value::NativeFunction(95));
        reflect_props.insert("setPrototypeOf".into(), Value::NativeFunction(96));
        reflect_props.insert("isExtensible".into(), Value::NativeFunction(97));
        reflect_props.insert("preventExtensions".into(), Value::NativeFunction(98));
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

        // Symbol - registered as NativeFunction(151) with well-known symbols accessible via GetProperty
        self.globals
            .insert("Symbol".into(), Value::NativeFunction(151));

        // JSON
        let mut json_props = HashMap::new();
        json_props.insert("parse".into(), Value::NativeFunction(8));
        json_props.insert("stringify".into(), Value::NativeFunction(9));
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
        array_props.insert("isArray".into(), Value::NativeFunction(163));
        array_props.insert("from".into(), Value::NativeFunction(164));
        array_props.insert("of".into(), Value::NativeFunction(165));
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
        self.globals
            .insert("BigInt".into(), Value::NativeFunction(169));

        // Encoding
        self.globals
            .insert("atob".into(), Value::NativeFunction(237));
        self.globals
            .insert("btoa".into(), Value::NativeFunction(238));

        // URL (factory function, not constructor)
        self.globals
            .insert("URL".into(), Value::NativeFunction(273));

        // URLSearchParams constructor
        self.globals
            .insert("URLSearchParams".into(), Value::NativeFunction(366));

        // Headers constructor
        self.globals
            .insert("Headers".into(), Value::NativeFunction(370));

        // Request constructor
        self.globals
            .insert("Request".into(), Value::NativeFunction(380));

        // Response constructor - the native function handles construction
        // Static methods are available via the native function dispatch
        self.globals
            .insert("Response".into(), Value::NativeFunction(381));

        // fetch
        self.globals
            .insert("fetch".into(), Value::NativeFunction(307));

        // Date
        let mut date_proto_props = HashMap::new();
        date_proto_props.insert("getTime".into(), Value::NativeFunction(174));
        date_proto_props.insert("getFullYear".into(), Value::NativeFunction(175));
        date_proto_props.insert("getMonth".into(), Value::NativeFunction(176));
        date_proto_props.insert("getDate".into(), Value::NativeFunction(177));
        date_proto_props.insert("getDay".into(), Value::NativeFunction(178));
        date_proto_props.insert("getHours".into(), Value::NativeFunction(179));
        date_proto_props.insert("getMinutes".into(), Value::NativeFunction(180));
        date_proto_props.insert("getSeconds".into(), Value::NativeFunction(181));
        date_proto_props.insert("getMilliseconds".into(), Value::NativeFunction(182));
        date_proto_props.insert("getTimezoneOffset".into(), Value::NativeFunction(183));
        date_proto_props.insert("getUTCFullYear".into(), Value::NativeFunction(184));
        date_proto_props.insert("getUTCMonth".into(), Value::NativeFunction(185));
        date_proto_props.insert("getUTCDate".into(), Value::NativeFunction(186));
        date_proto_props.insert("getUTCDay".into(), Value::NativeFunction(187));
        date_proto_props.insert("getUTCHours".into(), Value::NativeFunction(188));
        date_proto_props.insert("getUTCMinutes".into(), Value::NativeFunction(189));
        date_proto_props.insert("getUTCSeconds".into(), Value::NativeFunction(190));
        date_proto_props.insert("getUTCMilliseconds".into(), Value::NativeFunction(191));
        date_proto_props.insert("setTime".into(), Value::NativeFunction(192));
        date_proto_props.insert("setFullYear".into(), Value::NativeFunction(193));
        date_proto_props.insert("setMonth".into(), Value::NativeFunction(194));
        date_proto_props.insert("setDate".into(), Value::NativeFunction(195));
        date_proto_props.insert("setHours".into(), Value::NativeFunction(196));
        date_proto_props.insert("setMinutes".into(), Value::NativeFunction(197));
        date_proto_props.insert("setSeconds".into(), Value::NativeFunction(198));
        date_proto_props.insert("setMilliseconds".into(), Value::NativeFunction(199));
        date_proto_props.insert("setUTCFullYear".into(), Value::NativeFunction(200));
        date_proto_props.insert("setUTCMonth".into(), Value::NativeFunction(201));
        date_proto_props.insert("setUTCDate".into(), Value::NativeFunction(202));
        date_proto_props.insert("setUTCHours".into(), Value::NativeFunction(203));
        date_proto_props.insert("setUTCMinutes".into(), Value::NativeFunction(204));
        date_proto_props.insert("setUTCSeconds".into(), Value::NativeFunction(205));
        date_proto_props.insert("setUTCMilliseconds".into(), Value::NativeFunction(206));
        date_proto_props.insert("toString".into(), Value::NativeFunction(207));
        date_proto_props.insert("toISOString".into(), Value::NativeFunction(208));
        date_proto_props.insert("toUTCString".into(), Value::NativeFunction(209));
        date_proto_props.insert("toDateString".into(), Value::NativeFunction(210));
        date_proto_props.insert("toTimeString".into(), Value::NativeFunction(211));
        date_proto_props.insert("toJSON".into(), Value::NativeFunction(212));
        date_proto_props.insert("valueOf".into(), Value::NativeFunction(213));
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
            .insert("Date".into(), Value::NativeFunction(170));
        // Store the prototype index for Date constructor
        self.date_proto_idx = Some(date_proto_idx);

        // RegExp
        let mut regexp_proto_props = HashMap::new();
        regexp_proto_props.insert("test".into(), Value::NativeFunction(215));
        regexp_proto_props.insert("exec".into(), Value::NativeFunction(216));
        regexp_proto_props.insert("toString".into(), Value::NativeFunction(217));
        regexp_proto_props.insert("source".into(), Value::NativeFunction(218));
        regexp_proto_props.insert("flags".into(), Value::NativeFunction(219));
        regexp_proto_props.insert("global".into(), Value::NativeFunction(220));
        regexp_proto_props.insert("ignoreCase".into(), Value::NativeFunction(221));
        regexp_proto_props.insert("multiline".into(), Value::NativeFunction(222));
        regexp_proto_props.insert("dotAll".into(), Value::NativeFunction(223));
        regexp_proto_props.insert("unicode".into(), Value::NativeFunction(224));
        regexp_proto_props.insert("sticky".into(), Value::NativeFunction(225));
        regexp_proto_props.insert("lastIndex".into(), Value::NativeFunction(226));
        let regexp_proto_idx = self.gc.allocate(
            &mut self.heap,
            HeapValue::Object(JsObject {
                properties: regexp_proto_props,
                prototype: None,
                extensible: true,
            }),
        );
        // Register RegExp as a NativeFunction for constructor
        self.globals
            .insert("RegExp".into(), Value::NativeFunction(214));
        // Store the prototype index for RegExp constructor
        self.regexp_proto_idx = Some(regexp_proto_idx);

        // Math
        let mut math_props = HashMap::new();
        math_props.insert("PI".into(), Value::Float(std::f64::consts::PI));
        math_props.insert("E".into(), Value::Float(std::f64::consts::E));
        math_props.insert("abs".into(), Value::NativeFunction(18));
        math_props.insert("floor".into(), Value::NativeFunction(19));
        math_props.insert("ceil".into(), Value::NativeFunction(20));
        math_props.insert("round".into(), Value::NativeFunction(21));
        math_props.insert("min".into(), Value::NativeFunction(22));
        math_props.insert("max".into(), Value::NativeFunction(23));
        math_props.insert("random".into(), Value::NativeFunction(24));
        math_props.insert("pow".into(), Value::NativeFunction(25));
        math_props.insert("sqrt".into(), Value::NativeFunction(26));
        math_props.insert("log".into(), Value::NativeFunction(27));
        math_props.insert("sin".into(), Value::NativeFunction(28));
        math_props.insert("cos".into(), Value::NativeFunction(29));
        math_props.insert("tan".into(), Value::NativeFunction(30));
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
        number_props.insert("isFinite".into(), Value::NativeFunction(13));
        number_props.insert("isNaN".into(), Value::NativeFunction(12));
        number_props.insert("parseFloat".into(), Value::NativeFunction(11));
        number_props.insert("parseInt".into(), Value::NativeFunction(10));
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
        promise_proto_props.insert("then".into(), Value::NativeFunction(78));
        promise_proto_props.insert("catch".into(), Value::NativeFunction(79));
        promise_proto_props.insert("finally".into(), Value::NativeFunction(80));
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
        promise_ctor_props.insert("resolve".into(), Value::NativeFunction(81));
        promise_ctor_props.insert("reject".into(), Value::NativeFunction(82));
        promise_ctor_props.insert("all".into(), Value::NativeFunction(83));
        promise_ctor_props.insert("race".into(), Value::NativeFunction(84));
        self.gc.allocate(
            &mut self.heap,
            HeapValue::Object(JsObject {
                properties: promise_ctor_props,
                prototype: None,
                extensible: true,
            }),
        );
        self.globals
            .insert("Promise".into(), Value::NativeFunction(77));

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
            .insert("Error".into(), Value::NativeFunction(72));

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
        self.globals
            .insert("TypeError".into(), Value::NativeFunction(73));

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
        self.globals
            .insert("ReferenceError".into(), Value::NativeFunction(74));

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
        self.globals
            .insert("SyntaxError".into(), Value::NativeFunction(75));

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
        self.globals
            .insert("RangeError".into(), Value::NativeFunction(76));

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
            proto_props.insert("length".into(), Value::NativeFunction(107));
            proto_props.insert("get".into(), Value::NativeFunction(105));
            proto_props.insert("set".into(), Value::NativeFunction(106));
            proto_props.insert("subarray".into(), Value::NativeFunction(110));
            proto_props.insert("slice".into(), Value::NativeFunction(111));
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
            ctor_props.insert("from".into(), Value::NativeFunction(103));
            ctor_props.insert("of".into(), Value::NativeFunction(104));
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
        map_proto_props.insert("get".into(), Value::NativeFunction(113));
        map_proto_props.insert("set".into(), Value::NativeFunction(114));
        map_proto_props.insert("has".into(), Value::NativeFunction(115));
        map_proto_props.insert("delete".into(), Value::NativeFunction(116));
        map_proto_props.insert("clear".into(), Value::NativeFunction(117));
        map_proto_props.insert("size".into(), Value::NativeFunction(118));
        map_proto_props.insert("forEach".into(), Value::NativeFunction(119));
        map_proto_props.insert("keys".into(), Value::NativeFunction(120));
        map_proto_props.insert("values".into(), Value::NativeFunction(121));
        map_proto_props.insert("entries".into(), Value::NativeFunction(122));
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
            .insert("Map".into(), Value::NativeFunction(112));

        // Set
        let mut set_proto_props = HashMap::new();
        set_proto_props.insert("add".into(), Value::NativeFunction(124));
        set_proto_props.insert("has".into(), Value::NativeFunction(125));
        set_proto_props.insert("delete".into(), Value::NativeFunction(126));
        set_proto_props.insert("clear".into(), Value::NativeFunction(127));
        set_proto_props.insert("size".into(), Value::NativeFunction(128));
        set_proto_props.insert("forEach".into(), Value::NativeFunction(129));
        set_proto_props.insert("values".into(), Value::NativeFunction(130));
        set_proto_props.insert("keys".into(), Value::NativeFunction(131));
        set_proto_props.insert("entries".into(), Value::NativeFunction(132));
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
            .insert("Set".into(), Value::NativeFunction(123));

        // WeakMap
        let mut weakmap_proto_props = HashMap::new();
        weakmap_proto_props.insert("get".into(), Value::NativeFunction(134));
        weakmap_proto_props.insert("set".into(), Value::NativeFunction(135));
        weakmap_proto_props.insert("has".into(), Value::NativeFunction(136));
        weakmap_proto_props.insert("delete".into(), Value::NativeFunction(137));
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
        self.globals
            .insert("WeakMap".into(), Value::NativeFunction(133));

        // WeakSet
        let mut weakset_proto_props = HashMap::new();
        weakset_proto_props.insert("add".into(), Value::NativeFunction(139));
        weakset_proto_props.insert("has".into(), Value::NativeFunction(140));
        weakset_proto_props.insert("delete".into(), Value::NativeFunction(141));
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
        self.globals
            .insert("WeakSet".into(), Value::NativeFunction(138));

        // Generator
        let mut generator_proto_props = HashMap::new();
        generator_proto_props.insert("next".into(), Value::NativeFunction(142));
        generator_proto_props.insert("return".into(), Value::NativeFunction(143));
        generator_proto_props.insert("throw".into(), Value::NativeFunction(144));
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
        self.globals
            .insert("WebSocket".into(), Value::NativeFunction(359));
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
