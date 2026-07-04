use super::{HeapValue, Interpreter, PropertyStorage};
use crate::errors::{Error, Result};
use crate::objects::js_promise::PromiseState;
use crate::objects::Value;
use crate::runtime_env::native_fns::constants as c;

fn key_to_str(key: &Value) -> Option<String> {
    match key {
        Value::String(s) => Some(s.clone()),
        Value::Cons(c) => Some(c.flatten()),
        _ => None,
    }
}

fn check_call_apply_bind(key_str: &str) -> Option<Value> {
    match key_str {
        "call" => Some(Value::NativeFunction(c::FUNCTION_CALL)),
        "apply" => Some(Value::NativeFunction(c::FUNCTION_APPLY)),
        "bind" => Some(Value::NativeFunction(c::FUNCTION_BIND)),
        _ => None,
    }
}

impl Interpreter {
    pub fn new_object(&mut self) -> Value {
        let idx = self.heap.len();
        self.heap.push(HeapValue::Object(
            crate::vm::interpreter::heap_types::JsObject::new(),
        ));
        Value::Object(idx)
    }

    pub fn new_array(&mut self) -> Value {
        let idx = self.heap.len();
        self.heap.push(HeapValue::Array(
            crate::vm::interpreter::heap_types::JsArray {
                elements: Vec::new(),
            },
        ));
        Value::Array(idx)
    }

    pub fn get_property_str(&mut self, object: &Value, key: &str) -> Option<Value> {
        self.get_property(object, &Value::String(key.to_string()))
            .ok()
    }

    pub fn set_property_str(&mut self, object: &Value, key: &str, value: Value) {
        let _ = self.set_property(object, &Value::String(key.to_string()), value);
    }

