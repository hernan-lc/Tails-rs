use super::{HeapValue, Interpreter, JsObject, PropertyStorage};
use crate::errors::{Error, Result};
use crate::objects::js_promise::PromiseState;
use crate::objects::Value;
use crate::runtime_env::native_fns::constants as c;
use crate::well_known as wk;

fn key_to_str(key: &Value) -> Option<String> {
    match key {
        Value::String(s) => Some(s.to_string()),
        Value::Cons(c) => Some(c.flatten()),
        Value::Integer(n) => Some(n.to_string()),
        Value::Float(n) => Some(((*n) as i64).to_string()),
        Value::Symbol(id) => Some(format!("__sym_{}", id)),
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
            crate::vm::interpreter::heap_types::JsObject::with_prototype(self.object_proto_idx),
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
        self.get_property(object, &Value::from_string(key.to_string()))
            .ok()
    }

    pub fn set_property_str(&mut self, object: &Value, key: &str, value: Value) {
        let _ = self.set_property(object, &Value::from_string(key.to_string()), value);
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
                    // Defining a data property replaces any existing accessor.
                    obj.properties.remove(&format!("__getter_{}", key_str));
                    obj.properties.remove(&format!("__setter_{}", key_str));
                    obj.properties.insert(key_str.to_string(), value);
                }
            }
            Value::Array(arr_idx) => {
                if let HeapValue::Array(arr) = &mut self.heap[*arr_idx] {
                    if key_str == wk::LENGTH {
                        let new_len = to_i64_value(&value).max(0) as usize;
                        if new_len < arr.elements.len() {
                            arr.elements.truncate(new_len);
                        } else if new_len > arr.elements.len() {
                            arr.elements.resize(new_len, Value::Undefined);
                        }
                    } else if let Some(index) = parse_array_index(&key_str) {
                        if index >= arr.elements.len() {
                            arr.elements.resize(index + 1, Value::Undefined);
                        }
                        arr.elements[index] = value;
                    }
                }
            }
            Value::Function(func_idx) => {
                if let HeapValue::Function(f) = &mut self.heap[*func_idx] {
                    f.properties.remove(&format!("__getter_{}", key_str));
                    f.properties.remove(&format!("__setter_{}", key_str));
                    // Keep `f.prototype` in sync with the `prototype` property
                    // so that `util.inherits`/classical inheritance (which sets
                    // `ctor.prototype` via set_property) is reflected when the
                    // prototype is later read back. Otherwise `get_property`
                    // would lazily fabricate a fresh empty prototype and break
                    // the chain (`this.on` undefined inside subclasses).
                    if key_str == wk::PROTOTYPE {
                        if let Value::Object(pidx) = value {
                            f.prototype = Some(pidx);
                        } else if matches!(value, Value::Null) {
                            f.prototype = None;
                        }
                    }
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
            Value::NativeFunction(idx) if *idx == c::ERROR_CONSTRUCTOR => match key_str.as_str() {
                "stackTraceLimit" => {
                    self.error_stack_trace_limit = match &value {
                        Value::Integer(n) => *n,
                        Value::Float(n) => *n as i64,
                        _ => 10,
                    };
                }
                "prepareStackTrace" => {
                    self.error_prepare_stack_trace = if matches!(value, Value::Undefined) {
                        None
                    } else {
                        Some(value)
                    };
                }
                _ => {}
            },
            _ => {}
        }
        Ok(())
    }

    pub(crate) fn get_property(&mut self, object: &Value, key: &Value) -> Result<Value> {
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
                // Raw `HeapValue::Iterator` objects (from exec_get_iterator for
                // Map/Set/etc.) have no property map.  Without this guard the
                // slow‑path SpreadArray iterator protocol returns Undefined for
                // `.next` and throws "undefined is not a function".
                if let HeapValue::Iterator(_) = &self.heap[*obj_idx] {
                    let key_str = match key_to_str(key) {
                        Some(s) => s,
                        None => return Ok(Value::Undefined),
                    };
                    if key_str == wk::NEXT {
                        return Ok(Value::NativeFunction(c::ITERATOR_NEXT));
                    }
                    if key_str == wk::RETURN || key_str == "throw" {
                        return Ok(Value::NativeFunction(c::ITERATOR_NEXT));
                    }
                    return Ok(Value::Undefined);
                }

                // `constructor` resolves through the prototype chain to a native
                // constructor when the prototype is a built-in proto (streams,
                // errors, buffers, typed arrays, arrays, dates, regexps,
                // generators). This mirrors Node's behavior where
                // `stream.constructor.prototype.write` is the native method.
                let obj_proto = if let HeapValue::Object(obj) = &self.heap[*obj_idx] {
                    obj.prototype
                } else {
                    None
                };
                if let Value::String(ks) = key {
                    if **ks == *wk::CONSTRUCTOR {
                        if let Some(ctor) = self.constructor_for_proto_chain(obj_proto) {
                            return Ok(ctor);
                        }
                    }
                }
                if let HeapValue::Object(obj) = &mut self.heap[*obj_idx] {
                    let key_str = match key_to_str(key) {
                        Some(s) => s,
                        None => return Ok(Value::Undefined),
                    };
                    // Live binding for module namespaces: if tagged with
                    // __module_path, read from the registry so circular
                    // dependency imports see exports defined so far. Fall back
                    // to the object's own properties when the registry entry is
                    // missing or doesn't contain the key — otherwise objects
                    // that merely carry a `__module_path` tag (e.g. the default
                    // export of a native module, which is built via
                    // build_module_object_from_exports) would lose all their
                    // own properties.
                    if key_str != "__module_path" {
                        if let Some(Value::String(module_path)) =
                            obj.properties.get("__module_path")
                        {
                            if let Some(exports) = self.module_registry.get(module_path.as_ref()) {
                                if let Some(val) = exports.get(&key_str) {
                                    return Ok(val.clone());
                                }
                            }
                        }
                    }
                    // Phase 8.2: Use get_cached() to update inline cache
                    if let Some(val) = obj.properties.get_cached(&key_str) {
                        return Ok(val.clone());
                    }
                    if obj.properties.has_accessors() {
                        if let Some(getter_val) =
                            find_accessor(&obj.properties, "__getter_", &key_str)
                        {
                            return self.call_value(&getter_val, this, &[]);
                        }
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
                            if **key_str == *wk::LENGTH {
                                return Ok(Value::Float(arr.elements.len() as f64));
                            }
                            if let Ok(index) = key_str.parse::<usize>() {
                                return Ok(arr
                                    .elements
                                    .get(index)
                                    .cloned()
                                    .unwrap_or(Value::Undefined));
                            }
                            let method = self.get_array_method(key_str)?;
                            if !matches!(method, Value::Undefined) {
                                return Ok(method);
                            }
                            // Fall back to the Array.prototype → Object.prototype
                            // chain (e.g. `hasOwnProperty`, `toString`).
                            return self.get_from_prototype_chain(
                                self.array_proto_idx,
                                key,
                                object,
                            );
                        }
                        Value::Cons(c) => {
                            let s = c.flatten();
                            if s == wk::LENGTH {
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
                if key_str == wk::PROTOTYPE {
                    // Prefer a `prototype` set via set_property (used by
                    // `util.inherits`/classical inheritance) over the lazily
                    // allocated `f.prototype` field, so the chain that was
                    // actually installed is the one returned.
                    if let HeapValue::Function(f) = &self.heap[*func_idx] {
                        if let Some(val) = f.properties.get(&key_str) {
                            return Ok(val.clone());
                        }
                        if let Some(proto_idx) = f.prototype {
                            return Ok(Value::Object(proto_idx));
                        }
                    }
                    // Lazy allocation: prototype is None, allocate on demand
                    let proto_obj_idx = self
                        .gc
                        .allocate(&mut self.heap, HeapValue::Object(JsObject::new()));
                    if let HeapValue::Function(f) = &mut self.heap[*func_idx] {
                        f.prototype = Some(proto_obj_idx);
                    }
                    return Ok(Value::Object(proto_obj_idx));
                }
                if let HeapValue::Function(f) = &self.heap[*func_idx] {
                    if let Some(val) = f.properties.get(&key_str) {
                        return Ok(val.clone());
                    }
                    if f.properties.has_accessors() {
                        if let Some(getter_val) =
                            find_accessor(&f.properties, "__getter_", &key_str)
                        {
                            return self.call_value(&getter_val, this, &[]);
                        }
                    }
                    // Walk [[Prototype]] set via Object.setPrototypeOf.
                    if let Some(proto) = f.properties.get("__[[Prototype]]__").cloned() {
                        return self.get_property_with_this(&proto, key, this);
                    }
                    // Default: Function.prototype
                    if let Some(proto_idx) = self.function_proto_idx {
                        let proto_val = Value::Object(proto_idx);
                        return self.get_property_with_this(&proto_val, key, this);
                    }
                }
            }
            Value::Promise(promise_idx) => {
                let key_str = match key_to_str(key) {
                    Some(s) => s,
                    None => return Ok(Value::Undefined),
                };
                match key_str.as_str() {
                    wk::THEN => return Ok(Value::NativeFunction(c::PROMISE_THEN)),
                    wk::CATCH => return Ok(Value::NativeFunction(c::PROMISE_CATCH)),
                    wk::FINALLY => return Ok(Value::NativeFunction(c::PROMISE_FINALLY)),
                    "state" => {
                        if let HeapValue::Promise(p) = &self.heap[*promise_idx] {
                            return Ok(Value::from_string(format!("{:?}", p.state)));
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
                if *idx == c::ERROR_CONSTRUCTOR {
                    match key_str.as_str() {
                        "captureStackTrace" => {
                            return Ok(Value::NativeFunction(c::ERROR_CAPTURE_STACK_TRACE))
                        }
                        "stackTraceLimit" => {
                            return Ok(Value::Integer(self.error_stack_trace_limit))
                        }
                        "prepareStackTrace" => {
                            return Ok(self
                                .error_prepare_stack_trace
                                .clone()
                                .unwrap_or(Value::Undefined))
                        }
                        "prototype" => {
                            if let Some(proto_idx) = self.error_proto_idx {
                                return Ok(Value::Object(proto_idx));
                            }
                        }
                        _ => {}
                    }
                }
                // Subclass constructors also expose `.prototype`
                if matches!(
                    *idx,
                    c::TYPE_ERROR_CONSTRUCTOR
                        | c::REFERENCE_ERROR_CONSTRUCTOR
                        | c::SYNTAX_ERROR_CONSTRUCTOR
                        | c::RANGE_ERROR_CONSTRUCTOR
                ) && key_str == wk::PROTOTYPE
                {
                    if let Some(proto_idx) = self.find_native_prototype(*idx) {
                        return Ok(Value::Object(proto_idx));
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
                        s if s == wk::PROTOTYPE => {
                            if let Some(proto_idx) = self.symbol_proto_idx {
                                return Ok(Value::Object(proto_idx));
                            }
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
                        s if s == wk::PROTOTYPE => {
                            // Array.prototype is a real array instance methods map.
                            // Build a shared prototype object once if needed.
                            if let Some(proto_idx) = self.array_proto_idx {
                                return Ok(Value::Object(proto_idx));
                            }
                        }
                        _ => {}
                    }
                }
                if *idx == c::ARRAY_BUFFER_CONSTRUCTOR {
                    match key_str.as_str() {
                        "isView" => return Ok(Value::NativeFunction(c::ARRAY_BUFFER_IS_VIEW)),
                        "prototype" => return Ok(Value::Undefined),
                        _ => {}
                    }
                }
                if *idx == c::ASSERT {
                    match key_str.as_str() {
                        "ok" => return Ok(Value::NativeFunction(c::ASSERT)),
                        "strictEqual" | "equal" => {
                            return Ok(Value::NativeFunction(c::ASSERT_STRICT_EQUAL))
                        }
                        "notStrictEqual" | "notEqual" => {
                            return Ok(Value::NativeFunction(c::ASSERT_NOT_STRICT_EQUAL))
                        }
                        "deepStrictEqual" | "deepEqual" => {
                            return Ok(Value::NativeFunction(c::ASSERT_DEEP_EQUAL))
                        }
                        "notDeepStrictEqual" | "notDeepEqual" => {
                            return Ok(Value::NativeFunction(c::ASSERT_NOT_DEEP_STRICT_EQUAL))
                        }
                        "ifError" => return Ok(Value::NativeFunction(c::ASSERT_IF_ERROR)),
                        "fail" => return Ok(Value::NativeFunction(c::ASSERT_FAIL)),
                        "throws" => return Ok(Value::NativeFunction(c::ASSERT_THROWS)),
                        "doesNotThrow" => {
                            return Ok(Value::NativeFunction(c::ASSERT_DOES_NOT_THROW))
                        }
                        "rejects" => return Ok(Value::NativeFunction(c::ASSERT_REJECTS)),
                        "doesNotReject" => {
                            return Ok(Value::NativeFunction(c::ASSERT_DOES_NOT_REJECT))
                        }
                        "match" => return Ok(Value::NativeFunction(c::ASSERT_MATCH)),
                        "doesNotMatch" => return Ok(Value::NativeFunction(c::ASSERT_NOT_MATCH)),
                        "partialDeepStrictEqual" => {
                            return Ok(Value::NativeFunction(c::ASSERT_DEEP_EQUAL))
                        }
                        "assert" => return Ok(Value::NativeFunction(c::ASSERT)),
                        "prototype" => return Ok(Value::Undefined),
                        _ => {}
                    }
                }
                // stream.Readable / Writable / Transform / PassThrough.prototype
                if (*idx == c::STREAM_CONSTRUCTOR || *idx == c::STREAM_PASSTHROUGH_CONSTRUCTOR)
                    && key_str == wk::PROTOTYPE
                {
                    if let Some(proto_idx) = self.find_native_prototype(*idx) {
                        return Ok(Value::Object(proto_idx));
                    }
                }
                // TypedArray constructors are bare NativeFunctions; expose
                // statics (from/of/BYTES_PER_ELEMENT/prototype) here.
                if matches!(
                    *idx,
                    c::INT8_ARRAY_CONSTRUCTOR
                        | c::UINT8_ARRAY_CONSTRUCTOR
                        | c::UINT8_CLAMPED_ARRAY_CONSTRUCTOR
                        | c::INT16_ARRAY_CONSTRUCTOR
                        | c::UINT16_ARRAY_CONSTRUCTOR
                        | c::INT32_ARRAY_CONSTRUCTOR
                        | c::UINT32_ARRAY_CONSTRUCTOR
                        | c::FLOAT32_ARRAY_CONSTRUCTOR
                        | c::FLOAT64_ARRAY_CONSTRUCTOR
                        | c::BIGINT64_ARRAY_CONSTRUCTOR
                        | c::BIGUINT64_ARRAY_CONSTRUCTOR
                ) {
                    match key_str.as_str() {
                        "from" => return Ok(Value::NativeFunction(c::TYPED_ARRAY_FROM)),
                        "of" => return Ok(Value::NativeFunction(c::TYPED_ARRAY_OF)),
                        "BYTES_PER_ELEMENT" => {
                            let bpe = match *idx {
                                c::INT8_ARRAY_CONSTRUCTOR
                                | c::UINT8_ARRAY_CONSTRUCTOR
                                | c::UINT8_CLAMPED_ARRAY_CONSTRUCTOR => 1,
                                c::INT16_ARRAY_CONSTRUCTOR | c::UINT16_ARRAY_CONSTRUCTOR => 2,
                                c::INT32_ARRAY_CONSTRUCTOR
                                | c::UINT32_ARRAY_CONSTRUCTOR
                                | c::FLOAT32_ARRAY_CONSTRUCTOR => 4,
                                c::FLOAT64_ARRAY_CONSTRUCTOR
                                | c::BIGINT64_ARRAY_CONSTRUCTOR
                                | c::BIGUINT64_ARRAY_CONSTRUCTOR => 8,
                                _ => 1,
                            };
                            return Ok(Value::Integer(bpe));
                        }
                        s if s == wk::PROTOTYPE => {
                            if let Some(proto_idx) = self.find_native_prototype(*idx) {
                                return Ok(Value::Object(proto_idx));
                            }
                        }
                        _ => {}
                    }
                }
                if *idx == c::NUMBER_CONSTRUCTOR {
                    match key_str.as_str() {
                        "isFinite" => return Ok(Value::NativeFunction(c::IS_FINITE)),
                        "isNaN" => return Ok(Value::NativeFunction(c::IS_NAN)),
                        "parseFloat" => return Ok(Value::NativeFunction(c::NUMBER_PARSE_FLOAT)),
                        "parseInt" => return Ok(Value::NativeFunction(c::NUMBER_PARSE_INT)),
                        "isInteger" => return Ok(Value::NativeFunction(c::NUMBER_IS_INTEGER)),
                        "isSafeInteger" => {
                            return Ok(Value::NativeFunction(c::NUMBER_IS_SAFE_INTEGER))
                        }
                        s if s == wk::PROTOTYPE => {
                            if let Some(proto_idx) = self.number_proto_idx {
                                return Ok(Value::Object(proto_idx));
                            }
                        }
                        // ES Number static constants
                        "MAX_SAFE_INTEGER" => {
                            return Ok(Value::Float(9007199254740991.0));
                        }
                        "MIN_SAFE_INTEGER" => {
                            return Ok(Value::Float(-9007199254740991.0));
                        }
                        "MAX_VALUE" => return Ok(Value::Float(f64::MAX)),
                        "MIN_VALUE" => return Ok(Value::Float(f64::MIN_POSITIVE)),
                        "POSITIVE_INFINITY" => return Ok(Value::Float(f64::INFINITY)),
                        "NEGATIVE_INFINITY" => return Ok(Value::Float(f64::NEG_INFINITY)),
                        wk::NAN => return Ok(Value::Float(f64::NAN)),
                        "EPSILON" => return Ok(Value::Float(f64::EPSILON)),
                        _ => {}
                    }
                }
                if *idx == c::BOOLEAN_CONSTRUCTOR && key_str == wk::PROTOTYPE {
                    if let Some(proto_idx) = self.boolean_proto_idx {
                        return Ok(Value::Object(proto_idx));
                    }
                }
                if *idx == c::STRING_CONSTRUCTOR && key_str == wk::PROTOTYPE {
                    if let Some(proto_idx) = self.string_proto_idx {
                        return Ok(Value::Object(proto_idx));
                    }
                }
                if *idx == c::FUNCTION_CONSTRUCTOR && key_str == wk::PROTOTYPE {
                    if let Some(proto_idx) = self.function_proto_idx {
                        return Ok(Value::Object(proto_idx));
                    }
                }
                if *idx == c::REGEXP_CONSTRUCTOR && key_str == wk::PROTOTYPE {
                    if let Some(proto_idx) = self.regexp_proto_idx {
                        return Ok(Value::Object(proto_idx));
                    }
                }
                if *idx == c::DATE_CONSTRUCTOR && key_str == wk::PROTOTYPE {
                    if let Some(proto_idx) = self.date_proto_idx {
                        return Ok(Value::Object(proto_idx));
                    }
                }
                if *idx == c::BIGINT_CONSTRUCTOR && key_str == wk::PROTOTYPE {
                    if let Some(proto_idx) = self.bigint_proto_idx {
                        return Ok(Value::Object(proto_idx));
                    }
                }
                if *idx == c::EVENT_EMITTER_CONSTRUCTOR && key_str == wk::PROTOTYPE {
                    if let Some(proto_idx) = self.find_native_prototype(*idx) {
                        return Ok(Value::Object(proto_idx));
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
                    let trap = self.get_property(&handler, &Value::string(wk::GET));
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
                if key_str == wk::LENGTH {
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
                if let Value::Symbol(sym_id) = key {
                    if *sym_id == crate::objects::SYMBOL_ITERATOR {
                        // Map.prototype[Symbol.iterator] === entries
                        return Ok(Value::NativeFunction(c::MAP_ENTRIES));
                    }
                }
                let key_str = match key_to_str(key) {
                    Some(s) => s,
                    None => return Ok(Value::Undefined),
                };
                match key_str.as_str() {
                    s if s == wk::SIZE => {
                        if let Value::Map(map_idx) = this {
                            if let HeapValue::Map(map) = &self.heap[*map_idx] {
                                return Ok(Value::Float(map.size() as f64));
                            }
                        }
                    }
                    s if s == wk::GET => return Ok(Value::NativeFunction(c::MAP_GET)),
                    s if s == wk::SET_PROP => return Ok(Value::NativeFunction(c::MAP_SET)),
                    s if s == wk::HAS => return Ok(Value::NativeFunction(c::MAP_HAS)),
                    s if s == wk::DELETE => return Ok(Value::NativeFunction(c::MAP_DELETE)),
                    s if s == wk::CLEAR => return Ok(Value::NativeFunction(c::MAP_CLEAR)),
                    s if s == wk::FOR_EACH => return Ok(Value::NativeFunction(c::MAP_FOR_EACH)),
                    s if s == wk::KEYS => return Ok(Value::NativeFunction(c::MAP_KEYS)),
                    s if s == wk::VALUES => return Ok(Value::NativeFunction(c::MAP_VALUES)),
                    s if s == wk::ENTRIES => return Ok(Value::NativeFunction(c::MAP_ENTRIES)),
                    _ => {}
                }
            }
            Value::Set(_set_idx) => {
                if let Value::Symbol(sym_id) = key {
                    if *sym_id == crate::objects::SYMBOL_ITERATOR {
                        // Set.prototype[Symbol.iterator] === values
                        return Ok(Value::NativeFunction(c::SET_VALUES));
                    }
                }
                let key_str = match key_to_str(key) {
                    Some(s) => s,
                    None => return Ok(Value::Undefined),
                };
                match key_str.as_str() {
                    s if s == wk::SIZE => {
                        if let Value::Set(set_idx) = this {
                            if let HeapValue::Set(set) = &self.heap[*set_idx] {
                                return Ok(Value::Float(set.size() as f64));
                            }
                        }
                    }
                    s if s == wk::ADD => return Ok(Value::NativeFunction(c::SET_ADD)),
                    s if s == wk::HAS => return Ok(Value::NativeFunction(c::SET_HAS)),
                    s if s == wk::DELETE => return Ok(Value::NativeFunction(c::SET_DELETE)),
                    s if s == wk::CLEAR => return Ok(Value::NativeFunction(c::SET_CLEAR)),
                    s if s == wk::FOR_EACH => return Ok(Value::NativeFunction(c::SET_FOR_EACH)),
                    s if s == wk::VALUES => return Ok(Value::NativeFunction(c::SET_VALUES)),
                    s if s == wk::KEYS => return Ok(Value::NativeFunction(c::SET_KEYS)),
                    s if s == wk::ENTRIES => return Ok(Value::NativeFunction(c::SET_ENTRIES)),
                    _ => {}
                }
            }
            Value::WeakMap(_weakmap_idx) => {
                let key_str = match key_to_str(key) {
                    Some(s) => s,
                    None => return Ok(Value::Undefined),
                };
                match key_str.as_str() {
                    s if s == wk::GET => return Ok(Value::NativeFunction(c::WEAKMAP_GET)),
                    s if s == wk::SET_PROP => return Ok(Value::NativeFunction(c::WEAKMAP_SET)),
                    s if s == wk::HAS => return Ok(Value::NativeFunction(c::WEAKMAP_HAS)),
                    s if s == wk::DELETE => return Ok(Value::NativeFunction(c::WEAKMAP_DELETE)),
                    _ => {}
                }
            }
            Value::WeakSet(_weakset_idx) => {
                let key_str = match key_to_str(key) {
                    Some(s) => s,
                    None => return Ok(Value::Undefined),
                };
                match key_str.as_str() {
                    s if s == wk::ADD => return Ok(Value::NativeFunction(c::WEAKSET_ADD)),
                    s if s == wk::HAS => return Ok(Value::NativeFunction(c::WEAKSET_HAS)),
                    s if s == wk::DELETE => return Ok(Value::NativeFunction(c::WEAKSET_DELETE)),
                    _ => {}
                }
            }
            Value::TypedArray(_ta_idx) => {
                let key_str = match key_to_str(key) {
                    Some(s) => s,
                    None => return Ok(Value::Undefined),
                };
                match key_str.as_str() {
                    wk::LENGTH => {
                        if let Value::TypedArray(ta_idx) = this {
                            if let HeapValue::TypedArray(ta) = &self.heap[*ta_idx] {
                                let elem_size =
                                    crate::objects::js_array::TypedArray::element_size(&ta.kind);
                                return Ok(Value::Float((ta.byte_length / elem_size) as f64));
                            }
                        }
                    }
                    s if s == wk::BYTE_LENGTH => {
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
                    "buffer" => return Ok(Value::NativeFunction(c::TYPED_ARRAY_BUFFER)),
                    _ => {}
                }
                // Fall back to the TypedArray.prototype chain
                // (e.g. `[Symbol.toStringTag]` getter, BYTES_PER_ELEMENT).
                return self.get_from_prototype_chain(self.typed_array_proto_idx, key, this);
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

    /// Walk the prototype chain starting at `proto_idx` and return the first
    /// constructor NativeFunction found, so `instance.constructor` resolves for
    /// built-in objects whose prototype is a native proto.
    fn constructor_for_proto_chain(&self, proto_idx: Option<usize>) -> Option<Value> {
        let mut current = proto_idx?;
        for _ in 0..64 {
            if let Some(ctor) = self.constructor_for_proto(current) {
                return Some(ctor);
            }
            match &self.heap[current] {
                HeapValue::Object(obj) => {
                    if let Some(next) = obj.prototype {
                        current = next;
                    } else {
                        return None;
                    }
                }
                _ => return None,
            }
        }
        None
    }

    /// Resolve `key` by walking the prototype chain starting at `proto_idx`.
    /// Used by array/primitive property lookups to inherit Object.prototype
    /// members (e.g. `hasOwnProperty`, `toString`) that live on the shared
    /// prototype rather than the value's own property map.
    pub(super) fn get_from_prototype_chain(
        &self,
        proto_idx: Option<usize>,
        key: &Value,
        _this: &Value,
    ) -> Result<Value> {
        let mut current = match proto_idx {
            Some(idx) => Value::Object(idx),
            None => return Ok(Value::Undefined),
        };
        // `constructor` resolves to the first native constructor found while
        // walking the prototype chain (e.g. TypedArray/stream instances).
        if let Some(ks) = key_to_str(key) {
            if ks == wk::CONSTRUCTOR {
                if let Some(ctor) = self.constructor_for_proto_chain(proto_idx) {
                    return Ok(ctor);
                }
            }
        }
        // Guard against prototype loops.
        for _ in 0..64 {
            match &current {
                Value::Object(idx) => {
                    if let HeapValue::Object(obj) = &self.heap[*idx] {
                        let ks = match key_to_str(key) {
                            Some(s) => s,
                            None => return Ok(Value::Undefined),
                        };
                        if let Some(val) = obj.properties.get(&ks) {
                            return Ok(val.clone());
                        }
                        if let Some(next) = obj.prototype {
                            current = Value::Object(next);
                        } else {
                            return Ok(Value::Undefined);
                        }
                    } else {
                        return Ok(Value::Undefined);
                    }
                }
                _ => return Ok(Value::Undefined),
            }
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
            "at" => 469,
            _ => return Ok(Value::Undefined),
        };
        Ok(Value::NativeFunction(idx))
    }

    pub(super) fn get_property_from_primitive_string(&self, s: &str, key: &Value) -> Result<Value> {
        if let Value::Symbol(sym_id) = key {
            if *sym_id == crate::objects::SYMBOL_ITERATOR {
                // String.prototype[Symbol.iterator] — reuse array iterator after split.
                return Ok(Value::NativeFunction(c::STRING_ITERATOR));
            }
        }
        let key_str = match key_to_str(key) {
            Some(s) => s,
            None => return Ok(Value::Undefined),
        };
        if key_str == wk::LENGTH {
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
            "trimStart" | "trimLeft" => c::STRING_TRIM_START,
            "trimEnd" | "trimRight" => c::STRING_TRIM_END,
            "toLowerCase" => 61,
            "toUpperCase" => 62,
            "startsWith" => 63,
            "endsWith" => 64,
            "repeat" => 65,
            "padStart" => 66,
            "padEnd" => 67,
            "match" => 227,
            "search" => 229,
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
            wk::TO_STRING
            | "toFixed"
            | wk::VALUE_OF
            | "toExponential"
            | "toPrecision"
            | wk::TO_LOCALE_STRING => {
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
            wk::TO_STRING | wk::VALUE_OF => {
                return Ok(self.make_native_boolean_method(&key_str));
            }
            _ => {}
        }
        Ok(Value::Undefined)
    }

    pub(super) fn make_native_number_method(&self, name: &str) -> Value {
        match name {
            "toFixed" => Value::NativeFunction(c::NUMBER_TO_FIXED),
            wk::TO_STRING | wk::TO_LOCALE_STRING => Value::NativeFunction(c::NUMBER_TO_STRING),
            wk::VALUE_OF => Value::NativeFunction(c::NUMBER_VALUE_OF),
            "toExponential" => Value::NativeFunction(c::NUMBER_TO_EXPONENTIAL),
            "toPrecision" => Value::NativeFunction(c::NUMBER_TO_PRECISION),
            _ => Value::Undefined,
        }
    }

    pub(super) fn make_native_boolean_method(&self, name: &str) -> Value {
        match name {
            wk::TO_STRING | wk::TO_LOCALE_STRING => Value::NativeFunction(c::BOOLEAN_TO_STRING),
            wk::VALUE_OF => Value::NativeFunction(c::BOOLEAN_VALUE_OF),
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
        let proto_key = Value::from_string(wk::PROTOTYPE.to_string());
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
                    if obj.properties.has_accessors()
                        && (find_accessor(&obj.properties, "__getter_", &key_str).is_some()
                            || find_accessor(&obj.properties, "__setter_", &key_str).is_some())
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
                    if key_str == wk::LENGTH {
                        return Ok(Value::Boolean(true));
                    }
                    if let Ok(index) = key_str.parse::<usize>() {
                        return Ok(Value::Boolean(index < arr.elements.len()));
                    }
                }
                Ok(Value::Boolean(false))
            }
            Value::String(s) => {
                if key_str == wk::LENGTH {
                    return Ok(Value::Boolean(true));
                }
                if let Ok(index) = key_str.parse::<usize>() {
                    return Ok(Value::Boolean(index < s.len()));
                }
                Ok(Value::Boolean(false))
            }
            Value::Function(func_idx) => {
                if let HeapValue::Function(f) = &self.heap[*func_idx] {
                    if f.properties.contains_key(&key_str) {
                        return Ok(Value::Boolean(true));
                    }
                    if f.properties.has_accessors()
                        && (find_accessor(&f.properties, "__getter_", &key_str).is_some()
                            || find_accessor(&f.properties, "__setter_", &key_str).is_some())
                    {
                        return Ok(Value::Boolean(true));
                    }
                    // Walk Function.prototype / custom [[Prototype]] if present.
                    if let Some(proto) = f.properties.get("__[[Prototype]]__").cloned() {
                        return self.in_check_mut(key, &proto);
                    }
                    if let Some(proto_idx) = self.function_proto_idx {
                        return self.in_check_mut(key, &Value::Object(proto_idx));
                    }
                }
                Ok(Value::Boolean(false))
            }
            Value::Proxy(proxy_idx) => {
                if let HeapValue::Proxy(proxy) = &self.heap[*proxy_idx] {
                    let handler = proxy.handler.clone();
                    let target = proxy.target.clone();
                    let trap = self.get_property(&handler, &Value::string(wk::HAS));
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
        let trap = self.get_property(handler, &Value::from_string(trap_name.to_string()))?;
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

/// Parse a canonical array index string ("0", "1", …). Rejects leading zeros
/// like "01" so they remain ordinary property keys per the ES spec.
pub(crate) fn parse_array_index(key: &str) -> Option<usize> {
    if key.is_empty() {
        return None;
    }
    // Reject leading zeros except for "0" itself.
    if key.len() > 1 && key.starts_with('0') {
        return None;
    }
    let index: usize = key.parse().ok()?;
    // Array indices are uint32 values < 2^32 - 1.
    if index as u64 >= 0xFFFF_FFFF {
        return None;
    }
    // Ensure the string is the canonical decimal representation.
    if index.to_string() != key {
        return None;
    }
    Some(index)
}