    pub fn get_array_length(&self, array: &Value) -> Option<i64> {
        match array {
            Value::Array(arr_idx) => {
                if let HeapValue::Array(arr) = &self.heap[*arr_idx] {
                    Some(arr.elements.len() as i64)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn get_array_element(&self, array: &Value, index: usize) -> Option<Value> {
        match array {
            Value::Array(arr_idx) => {
                if let HeapValue::Array(arr) = &self.heap[*arr_idx] {
                    arr.elements.get(index).cloned()
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn push_array_element(&mut self, array: &Value, value: Value) {
        if let Value::Array(arr_idx) = array {
            if let HeapValue::Array(arr) = &mut self.heap[*arr_idx] {
                arr.elements.push(value);
            }
        }
    }

    pub fn set_property(&mut self, object: &Value, key: &Value, value: Value) -> Result<()> {
        let key_str = match key_to_str(key) {
            Some(s) => s,
            None => return Ok(()),
        };
        match object {
            Value::Object(obj_idx) => {
                if let HeapValue::Object(obj) = &mut self.heap[*obj_idx] {
                    obj.properties.insert(key_str.to_string(), value);
                }
            }
            Value::Array(arr_idx) => {
                if let HeapValue::Array(arr) = &mut self.heap[*arr_idx] {
                    if let Ok(index) = key_str.parse::<usize>() {
                        if index < arr.elements.len() {
                            arr.elements[index] = value;
                        }
                    }
                }
            }
            Value::Function(func_idx) => {
                if let HeapValue::Function(f) = &mut self.heap[*func_idx] {
                    f.properties.insert(key_str.to_string(), value);
                }
            }
            Value::Buffer(buf_idx) => {
                if let HeapValue::Buffer(buf) = &mut self.heap[*buf_idx] {
                    if let Ok(index) = key_str.parse::<usize>() {
                        if index < buf.len() {
                            buf[index] = to_i64_value(&value) as u8;
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub(super) fn get_property(&mut self, object: &Value, key: &Value) -> Result<Value> {
        self.get_property_with_this(object, key, object)
    }

    pub(crate) fn get_property_with_this(
        &mut self,
        object: &Value,
        key: &Value,
        this: &Value,
    ) -> Result<Value> {
        match object {
            Value::Null | Value::Undefined => {
                return Err(self.err_at_location(Error::TypeError(format!(
                    "Cannot read properties of {} (reading '{}')",
                    self.value_to_string(object),
                    self.value_to_string(key)
                ))));
            }
            Value::Object(obj_idx) => {
                if let HeapValue::Object(obj) = &self.heap[*obj_idx] {
                    let key_str = match key_to_str(key) {
                        Some(s) => s,
                        None => return Ok(Value::Undefined),
                    };
                    if let Some(val) = obj.properties.get(&key_str) {
                        return Ok(val.clone());
                    }
                    if let Some(getter_val) = find_accessor(&obj.properties, "__getter_", &key_str) {
                        return self.call_value(&getter_val, this, &[]);
                    }
                    if let Some(proto_idx) = obj.prototype {
                        let proto_val = Value::Object(proto_idx);
                        return self.get_property_with_this(&proto_val, key, this);
                    }
                }
            }
            Value::Array(arr_idx) => {
                if let HeapValue::Array(arr) = &self.heap[*arr_idx] {
                    match key {
                        Value::String(key_str) => {
                            if key_str == "length" {
                                return Ok(Value::Float(arr.elements.len() as f64));
                            }
                            if let Ok(index) = key_str.parse::<usize>() {
                                return Ok(arr
                                    .elements
                                    .get(index)
                                    .cloned()
                                    .unwrap_or(Value::Undefined));
                            }
                            return self.get_array_method(key_str);
                        }
                        Value::Cons(c) => {
                            let s = c.flatten();
                            if s == "length" {
                                return Ok(Value::Float(arr.elements.len() as f64));
                            }
                            if let Ok(index) = s.parse::<usize>() {
                                return Ok(arr
                                    .elements
                                    .get(index)
                                    .cloned()
                                    .unwrap_or(Value::Undefined));
                            }
                            return self.get_array_method(&s);
                        }
                        Value::Integer(index) => {
                            let idx = *index as usize;
                            return Ok(arr.elements.get(idx).cloned().unwrap_or(Value::Undefined));
                        }
                        Value::Float(f) => {
                            let idx = *f as usize;
                            return Ok(arr.elements.get(idx).cloned().unwrap_or(Value::Undefined));
                        }
                        Value::Symbol(sym_id) if *sym_id == crate::objects::SYMBOL_ITERATOR => {
                            return Ok(Value::NativeFunction(c::ARRAY_ITERATOR));
                        }
                        _ => {}
                    }
                }
            }
            Value::String(s) => {
                return self.get_property_from_primitive_string(s, key);
            }
            Value::Cons(c) => {
                let flat = c.flatten();
                return self.get_property_from_primitive_string(&flat, key);
            }
            Value::Integer(_) | Value::Float(_) => {
                return self.get_property_from_primitive_number(object, key);
            }
            Value::Boolean(_) => {
                return self.get_property_from_primitive_boolean(object, key);
            }
            Value::Function(func_idx) => {
                let key_str = match key_to_str(key) {
                    Some(s) => s,
                    None => return Ok(Value::Undefined),
                };
                if let Some(val) = check_call_apply_bind(&key_str) {
                    return Ok(val);
                }
                if key_str == "prototype" {
                    if let HeapValue::Function(f) = &self.heap[*func_idx] {
                        if let Some(proto_idx) = f.prototype {
                            return Ok(Value::Object(proto_idx));
                        }
                    }
                }
                if let HeapValue::Function(f) = &self.heap[*func_idx] {
                    if let Some(val) = f.properties.get(&key_str) {
                        return Ok(val.clone());
                    }
                }
            }
            Value::Promise(promise_idx) => {
                let key_str = match key_to_str(key) {
                    Some(s) => s,
                    None => return Ok(Value::Undefined),
                };
                match key_str.as_str() {
                    "then" => return Ok(Value::NativeFunction(c::PROMISE_THEN)),
                    "catch" => return Ok(Value::NativeFunction(c::PROMISE_CATCH)),
                    "finally" => return Ok(Value::NativeFunction(c::PROMISE_FINALLY)),
                    "state" => {
                        if let HeapValue::Promise(p) = &self.heap[*promise_idx] {
                            return Ok(Value::String(format!("{:?}", p.state)));
                        }
                    }
                    "value" => {
                        if let HeapValue::Promise(p) = &self.heap[*promise_idx] {
                            if let PromiseState::Fulfilled(v) = &p.state {
                                return Ok(v.clone());
                            }
                        }
                    }
                    "reason" => {
                        if let HeapValue::Promise(p) = &self.heap[*promise_idx] {
                            if let PromiseState::Rejected(r) = &p.state {
                                return Ok(r.clone());
                            }
                        }
                    }
                    _ => {}
                }
            }
            Value::NativeFunction(idx) => {
                let key_str = match key_to_str(key) {
                    Some(s) => s,
                    None => return Ok(Value::Undefined),
                };
                if let Some(val) = check_call_apply_bind(&key_str) {
                    return Ok(val);
                }
                if *idx == c::PROMISE_CONSTRUCTOR {
                    match key_str.as_str() {
                        "resolve" => return Ok(Value::NativeFunction(c::PROMISE_RESOLVE)),
                        "reject" => return Ok(Value::NativeFunction(c::PROMISE_REJECT)),
                        "all" => return Ok(Value::NativeFunction(c::PROMISE_ALL)),
                        "race" => return Ok(Value::NativeFunction(c::PROMISE_RACE)),
                        "allSettled" => return Ok(Value::NativeFunction(c::PROMISE_ALL_SETTLED)),
                        "any" => return Ok(Value::NativeFunction(c::PROMISE_ANY)),
                        "withResolvers" => {
                            return Ok(Value::NativeFunction(c::PROMISE_WITH_RESOLVERS))
                        }
                        _ => {}
                    }
                }
                if *idx == c::SYMBOL_CONSTRUCTOR {
                    match key_str.as_str() {
                        "for" => return Ok(Value::NativeFunction(c::SYMBOL_FOR)),
                        "keyFor" => return Ok(Value::NativeFunction(c::SYMBOL_KEY_FOR)),
                        "iterator" => return Ok(Value::Symbol(crate::objects::SYMBOL_ITERATOR)),
                        "toStringTag" => {
                            return Ok(Value::Symbol(crate::objects::SYMBOL_TO_STRING_TAG))
                        }
                        "hasInstance" => {
                            return Ok(Value::Symbol(crate::objects::SYMBOL_HAS_INSTANCE))
                        }
                        "toPrimitive" => {
                            return Ok(Value::Symbol(crate::objects::SYMBOL_TO_PRIMITIVE))
                        }
                        "species" => return Ok(Value::Symbol(crate::objects::SYMBOL_SPECIES)),
                        "unscopables" => {
                            return Ok(Value::Symbol(crate::objects::SYMBOL_UNSCOPABLES))
                        }
                        "asyncIterator" => {
                            return Ok(Value::Symbol(crate::objects::SYMBOL_ASYNC_ITERATOR))
                        }
                        _ => {}
                    }
                }
                if *idx == c::DATE_CONSTRUCTOR {
                    match key_str.as_str() {
                        "now" => return Ok(Value::NativeFunction(c::DATE_NOW)),
                        "parse" => return Ok(Value::NativeFunction(c::DATE_PARSE)),
                        "UTC" => return Ok(Value::NativeFunction(c::DATE_UTC)),
                        _ => {}
                    }
                }
                if *idx == c::ARRAY_CONSTRUCTOR {
                    match key_str.as_str() {
                        "isArray" => return Ok(Value::NativeFunction(c::ARRAY_IS_ARRAY)),
                        "from" => return Ok(Value::NativeFunction(c::ARRAY_FROM)),
                        "of" => return Ok(Value::NativeFunction(c::ARRAY_OF)),
                        _ => {}
                    }
                }
                if *idx == c::URL_CONSTRUCTOR {
                    match key_str.as_str() {
                        "canParse" => return Ok(Value::NativeFunction(c::URL_CAN_PARSE)),
                        "parse" => return Ok(Value::NativeFunction(c::URL_PARSE)),
                        _ => {}
                    }
                }
                if *idx == c::RESPONSE_CONSTRUCTOR {
                    match key_str.as_str() {
                        "json" => return Ok(Value::NativeFunction(c::RESPONSE_JSON_STATIC)),
                        "error" => return Ok(Value::NativeFunction(c::RESPONSE_ERROR)),
                        "redirect" => return Ok(Value::NativeFunction(c::RESPONSE_REDIRECT)),
                        _ => {}
                    }
                }
            }
            Value::Proxy(proxy_idx) => {
                if let HeapValue::Proxy(proxy) = &self.heap[*proxy_idx] {
                    let handler = proxy.handler.clone();
                    let target = proxy.target.clone();
                    let trap = self.get_property(&handler, &Value::String("get".to_string()));
                    match &trap {
                        Ok(Value::Function(_)) | Ok(Value::NativeFunction(_)) => {
                            let trap_val = trap.unwrap();
                            let trap_result = self.call_value(
                                &trap_val,
                                &handler,
                                &[target, key.clone(), this.clone()],
                            );
                            if let Ok(v) = trap_result {
                                return Ok(v);
                            }
                        }
                        _ => {
                            return self.get_property_with_this(&target, key, this);
                        }
                    }
                }
            }
            Value::Date(_date_idx) => {
                let key_str = match key_to_str(key) {
                    Some(s) => s,
                    None => return Ok(Value::Undefined),
                };
                let _ = key_str;
                let proto_idx = self.date_proto_idx;
                if let Some(proto_idx) = proto_idx {
                    let proto_val = Value::Object(proto_idx);
                    return self.get_property_with_this(&proto_val, key, this);
                }
            }
            Value::RegExp(_re_idx) => {
                let key_str = match key_to_str(key) {
                    Some(s) => s,
                    None => return Ok(Value::Undefined),
                };
                let _ = key_str;
                let proto_idx = self.regexp_proto_idx;
                if let Some(proto_idx) = proto_idx {
                    let proto_val = Value::Object(proto_idx);
                    return self.get_property_with_this(&proto_val, key, this);
                }
            }
            Value::Buffer(_buf_idx) => {
                let key_str = match key_to_str(key) {
                    Some(s) => s,
                    None => return Ok(Value::Undefined),
                };
                if key_str == "length" {
                    if let Value::Buffer(bidx) = this {
                        if let HeapValue::Buffer(buf) = &self.heap[*bidx] {
                            return Ok(Value::Integer(buf.len() as i64));
                        }
                    }
                }
                if let Some(proto_idx) = self.buffer_proto_idx {
                    let proto_val = Value::Object(proto_idx);
                    return self.get_property_with_this(&proto_val, key, this);
                }
            }
            Value::Map(_map_idx) => {
                let key_str = match key_to_str(key) {
                    Some(s) => s,
                    None => return Ok(Value::Undefined),
                };
                match key_str.as_str() {
                    "size" => {
                        if let Value::Map(map_idx) = this {
                            if let HeapValue::Map(map) = &self.heap[*map_idx] {
                                return Ok(Value::Float(map.size() as f64));
                            }
                        }
                    }
                    "get" => return Ok(Value::NativeFunction(c::MAP_GET)),
                    "set" => return Ok(Value::NativeFunction(c::MAP_SET)),
                    "has" => return Ok(Value::NativeFunction(c::MAP_HAS)),
                    "delete" => return Ok(Value::NativeFunction(c::MAP_DELETE)),
                    "clear" => return Ok(Value::NativeFunction(c::MAP_CLEAR)),
                    "forEach" => return Ok(Value::NativeFunction(c::MAP_FOR_EACH)),
                    "keys" => return Ok(Value::NativeFunction(c::MAP_KEYS)),
                    "values" => return Ok(Value::NativeFunction(c::MAP_VALUES)),
                    "entries" => return Ok(Value::NativeFunction(c::MAP_ENTRIES)),
                    _ => {}
                }
            }
            Value::Set(_set_idx) => {
                let key_str = match key_to_str(key) {
                    Some(s) => s,
                    None => return Ok(Value::Undefined),
                };
                match key_str.as_str() {
                    "size" => {
                        if let Value::Set(set_idx) = this {
                            if let HeapValue::Set(set) = &self.heap[*set_idx] {
                                return Ok(Value::Float(set.size() as f64));
                            }
                        }
                    }
                    "add" => return Ok(Value::NativeFunction(c::SET_ADD)),
                    "has" => return Ok(Value::NativeFunction(c::SET_HAS)),
                    "delete" => return Ok(Value::NativeFunction(c::SET_DELETE)),
                    "clear" => return Ok(Value::NativeFunction(c::SET_CLEAR)),
                    "forEach" => return Ok(Value::NativeFunction(c::SET_FOR_EACH)),
                    "values" => return Ok(Value::NativeFunction(c::SET_VALUES)),
                    "keys" => return Ok(Value::NativeFunction(c::SET_KEYS)),
                    "entries" => return Ok(Value::NativeFunction(c::SET_ENTRIES)),
                    _ => {}
                }
            }
            Value::WeakMap(_weakmap_idx) => {
                let key_str = match key_to_str(key) {
                    Some(s) => s,
                    None => return Ok(Value::Undefined),
                };
                match key_str.as_str() {
                    "get" => return Ok(Value::NativeFunction(c::WEAKMAP_GET)),
                    "set" => return Ok(Value::NativeFunction(c::WEAKMAP_SET)),
                    "has" => return Ok(Value::NativeFunction(c::WEAKMAP_HAS)),
                    "delete" => return Ok(Value::NativeFunction(c::WEAKMAP_DELETE)),
                    _ => {}
                }
            }
            Value::WeakSet(_weakset_idx) => {
                let key_str = match key_to_str(key) {
                    Some(s) => s,
                    None => return Ok(Value::Undefined),
                };
                match key_str.as_str() {
                    "add" => return Ok(Value::NativeFunction(c::WEAKSET_ADD)),
                    "has" => return Ok(Value::NativeFunction(c::WEAKSET_HAS)),
                    "delete" => return Ok(Value::NativeFunction(c::WEAKSET_DELETE)),
                    _ => {}
                }
            }
            Value::TypedArray(_ta_idx) => {
                let key_str = match key_to_str(key) {
                    Some(s) => s,
                    None => return Ok(Value::Undefined),
                };
                match key_str.as_str() {
                    "length" => {
                        if let Value::TypedArray(ta_idx) = this {
                            if let HeapValue::TypedArray(ta) = &self.heap[*ta_idx] {
                                let elem_size =
                                    crate::objects::js_array::TypedArray::element_size(&ta.kind);
                                return Ok(Value::Float((ta.byte_length / elem_size) as f64));
                            }
                        }
                    }
                    "byteLength" => {
                        if let Value::TypedArray(ta_idx) = this {
                            if let HeapValue::TypedArray(ta) = &self.heap[*ta_idx] {
                                return Ok(Value::Float(ta.byte_length as f64));
                            }
                        }
                    }
                    "byteOffset" => {
                        if let Value::TypedArray(ta_idx) = this {
                            if let HeapValue::TypedArray(ta) = &self.heap[*ta_idx] {
                                return Ok(Value::Float(ta.byte_offset as f64));
                            }
                        }
                    }
                    "get" => return Ok(Value::NativeFunction(c::TYPED_ARRAY_GET)),
                    "set" => return Ok(Value::NativeFunction(c::TYPED_ARRAY_SET)),
                    "subarray" => return Ok(Value::NativeFunction(c::TYPED_ARRAY_SUBARRAY)),
                    "slice" => return Ok(Value::NativeFunction(c::TYPED_ARRAY_SLICE)),
                    _ => {}
                }
            }
            Value::Generator(_gen_idx) => {
                if let Some(proto_idx) = self.generator_proto_idx {
                    if let Value::Symbol(sym_id) = key {
                        if *sym_id == crate::objects::SYMBOL_ITERATOR {
                            return Ok(Value::NativeFunction(c::GENERATOR_SYMBOL_ITERATOR));
                        }
                    }
                    let proto_val = Value::Object(proto_idx);
                    return self.get_property_with_this(&proto_val, key, this);
                }
            }
            Value::NativeObject(obj_id) => {
                let key_str = match key_to_str(key) {
                    Some(s) => s,
                    None => return Ok(Value::Undefined),
                };
                if let Some(methods) = self.native_object_methods.get(&obj_id.0) {
                    if let Some(method) = methods.get(&key_str) {
                        return Ok(method.clone());
                    }
                }
            }
            _ => {}
        }
        Ok(Value::Undefined)
    }

    pub(super) fn get_array_method(&self, name: &str) -> Result<Value> {
        let idx = match name {
            "push" => 31,
            "pop" => 32,
            "shift" => 33,
            "unshift" => 34,
            "slice" => 35,
            "splice" => 36,
            "indexOf" => 37,
            "includes" => 38,
            "find" => 39,
            "findIndex" => 40,
            "map" => 41,
            "filter" => 42,
            "reduce" => 43,
            "forEach" => 44,
            "some" => 45,
            "every" => 46,
            "join" => 47,
            "reverse" => 48,
            "sort" => 49,
            "concat" => 50,
            "flat" => 51,
            "copyWithin" => 157,
            "fill" => 158,
            "findLast" => 159,
            "findLastIndex" => 160,
            "flatMap" => 161,
            "lastIndexOf" => 162,
            _ => return Ok(Value::Undefined),
        };
        Ok(Value::NativeFunction(idx))
    }

    pub(super) fn get_property_from_primitive_string(&self, s: &str, key: &Value) -> Result<Value> {
        let key_str = match key_to_str(key) {
            Some(s) => s,
            None => return Ok(Value::Undefined),
        };
        if key_str == "length" {
            return Ok(Value::Float(s.len() as f64));
        }
        self.get_string_method(&key_str)
    }

    pub(super) fn get_string_method(&self, name: &str) -> Result<Value> {
        let idx = match name {
            "charAt" => 52,
            "charCodeAt" => 53,
            "slice" => 54,
            "substring" => 55,
            "indexOf" => 56,
            "includes" => 57,
            "replace" => 58,
            "split" => 59,
            "trim" => 60,
            "toLowerCase" => 61,
            "toUpperCase" => 62,
            "startsWith" => 63,
            "endsWith" => 64,
            "repeat" => 65,
            "padStart" => 66,
            "padEnd" => 67,
            "matchAll" => 393,
            _ => return Ok(Value::Undefined),
        };
        Ok(Value::NativeFunction(idx))
    }

    pub(super) fn get_property_from_primitive_number(
        &self,
        _n: &Value,
        key: &Value,
    ) -> Result<Value> {
        let key_str = match key_to_str(key) {
            Some(s) => s,
            None => return Ok(Value::Undefined),
        };
        match key_str.as_str() {
            "toString" | "toFixed" | "valueOf" | "toExponential" | "toPrecision"
            | "toLocaleString" => {
                return Ok(self.make_native_number_method(&key_str));
            }
            _ => {}
        }
        Ok(Value::Undefined)
    }

    pub(super) fn get_property_from_primitive_boolean(
        &self,
        _b: &Value,
        key: &Value,
    ) -> Result<Value> {
        let key_str = match key_to_str(key) {
            Some(s) => s,
            None => return Ok(Value::Undefined),
        };
        match key_str.as_str() {
            "toString" | "valueOf" => {
                return Ok(self.make_native_boolean_method(&key_str));
            }
            _ => {}
        }
        Ok(Value::Undefined)
    }

    pub(super) fn make_native_number_method(&self, name: &str) -> Value {
        match name {
            "toFixed" => Value::NativeFunction(c::NUMBER_TO_FIXED),
            "toString" | "toLocaleString" => Value::NativeFunction(c::NUMBER_TO_STRING),
            "valueOf" => Value::NativeFunction(c::NUMBER_VALUE_OF),
            "toExponential" => Value::NativeFunction(c::NUMBER_TO_EXPONENTIAL),
            "toPrecision" => Value::NativeFunction(c::NUMBER_TO_PRECISION),
            _ => Value::Undefined,
        }
    }

    pub(super) fn make_native_boolean_method(&self, name: &str) -> Value {
        match name {
            "toString" | "toLocaleString" => Value::NativeFunction(c::BOOLEAN_TO_STRING),
            "valueOf" => Value::NativeFunction(c::BOOLEAN_VALUE_OF),
            _ => Value::Undefined,
        }
    }

    pub(crate) fn delete_property(&mut self, object: &Value, key: &Value) -> Value {
        let key_str = match key_to_str(key) {
            Some(s) => s,
            None => return Value::Boolean(true),
        };
        match object {
            Value::Object(obj_idx) => {
                if let HeapValue::Object(obj) = &mut self.heap[*obj_idx] {
                    if obj.properties.remove(&key_str).is_some() {
                        return Value::Boolean(true);
                    }
                }
                Value::Boolean(false)
            }
            Value::Array(arr_idx) => {
                if let Ok(index) = key_str.parse::<usize>() {
                    if let HeapValue::Array(arr) = &mut self.heap[*arr_idx] {
                        if index < arr.elements.len() {
                            arr.elements[index] = Value::Undefined;
                            return Value::Boolean(true);
                        }
                    }
                }
                Value::Boolean(false)
            }
            _ => Value::Boolean(true),
        }
    }

    pub(super) fn instanceof_check(&mut self, left: &Value, right: &Value) -> Result<Value> {
        let proto_key = Value::String("prototype".to_string());
        let right_proto = match self.get_property(right, &proto_key) {
            Ok(val) => val,
            Err(_) => return Ok(Value::Boolean(false)),
        };

        let proto_idx = match &right_proto {
            Value::Object(idx) => *idx,
            _ => return Ok(Value::Boolean(false)),
        };

        let mut current = left.clone();
        loop {
            match &current {
                Value::Object(obj_idx) => {
                    if let HeapValue::Object(obj) = &self.heap[*obj_idx] {
                        if obj.prototype == Some(proto_idx) {
                            return Ok(Value::Boolean(true));
                        }
                        if let Some(parent_idx) = obj.prototype {
                            current = Value::Object(parent_idx);
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                Value::Array(_arr_idx) => {
                    break;
                }
                _ => break,
            }
        }
        Ok(Value::Boolean(false))
    }

    pub(crate) fn in_check_mut(&mut self, key: &Value, object: &Value) -> Result<Value> {
        let key_str = match key_to_str(key) {
            Some(s) => s,
            None => return Ok(Value::Boolean(false)),
        };
        match object {
            Value::Object(obj_idx) => {
                if let HeapValue::Object(obj) = &self.heap[*obj_idx] {
                    if obj.properties.contains_key(&key_str) {
                        return Ok(Value::Boolean(true));
                    }
                    if find_accessor(&obj.properties, "__getter_", &key_str).is_some()
                        || find_accessor(&obj.properties, "__setter_", &key_str).is_some()
                    {
                        return Ok(Value::Boolean(true));
                    }
                    if let Some(proto_idx) = obj.prototype {
                        let proto_val = Value::Object(proto_idx);
                        return self.in_check_mut(key, &proto_val);
                    }
                }
                Ok(Value::Boolean(false))
            }
            Value::Array(arr_idx) => {
                if let HeapValue::Array(arr) = &self.heap[*arr_idx] {
                    if key_str == "length" {
                        return Ok(Value::Boolean(true));
                    }
                    if let Ok(index) = key_str.parse::<usize>() {
                        return Ok(Value::Boolean(index < arr.elements.len()));
                    }
                }
                Ok(Value::Boolean(false))
            }
            Value::String(s) => {
                if key_str == "length" {
                    return Ok(Value::Boolean(true));
                }
                if let Ok(index) = key_str.parse::<usize>() {
                    return Ok(Value::Boolean(index < s.len()));
                }
                Ok(Value::Boolean(false))
            }
            Value::Proxy(proxy_idx) => {
                if let HeapValue::Proxy(proxy) = &self.heap[*proxy_idx] {
                    let handler = proxy.handler.clone();
                    let target = proxy.target.clone();
                    let trap = self.get_property(&handler, &Value::String("has".to_string()));
                    if let Ok(Value::Function(_)) | Ok(Value::NativeFunction(_)) = &trap {
                        let trap_result = self.call_value(&trap?, &handler, &[target, key.clone()]);
                        if let Ok(v) = trap_result {
                            return Ok(v);
                        }
                    } else {
                        return self.in_check_mut(key, &target);
                    }
                }
                Ok(Value::Boolean(false))
            }
            _ => Ok(Value::Boolean(false)),
        }
    }

    pub(crate) fn call_proxy_trap(
        &mut self,
        handler: &Value,
        trap_name: &str,
        args: &[Value],
    ) -> Result<Value> {
        let trap = self.get_property(handler, &Value::String(trap_name.to_string()))?;
        if matches!(trap, Value::Undefined) {
            return Err(self.err_at_location(Error::RuntimeError(format!(
                "Proxy has no '{}' trap",
                trap_name
            ))));
        }
        self.call_value(&trap, handler, args)
    }
}

/// Look for an accessor method (`__getter_<name>` or `__setter_<name>`)
/// on an object's own property map. Returns a clone of the value if
/// found, `None` otherwise.
///
/// This helper replaces the previous `format!("__getter_{}", key)`
/// allocation that fired on *every* property miss (not just for keys
/// that actually have accessors). For an object with N properties the
/// scan is O(N) but it allocates nothing, and for the common case
/// (no accessors on this key) it falls through to the prototype walk
/// immediately.
///
/// Hot path: for the typical 0–8 property object the scan visits
/// at most 8 short string keys (the `__getter_` / `__setter_`
/// prefix is 9 bytes; the iterator can reject almost every key with
/// a length check before the prefix compare).
fn find_accessor(properties: &PropertyStorage, prefix: &str, key: &str) -> Option<Value> {
    let needed_len = prefix.len() + key.len();
    for (k, v) in properties {
        if k.len() == needed_len && k.starts_with(prefix) && k.ends_with(key) {
            return Some(v.clone());
        }
    }
    None
}

fn to_i64_value(v: &Value) -> i64 {
    match v {
        Value::Integer(n) => *n,
        Value::Float(n) => *n as i64,
        Value::Boolean(b) => {
            if *b {
                1
            } else {
                0
            }
        }
        Value::String(s) => s.parse::<i64>().unwrap_or(0),
        Value::Cons(c) => c.flatten().parse::<i64>().unwrap_or(0),
        Value::Null => 0,
        _ => 0,
    }
}
