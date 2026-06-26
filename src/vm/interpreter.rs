use std::collections::HashMap;
use crate::compiler::{CompiledModule, Instruction};
use crate::errors::{Error, Result};
use crate::objects::Value;
use crate::runtime_env::native_fns::NATIVE_TABLE;
use crate::runtime_env::async_runtime::AsyncRuntime;
use crate::objects::js_promise::{JsPromise, PromiseState};

#[derive(Debug, Clone)]
pub struct JsObject {
    pub properties: HashMap<String, Value>,
    pub prototype: Option<usize>,
}

impl JsObject {
    pub fn new() -> Self {
        Self {
            properties: HashMap::new(),
            prototype: None,
        }
    }

    pub fn with_prototype(prototype: Option<usize>) -> Self {
        Self {
            properties: HashMap::new(),
            prototype,
        }
    }
}

#[derive(Debug, Clone)]
pub struct JsArray {
    pub elements: Vec<Value>,
}

#[derive(Debug, Clone)]
pub struct JsFunction {
    pub name: Option<String>,
    pub params: Vec<String>,
    pub bytecode_index: usize,
    pub closure: Vec<Value>,
    pub prototype: Option<usize>,
    pub super_class: Option<Value>,
    pub properties: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
pub enum HeapValue {
    String(String),
    Object(JsObject),
    Array(JsArray),
    Function(JsFunction),
    Promise(JsPromise),
}

#[derive(Debug, Clone)]
struct CallFrame {
    return_address: usize,
    base_pointer: usize,
    closure_var_count: usize,
    func_heap_idx: Option<usize>,
    this_value: Option<Value>,
    is_construct: bool,
}

#[derive(Debug, Clone)]
struct ExceptionHandler {
    catch_pc: u32,
    finally_pc: u32,
    stack_depth: usize,
}

pub struct Interpreter {
    pub(crate) globals: HashMap<String, Value>,
    pub(crate) stack: Vec<Value>,
    pub(crate) heap: Vec<HeapValue>,
    call_stack: Vec<CallFrame>,
    current_module: Option<CompiledModule>,
    exception_handlers: Vec<ExceptionHandler>,
    pending_exception: Option<Value>,
    pub(crate) async_runtime: AsyncRuntime,
    pub(crate) promise_stack: Vec<usize>,
    timer_id_counter: u32,
}

impl Interpreter {
    pub fn new() -> Result<Self> {
        let mut interp = Self {
            globals: HashMap::new(),
            stack: Vec::new(),
            heap: Vec::new(),
            call_stack: Vec::new(),
            current_module: None,
            exception_handlers: Vec::new(),
            pending_exception: None,
            async_runtime: AsyncRuntime::new(),
            promise_stack: Vec::new(),
            timer_id_counter: 1,
        };
        interp.init_builtins();
        Ok(interp)
    }

    fn init_builtins(&mut self) {
        // Global functions
        self.globals.insert("parseInt".into(), Value::NativeFunction(10));
        self.globals.insert("parseFloat".into(), Value::NativeFunction(11));
        self.globals.insert("isNaN".into(), Value::NativeFunction(12));
        self.globals.insert("isFinite".into(), Value::NativeFunction(13));

        // Timer stubs
        self.globals.insert("setTimeout".into(), Value::NativeFunction(14));
        self.globals.insert("setInterval".into(), Value::NativeFunction(15));
        self.globals.insert("clearTimeout".into(), Value::NativeFunction(16));
        self.globals.insert("clearInterval".into(), Value::NativeFunction(17));

        // console object
        let console_obj_idx = self.heap.len();
        let mut console_props = HashMap::new();
        console_props.insert("log".into(), Value::NativeFunction(0));
        console_props.insert("warn".into(), Value::NativeFunction(1));
        console_props.insert("error".into(), Value::NativeFunction(2));
        console_props.insert("info".into(), Value::NativeFunction(3));
        self.heap.push(HeapValue::Object(JsObject { properties: console_props, prototype: None }));
        self.globals.insert("console".into(), Value::Object(console_obj_idx));

        // Object
        let object_obj_idx = self.heap.len();
        let mut object_props = HashMap::new();
        object_props.insert("keys".into(), Value::NativeFunction(4));
        object_props.insert("values".into(), Value::NativeFunction(5));
        object_props.insert("entries".into(), Value::NativeFunction(6));
        object_props.insert("assign".into(), Value::NativeFunction(7));
        self.heap.push(HeapValue::Object(JsObject { properties: object_props, prototype: None }));
        self.globals.insert("Object".into(), Value::Object(object_obj_idx));

        // JSON
        let json_obj_idx = self.heap.len();
        let mut json_props = HashMap::new();
        json_props.insert("parse".into(), Value::NativeFunction(8));
        json_props.insert("stringify".into(), Value::NativeFunction(9));
        self.heap.push(HeapValue::Object(JsObject { properties: json_props, prototype: None }));
        self.globals.insert("JSON".into(), Value::Object(json_obj_idx));

        // Math
        let math_obj_idx = self.heap.len();
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
        self.heap.push(HeapValue::Object(JsObject { properties: math_props, prototype: None }));
        self.globals.insert("Math".into(), Value::Object(math_obj_idx));

        // Number constructor
        let number_obj_idx = self.heap.len();
        let mut number_props = HashMap::new();
        number_props.insert("isFinite".into(), Value::NativeFunction(13));
        number_props.insert("isNaN".into(), Value::NativeFunction(12));
        number_props.insert("parseFloat".into(), Value::NativeFunction(11));
        number_props.insert("parseInt".into(), Value::NativeFunction(10));
        self.heap.push(HeapValue::Object(JsObject { properties: number_props, prototype: None }));
        self.globals.insert("Number".into(), Value::Object(number_obj_idx));

        // Promise constructor and prototype
        let promise_proto_idx = self.heap.len();
        let mut promise_proto_props = HashMap::new();
        promise_proto_props.insert("then".into(), Value::NativeFunction(78));
        promise_proto_props.insert("catch".into(), Value::NativeFunction(79));
        promise_proto_props.insert("finally".into(), Value::NativeFunction(80));
        self.heap.push(HeapValue::Object(JsObject { properties: promise_proto_props, prototype: None }));

        let _promise_ctor_idx = self.heap.len();
        let mut promise_ctor_props = HashMap::new();
        promise_ctor_props.insert("prototype".into(), Value::Object(promise_proto_idx));
        promise_ctor_props.insert("resolve".into(), Value::NativeFunction(81));
        promise_ctor_props.insert("reject".into(), Value::NativeFunction(82));
        promise_ctor_props.insert("all".into(), Value::NativeFunction(83));
        promise_ctor_props.insert("race".into(), Value::NativeFunction(84));
        self.heap.push(HeapValue::Object(JsObject { properties: promise_ctor_props, prototype: None }));
        self.globals.insert("Promise".into(), Value::NativeFunction(77));

        // Error constructor
        let error_proto_idx = self.heap.len();
        self.heap.push(HeapValue::Object(JsObject::new()));
        let _error_ctor_idx = self.heap.len();
        let mut error_ctor_props = HashMap::new();
        error_ctor_props.insert("prototype".into(), Value::Object(error_proto_idx));
        self.heap.push(HeapValue::Object(JsObject { properties: error_ctor_props, prototype: None }));
        self.globals.insert("Error".into(), Value::NativeFunction(72));

        // TypeError constructor
        let type_error_proto_idx = self.heap.len();
        let mut type_error_proto_props = HashMap::new();
        type_error_proto_props.insert("name".into(), Value::String("TypeError".into()));
        self.heap.push(HeapValue::Object(JsObject { properties: type_error_proto_props, prototype: Some(error_proto_idx) }));
        let _type_error_ctor_idx = self.heap.len();
        let mut type_error_ctor_props = HashMap::new();
        type_error_ctor_props.insert("prototype".into(), Value::Object(type_error_proto_idx));
        self.heap.push(HeapValue::Object(JsObject { properties: type_error_ctor_props, prototype: None }));
        self.globals.insert("TypeError".into(), Value::NativeFunction(73));

        // ReferenceError constructor
        let ref_error_proto_idx = self.heap.len();
        let mut ref_error_proto_props = HashMap::new();
        ref_error_proto_props.insert("name".into(), Value::String("ReferenceError".into()));
        self.heap.push(HeapValue::Object(JsObject { properties: ref_error_proto_props, prototype: Some(error_proto_idx) }));
        let _ref_error_ctor_idx = self.heap.len();
        let mut ref_error_ctor_props = HashMap::new();
        ref_error_ctor_props.insert("prototype".into(), Value::Object(ref_error_proto_idx));
        self.heap.push(HeapValue::Object(JsObject { properties: ref_error_ctor_props, prototype: None }));
        self.globals.insert("ReferenceError".into(), Value::NativeFunction(74));

        // SyntaxError constructor
        let syntax_error_proto_idx = self.heap.len();
        let mut syntax_error_proto_props = HashMap::new();
        syntax_error_proto_props.insert("name".into(), Value::String("SyntaxError".into()));
        self.heap.push(HeapValue::Object(JsObject { properties: syntax_error_proto_props, prototype: Some(error_proto_idx) }));
        let _syntax_error_ctor_idx = self.heap.len();
        let mut syntax_error_ctor_props = HashMap::new();
        syntax_error_ctor_props.insert("prototype".into(), Value::Object(syntax_error_proto_idx));
        self.heap.push(HeapValue::Object(JsObject { properties: syntax_error_ctor_props, prototype: None }));
        self.globals.insert("SyntaxError".into(), Value::NativeFunction(75));

        // RangeError constructor
        let range_error_proto_idx = self.heap.len();
        let mut range_error_proto_props = HashMap::new();
        range_error_proto_props.insert("name".into(), Value::String("RangeError".into()));
        self.heap.push(HeapValue::Object(JsObject { properties: range_error_proto_props, prototype: Some(error_proto_idx) }));
        let _range_error_ctor_idx = self.heap.len();
        let mut range_error_ctor_props = HashMap::new();
        range_error_ctor_props.insert("prototype".into(), Value::Object(range_error_proto_idx));
        self.heap.push(HeapValue::Object(JsObject { properties: range_error_ctor_props, prototype: None }));
        self.globals.insert("RangeError".into(), Value::NativeFunction(76));
    }

    pub fn execute(&mut self, module: &CompiledModule) -> Result<Value> {
        self.current_module = Some(module.clone());
        let result = self.execute_from(module, 0);
        self.drain_microtasks();
        let macrotasks: Vec<_> = self.async_runtime.run_macrotasks();
        for task in macrotasks {
            let _ = self.call_value(&task.callback, &Value::Undefined, &[]);
        }
        result
    }
    
    fn execute_from(&mut self, module: &CompiledModule, start_pc: usize) -> Result<Value> {
        let mut pc = start_pc;

        'main: loop {
            if pc >= module.instructions.len() {
                break;
            }

            let instruction = module.instructions[pc].clone();
            
            match &instruction {
                Instruction::LoadConst(idx) => {
                    let value = module.constants[*idx as usize].clone();
                    self.stack.push(value);
                }
                Instruction::LoadNull => {
                    self.stack.push(Value::Null);
                }
                Instruction::LoadUndefined => {
                    self.stack.push(Value::Undefined);
                }
                Instruction::LoadTrue => {
                    self.stack.push(Value::Boolean(true));
                }
                Instruction::LoadFalse => {
                    self.stack.push(Value::Boolean(false));
                }
                Instruction::LoadGlobal(name) => {
                    let value = self.globals.get(name)
                        .cloned()
                        .unwrap_or(Value::Undefined);
                    self.stack.push(value);
                }
                Instruction::StoreGlobal(name) => {
                    let value = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    self.globals.insert(name.clone(), value);
                }
                Instruction::LoadLocal(slot) => {
                    let base = self.call_stack.last()
                        .map(|f| f.base_pointer)
                        .unwrap_or(0);
                    let idx = base + *slot as usize;
                    let value = self.stack.get(idx)
                        .cloned()
                        .unwrap_or(Value::Undefined);
                    if self.call_stack.len() >= 1 {
                        eprintln!("  LoadLocal(slot={} base={} idx={} stack_len={}) → {:?}", slot, base, idx, self.stack.len(), value);
                    }
                    self.stack.push(value);
                }
                Instruction::StoreLocal(slot) => {
                    let value = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let base = self.call_stack.last()
                        .map(|f| f.base_pointer)
                        .unwrap_or(0);
                    let idx = base + *slot as usize;

                    if idx >= self.stack.len() {
                        self.stack.resize(idx + 1, Value::Undefined);
                    }
                    self.stack[idx] = value;
                }
                Instruction::Add => {
                    let right = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let left = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let result = self.add(left, right)?;
                    self.stack.push(result);
                }
                Instruction::Sub => {
                    let right = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let left = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let result = self.sub(left, right)?;
                    self.stack.push(result);
                }
                Instruction::Mul => {
                    let right = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let left = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let result = self.mul(left, right)?;
                    self.stack.push(result);
                }
                Instruction::Div => {
                    let right = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let left = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let result = self.div(left, right)?;
                    self.stack.push(result);
                }
                Instruction::Mod => {
                    let right = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let left = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let result = self.modulo(left, right)?;
                    self.stack.push(result);
                }
                Instruction::Power => {
                    let right = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let left = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let result = self.power(left, right)?;
                    self.stack.push(result);
                }
                Instruction::Negate => {
                    let value = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let result = self.negate(value)?;
                    self.stack.push(result);
                }
                Instruction::Not => {
                    let value = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let result = Value::Boolean(!self.is_truthy(&value));
                    self.stack.push(result);
                }
                Instruction::Eq => {
                    let right = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let left = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    self.stack.push(Value::Boolean(self.is_equal(&left, &right)));
                }
                Instruction::StrictEq => {
                    let right = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let left = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    self.stack.push(Value::Boolean(left == right));
                }
                Instruction::NotEqual => {
                    let right = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let left = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    self.stack.push(Value::Boolean(!self.is_equal(&left, &right)));
                }
                Instruction::StrictNotEqual => {
                    let right = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let left = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    self.stack.push(Value::Boolean(left != right));
                }
                Instruction::Less => {
                    let right = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let left = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let result = self.less_than(&left, &right)?;
                    self.stack.push(Value::Boolean(result));
                }
                Instruction::Greater => {
                    let right = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let left = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let result = self.greater_than(&left, &right)?;
                    self.stack.push(Value::Boolean(result));
                }
                Instruction::LessEqual => {
                    let right = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let left = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let result = self.less_than_or_equal(&left, &right)?;
                    self.stack.push(Value::Boolean(result));
                }
                Instruction::GreaterEqual => {
                    let right = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let left = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let result = self.greater_than_or_equal(&left, &right)?;
                    self.stack.push(Value::Boolean(result));
                }
                Instruction::And => {
                    let right = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let left = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let result = self.is_truthy(&left) && self.is_truthy(&right);
                    self.stack.push(Value::Boolean(result));
                }
                Instruction::Or => {
                    let right = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let left = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let result = self.is_truthy(&left) || self.is_truthy(&right);
                    self.stack.push(Value::Boolean(result));
                }
                Instruction::Jump(target) => {
                    pc = *target as usize;
                    continue;
                }
                Instruction::JumpIf(target) => {
                    let value = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    if self.is_truthy(&value) {
                        pc = *target as usize;
                        continue;
                    }
                }
                Instruction::JumpIfNot(target) => {
                    let value = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    if !self.is_truthy(&value) {
                        pc = *target as usize;
                        continue;
                    }
                }
                Instruction::Call(argc) => {
                    let callee = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;

                    let mut args = Vec::new();
                    for _ in 0..*argc {
                        args.push(self.stack.pop()
                            .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?);
                    }
                    args.reverse();

                    if let Value::Function(func_idx) = &callee {
                        if let HeapValue::Function(f) = &self.heap[*func_idx] {
                            if f.bytecode_index == usize::MAX {
                                eprintln!("Call: resolve/reject func idx={} name={:?} closure_len={}", func_idx, f.name, f.closure.len());
                            }
                        }
                    }

                    match callee {
                        Value::Function(func_idx) => {
                            let func = self.heap[func_idx].clone();
                            if let HeapValue::Function(f) = func {
                                if f.bytecode_index == usize::MAX {
                                    if let Some(Value::Promise(promise_idx)) = f.closure.first() {
                                        match f.name.as_deref() {
                                            Some("resolve") => {
                                                let val = args.first().cloned().unwrap_or(Value::Undefined);
                                                self.resolve_promise(*promise_idx, val);
                                                self.stack.push(Value::Undefined);
                                            }
                                            Some("reject") => {
                                                let reason = args.first().cloned().unwrap_or(Value::Undefined);
                                                self.reject_promise(*promise_idx, reason);
                                                self.stack.push(Value::Undefined);
                                            }
                                            _ => {
                                                self.stack.push(Value::Undefined);
                                            }
                                        }
                                        continue;
                                    }
                                }
                                let return_address = pc + 1;
                                let base_pointer = self.stack.len();
                                let closure_count = f.closure.len();

                                self.call_stack.push(CallFrame {
                                    return_address,
                                    base_pointer,
                                    closure_var_count: closure_count,
                                    func_heap_idx: Some(func_idx),
                                    this_value: None,
                                    is_construct: false,
                                });

                                for closure_var in &f.closure {
                                    self.stack.push(closure_var.clone());
                                }

                                for arg in args {
                                    self.stack.push(arg);
                                }

                                pc = f.bytecode_index;
                                continue;
                            }
                        }
                        Value::NativeFunction(native_idx) => {
                            let result = self.call_native(native_idx, &Value::Undefined, &args)?;
                            self.stack.push(result);
                        }
                        _ => {
                            return Err(Error::TypeError(format!("{} is not a function", self.value_to_string(&callee))));
                        }
                    }
                }
                Instruction::CallMethod(argc) => {
                    let mut args = Vec::new();
                    for _ in 0..*argc {
                        args.push(self.stack.pop()
                            .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?);
                    }
                    args.reverse();

                    let key = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let object = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;

                    let method = self.get_property(&object, &key)?;

                    match method {
                        Value::Function(func_idx) => {
                            if let HeapValue::Function(f) = &self.heap[func_idx] {
                                let f_clone = f.clone();
                                let return_address = pc + 1;
                                let base_pointer = self.stack.len();
                                let closure_count = f_clone.closure.len();

                                self.call_stack.push(CallFrame {
                                    return_address,
                                    base_pointer,
                                    closure_var_count: closure_count,
                                    func_heap_idx: Some(func_idx),
                                    this_value: Some(object),
                                    is_construct: false,
                                });

                                for closure_var in &f_clone.closure {
                                    self.stack.push(closure_var.clone());
                                }

                                for arg in args {
                                    self.stack.push(arg);
                                }

                                pc = f_clone.bytecode_index;
                                continue;
                            }
                        }
                        Value::NativeFunction(native_idx) => {
                            let result = self.call_native(native_idx, &object, &args)?;
                            self.stack.push(result);
                        }
                        _ => {
                            return Err(Error::TypeError(format!("{} is not a function", self.value_to_string(&method))));
                        }
                    }
                }
                Instruction::Construct(argc) => {
                    let mut args = Vec::new();
                    for _ in 0..*argc {
                        args.push(self.stack.pop()
                            .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?);
                    }
                    args.reverse();

                    let constructor = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;

                    match &constructor {
                        Value::Function(func_idx) => {
                            let proto_idx = if let Value::Object(proto_obj_idx) = self.get_property(&constructor, &Value::String("prototype".to_string()))? {
                                Some(proto_obj_idx)
                            } else {
                                None
                            };

                            let new_obj_heap_idx = self.heap.len();
                            self.heap.push(HeapValue::Object(JsObject::with_prototype(proto_idx)));
                            let this_val = Value::Object(new_obj_heap_idx);

                            if let HeapValue::Function(f) = &self.heap[*func_idx] {
                                if f.bytecode_index == usize::MAX {
                                    self.stack.push(this_val);
                                } else {
                                    let f_clone = f.clone();
                                    let return_address = pc + 1;
                                    let base_pointer = self.stack.len();
                                    let closure_count = f_clone.closure.len();

                                    self.call_stack.push(CallFrame {
                                        return_address,
                                        base_pointer,
                                        closure_var_count: closure_count,
                                        func_heap_idx: Some(*func_idx),
                                        this_value: Some(this_val.clone()),
                                        is_construct: true,
                                    });

                                    for closure_var in &f_clone.closure {
                                        self.stack.push(closure_var.clone());
                                    }

                                    for arg in args {
                                        self.stack.push(arg);
                                    }

                                    pc = f_clone.bytecode_index;
                                    continue;
                                }
                            }
                        }
                        Value::NativeFunction(native_idx) => {
                            let proto_idx = self.find_native_prototype(*native_idx);
                            let new_obj_heap_idx = self.heap.len();
                            self.heap.push(HeapValue::Object(JsObject::with_prototype(proto_idx)));
                            let this_val = Value::Object(new_obj_heap_idx);
                            let result = self.call_native(*native_idx, &this_val, &args)?;
                            match result {
                                Value::Object(_) | Value::Array(_) | Value::Function(_) | Value::Promise(_) => {
                                    self.stack.push(result);
                                }
                                _ => {
                                    self.stack.push(this_val);
                                }
                            }
                        }
                        _ => {
                            return Err(Error::TypeError(format!("{} is not a constructor", self.value_to_string(&constructor))));
                        }
                    }
                }
                Instruction::LoadThis => {
                    let this = self.call_stack.last()
                        .and_then(|f| f.this_value.clone())
                        .unwrap_or(Value::Undefined);
                    self.stack.push(this);
                }
                Instruction::Return => {
                    let return_value = self.stack.pop()
                        .unwrap_or(Value::Undefined);

                    if let Some(frame) = self.call_stack.pop() {
                        if frame.closure_var_count > 0 {
                            if let Some(heap_idx) = frame.func_heap_idx {
                                if let HeapValue::Function(f) = &mut self.heap[heap_idx] {
                                    f.closure.clear();
                                    for i in 0..frame.closure_var_count {
                                        let val = self.stack.get(frame.base_pointer + i)
                                            .cloned()
                                            .unwrap_or(Value::Undefined);
                                        f.closure.push(val);
                                    }
                                }
                            }
                        }

                        let final_value = if frame.is_construct {
                            match &return_value {
                                Value::Object(_) | Value::Array(_) | Value::Function(_) => return_value,
                                _ => frame.this_value.clone().unwrap_or(return_value),
                            }
                        } else {
                            return_value
                        };

                        self.stack.truncate(frame.base_pointer);
                        self.stack.push(final_value);
                        pc = frame.return_address;
                        continue;
                    } else {
                        return Ok(return_value);
                    }
                }
                Instruction::Pop => {
                    self.stack.pop();
                }
                Instruction::Dup => {
                    let val = self.stack.last()
                        .cloned()
                        .unwrap_or(Value::Undefined);
                    self.stack.push(val);
                }
                Instruction::MakeFunction(func_idx) => {
                    let func_info = module.functions[*func_idx as usize].clone();

                    let proto_obj_idx = self.heap.len();
                    self.heap.push(HeapValue::Object(JsObject::new()));

                    let heap_idx = self.heap.len();
                    self.heap.push(HeapValue::Function(JsFunction {
                        name: func_info.name,
                        params: func_info.params,
                        bytecode_index: func_info.bytecode_index,
                        closure: Vec::new(),
                        prototype: Some(proto_obj_idx),
                        super_class: None,
                        properties: HashMap::new(),
                    }));

                    self.stack.push(Value::Function(heap_idx));
                }
                Instruction::MakeClosure(func_idx, _capture_slots) => {
                    let func_info = module.functions[*func_idx as usize].clone();

                    let mut closure_vars = Vec::new();
                    for slot in _capture_slots {
                        let value = self.stack.get(*slot as usize)
                            .cloned()
                            .unwrap_or(Value::Undefined);
                        closure_vars.push(value);
                    }

                    let proto_obj_idx = self.heap.len();
                    self.heap.push(HeapValue::Object(JsObject::new()));

                    let heap_idx = self.heap.len();
                    self.heap.push(HeapValue::Function(JsFunction {
                        name: func_info.name,
                        params: func_info.params,
                        bytecode_index: func_info.bytecode_index,
                        closure: closure_vars,
                        prototype: Some(proto_obj_idx),
                        super_class: None,
                        properties: HashMap::new(),
                    }));

                    self.stack.push(Value::Function(heap_idx));
                }
                Instruction::NewObject => {
                    let heap_idx = self.heap.len();
                    self.heap.push(HeapValue::Object(JsObject::new()));
                    self.stack.push(Value::Object(heap_idx));
                }
                Instruction::SetProperty => {
                    let value = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let key = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let object = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;

                    if let Value::Object(obj_idx) = &object {
                        if let HeapValue::Object(obj) = &mut self.heap[*obj_idx] {
                            if let Value::String(key_str) = &key {
                                obj.properties.insert(key_str.clone(), value);
                            }
                        }
                    } else if let Value::Function(func_idx) = &object {
                        if let HeapValue::Function(f) = &mut self.heap[*func_idx] {
                            if let Value::String(key_str) = &key {
                                f.properties.insert(key_str.clone(), value);
                            }
                        }
                    }
                    self.stack.push(object);
                }
                Instruction::GetProperty => {
                    let key = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let object = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    
                    let result = self.get_property(&object, &key)?;
                    self.stack.push(result);
                }
                Instruction::NewArray(size) => {
                    let mut elements = Vec::new();
                    for _ in 0..*size {
                        elements.push(self.stack.pop().unwrap_or(Value::Undefined));
                    }
                    let heap_idx = self.heap.len();
                    self.heap.push(HeapValue::Array(JsArray { elements }));
                    self.stack.push(Value::Array(heap_idx));
                }
                Instruction::TypeOf => {
                    let value = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let type_str = match &value {
                        Value::Undefined => "undefined",
                        Value::Null => "object",
                        Value::Boolean(_) => "boolean",
                        Value::Integer(_) | Value::Float(_) => "number",
                        Value::String(_) => "string",
                        Value::BigInt(_) => "bigint",
                    Value::Function(_) | Value::NativeFunction(_) => "function",
                    Value::Object(_) | Value::Array(_) | Value::Promise(_) => "object",
                    };
                    self.stack.push(Value::String(type_str.to_string()));
                }
                Instruction::BitNot => {
                    let value = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let num = self.to_number(&value)?;
                    self.stack.push(Value::Integer(!(num as i64) as i64));
                }
                Instruction::Void => {
                    self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    self.stack.push(Value::Undefined);
                }
                Instruction::Delete => {
                    let key = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let object = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;

                    let result = self.delete_property(&object, &key);
                    self.stack.push(result);
                }
                Instruction::InstanceOf => {
                    let right = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let left = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;

                    let result = self.instanceof_check(&left, &right)?;
                    self.stack.push(result);
                }
                Instruction::In => {
                    let right = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let left = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;

                    let result = self.in_check(&left, &right)?;
                    self.stack.push(result);
                }
                Instruction::Throw => {
                    let value = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    self.pending_exception = Some(value.clone());

                    while let Some(handler) = self.exception_handlers.last().cloned() {
                        if handler.catch_pc != 0 {
                            self.exception_handlers.pop();
                            self.stack.truncate(handler.stack_depth);
                            pc = handler.catch_pc as usize;
                            continue 'main;
                        } else if handler.finally_pc != 0 {
                            self.exception_handlers.pop();
                            self.stack.truncate(handler.stack_depth);
                            pc = handler.finally_pc as usize;
                            continue 'main;
                        } else {
                            self.exception_handlers.pop();
                        }
                    }

                    return Err(Error::RuntimeError(format!("Thrown: {}", self.value_to_string(&value))));
                }
                Instruction::TryJump(catch_pc, finally_pc) => {
                    let handler = ExceptionHandler {
                        catch_pc: *catch_pc,
                        finally_pc: *finally_pc,
                        stack_depth: self.stack.len(),
                    };
                    self.exception_handlers.push(handler);
                }
                Instruction::PopTryHandler => {
                    self.exception_handlers.pop();
                    if self.pending_exception.is_some() {
                        while let Some(handler) = self.exception_handlers.last().cloned() {
                            if handler.catch_pc != 0 {
                                self.exception_handlers.pop();
                                self.stack.truncate(handler.stack_depth);
                                pc = handler.catch_pc as usize;
                                continue 'main;
                            } else if handler.finally_pc != 0 {
                                self.exception_handlers.pop();
                                self.stack.truncate(handler.stack_depth);
                                pc = handler.finally_pc as usize;
                                continue 'main;
                            } else {
                                self.exception_handlers.pop();
                            }
                        }
                        let exc = self.pending_exception.take()
                            .unwrap_or(Value::Undefined);
                        return Err(Error::RuntimeError(format!("Thrown: {}", self.value_to_string(&exc))));
                    }
                }
                Instruction::LoadException => {
                    let exc = self.pending_exception.take()
                        .unwrap_or(Value::Undefined);
                    self.stack.push(exc);
                }
                Instruction::ReThrowIfPending => {
                    if self.pending_exception.is_some() {
                        while let Some(handler) = self.exception_handlers.last().cloned() {
                            if handler.catch_pc != 0 {
                                self.exception_handlers.pop();
                                self.stack.truncate(handler.stack_depth);
                                pc = handler.catch_pc as usize;
                                continue 'main;
                            } else if handler.finally_pc != 0 {
                                self.exception_handlers.pop();
                                self.stack.truncate(handler.stack_depth);
                                pc = handler.finally_pc as usize;
                                continue 'main;
                            } else {
                                self.exception_handlers.pop();
                            }
                        }
                        let exc = self.pending_exception.take()
                            .unwrap_or(Value::Undefined);
                        return Err(Error::RuntimeError(format!("Thrown: {}", self.value_to_string(&exc))));
                    }
                }
                Instruction::ToString => {
                    let value = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let s = self.value_to_string_raw(&value);
                    self.stack.push(Value::String(s));
                }
                Instruction::MakeClass(class_info_idx) => {
                    let class_info = module.class_infos[*class_info_idx as usize].clone();

                    let super_val = if class_info.superclass.is_some() {
                        self.stack.pop().ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?
                    } else {
                        Value::Undefined
                    };

                    let proto_obj_idx = self.heap.len();
                    self.heap.push(HeapValue::Object(JsObject::new()));

                    let super_proto = match &super_val {
                        Value::Object(super_obj_idx) => {
                            if let HeapValue::Object(super_obj) = &self.heap[*super_obj_idx] {
                                super_obj.properties.get("prototype").cloned()
                            } else {
                                None
                            }
                        }
                        Value::Function(func_idx) => {
                            if let HeapValue::Function(f) = &self.heap[*func_idx] {
                                f.prototype.map(|idx| Value::Object(idx))
                            } else {
                                None
                            }
                        }
                        _ => None,
                    };
                    if let Some(Value::Object(sp_idx)) = super_proto {
                        self.heap[proto_obj_idx] = HeapValue::Object(JsObject::with_prototype(Some(sp_idx)));
                    }

                    let ctor_heap_idx = if let Some(ctor_func_idx) = class_info.constructor_func_idx {
                        let func_info = module.functions[ctor_func_idx as usize].clone();

                        let idx = self.heap.len();
                        self.heap.push(HeapValue::Function(JsFunction {
                            name: func_info.name,
                            params: func_info.params,
                            bytecode_index: func_info.bytecode_index,
                            closure: Vec::new(),
                            prototype: Some(proto_obj_idx),
                            super_class: Some(super_val.clone()),
                            properties: HashMap::new(),
                        }));
                        idx
                    } else {
                        let idx = self.heap.len();
                        self.heap.push(HeapValue::Function(JsFunction {
                            name: Some(class_info.name.clone()),
                            params: Vec::new(),
                            bytecode_index: usize::MAX,
                            closure: Vec::new(),
                            prototype: Some(proto_obj_idx),
                            super_class: Some(super_val.clone()),
                            properties: HashMap::new(),
                        }));
                        idx
                    };

                    if let HeapValue::Object(proto_obj) = &mut self.heap[proto_obj_idx] {
                        proto_obj.properties.insert("constructor".to_string(), Value::Function(ctor_heap_idx));
                    }

                    for method_info in &class_info.methods {
                        let method_func_info = module.functions[method_info.func_idx as usize].clone();

                        let method_proto_idx = self.heap.len();
                        self.heap.push(HeapValue::Object(JsObject::new()));

                        let method_heap_idx = self.heap.len();
                        self.heap.push(HeapValue::Function(JsFunction {
                            name: Some(method_info.name.clone()),
                            params: method_func_info.params,
                            bytecode_index: method_func_info.bytecode_index,
                            closure: Vec::new(),
                            prototype: Some(method_proto_idx),
                            super_class: None,
                            properties: HashMap::new(),
                        }));
                        let method_val = Value::Function(method_heap_idx);

                        if method_info.is_static {
                            if let HeapValue::Function(ctor_func) = &mut self.heap[ctor_heap_idx] {
                                ctor_func.properties.insert(method_info.name.clone(), method_val);
                            }
                        } else {
                            match &method_info.kind {
                                crate::compiler::ClassMethodKind::Getter => {
                                    if let HeapValue::Object(proto_obj) = &mut self.heap[proto_obj_idx] {
                                        proto_obj.properties.insert(
                                            format!("__getter_{}", method_info.name),
                                            method_val,
                                        );
                                    }
                                }
                                crate::compiler::ClassMethodKind::Setter => {
                                    if let HeapValue::Object(proto_obj) = &mut self.heap[proto_obj_idx] {
                                        proto_obj.properties.insert(
                                            format!("__setter_{}", method_info.name),
                                            method_val,
                                        );
                                    }
                                }
                                crate::compiler::ClassMethodKind::Method => {
                                    if let HeapValue::Object(proto_obj) = &mut self.heap[proto_obj_idx] {
                                        proto_obj.properties.insert(method_info.name.clone(), method_val);
                                    }
                                }
                            }
                        }
                    }

                    self.stack.push(Value::Function(ctor_heap_idx));
                }
                Instruction::SuperConstruct(argc) => {
                    let mut args = Vec::new();
                    for _ in 0..*argc {
                        args.push(self.stack.pop()
                            .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?);
                    }
                    args.reverse();

                    let this_val = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;

                    let super_class = {
                        let mut found = Value::Undefined;
                        for frame in self.call_stack.iter().rev() {
                            if let Some(func_idx) = frame.func_heap_idx {
                                if let HeapValue::Function(f) = &self.heap[func_idx] {
                                    if let Some(ref sc) = f.super_class {
                                        found = sc.clone();
                                        break;
                                    }
                                }
                            }
                        }
                        found
                    };

                    match super_class {
                        Value::Function(func_idx) => {
                            if let HeapValue::Function(f) = &self.heap[func_idx] {
                                let f_clone = f.clone();

                                let new_obj_heap_idx = self.heap.len();
                                let proto_val = self.get_property(&super_class, &Value::String("prototype".to_string()))?;
                                let proto_idx = if let Value::Object(pi) = proto_val { Some(pi) } else { None };
                                self.heap.push(HeapValue::Object(JsObject::with_prototype(proto_idx)));

                                let constructed = Value::Object(new_obj_heap_idx);

                                let return_address = pc + 1;
                                let base_pointer = self.stack.len();
                                let closure_count = f_clone.closure.len();

                                self.call_stack.push(CallFrame {
                                    return_address,
                                    base_pointer,
                                    closure_var_count: closure_count,
                                    func_heap_idx: Some(func_idx),
                                    this_value: Some(this_val.clone()),
                                    is_construct: true,
                                });

                                for closure_var in &f_clone.closure {
                                    self.stack.push(closure_var.clone());
                                }
                                for arg in args {
                                    self.stack.push(arg);
                                }

                                pc = f_clone.bytecode_index;
                                continue;
                            }
                        }
                        Value::NativeFunction(native_idx) => {
                            let result = self.call_native(native_idx, &this_val, &args)?;
                            self.stack.push(result);
                        }
                        _ => {
                            return Err(Error::TypeError("Superclass is not a constructor".into()));
                        }
                    }
                }
                Instruction::SuperGet => {
                    let key = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let _this = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;

                    let mut super_val = Value::Undefined;
                    for frame in self.call_stack.iter().rev() {
                        if let Some(func_idx) = frame.func_heap_idx {
                            if let HeapValue::Function(f) = &self.heap[func_idx] {
                                if let Some(ref sc) = f.super_class {
                                    super_val = sc.clone();
                                    break;
                                }
                            }
                        }
                    }

                    match &super_val {
                        Value::Function(func_idx) => {
                            if let HeapValue::Function(f) = &self.heap[*func_idx] {
                                if let Some(proto_idx) = f.prototype {
                                    let proto_val = Value::Object(proto_idx);
                                    let result = self.get_property(&proto_val, &key)?;
                                    self.stack.push(result);
                                    continue;
                                }
                            }
                        }
                        _ => {}
                    }
                    self.stack.push(Value::Undefined);
                }
                _ => {
                    return Err(Error::RuntimeError(format!("Unhandled instruction: {:?}", instruction)));
                }
            }
            
            pc += 1;
        }
        
        Ok(self.stack.pop().unwrap_or(Value::Undefined))
    }
    
    pub fn get_global(&self, name: &str) -> Option<Value> {
        self.globals.get(name).cloned()
    }
    
    pub fn set_global(&mut self, name: &str, value: Value) {
        self.globals.insert(name.to_string(), value);
    }

    pub fn call_value(&mut self, callee: &Value, this: &Value, args: &[Value]) -> Result<Value> {
        match callee {
            Value::Function(func_idx) => {
                if let HeapValue::Function(f) = &self.heap[*func_idx] {
                    let f_clone = f.clone();
                    let return_address = self.current_module.as_ref()
                        .map(|m| m.instructions.len())
                        .unwrap_or(0);
                    let base_pointer = self.stack.len();
                    let closure_count = f_clone.closure.len();

                    eprintln!("call_value: func={:?} base={} closures={} args={} bytecode={}",
                        f_clone.name, base_pointer, closure_count, args.len(), f_clone.bytecode_index);

                    self.call_stack.push(CallFrame {
                        return_address,
                        base_pointer,
                        closure_var_count: closure_count,
                        func_heap_idx: Some(*func_idx),
                        this_value: Some(this.clone()),
                        is_construct: false,
                    });

                    for closure_var in &f_clone.closure {
                        self.stack.push(closure_var.clone());
                    }
                    for arg in args {
                        self.stack.push(arg.clone());
                    }

                    if let Some(module) = self.current_module.clone() {
                        self.execute_from(&module, f_clone.bytecode_index)
                    } else {
                        Ok(Value::Undefined)
                    }
                } else {
                    Err(Error::TypeError("Not a function".into()))
                }
            }
            Value::NativeFunction(native_idx) => {
                self.call_native(*native_idx, this, args)
            }
            _ => Err(Error::TypeError(format!("{} is not a function", self.value_to_string(callee)))),
        }
    }

    fn call_native(&mut self, idx: usize, this: &Value, args: &[Value]) -> Result<Value> {
        if idx < NATIVE_TABLE.len() {
            NATIVE_TABLE[idx](self, this, args)
        } else {
            Err(Error::RuntimeError(format!("Unknown native function index: {}", idx)))
        }
    }

    fn find_native_prototype(&self, native_idx: usize) -> Option<usize> {
        let ctor_name = match native_idx {
            72 => "Error",
            73 => "TypeError",
            74 => "ReferenceError",
            75 => "SyntaxError",
            76 => "RangeError",
            _ => return None,
        };
        for (i, hv) in self.heap.iter().enumerate() {
            if let HeapValue::Object(obj) = hv {
                if let Some(Value::String(name)) = obj.properties.get("name") {
                    if name == ctor_name {
                        return Some(i);
                    }
                }
            }
        }
        None
    }
    
    fn get_property(&mut self, object: &Value, key: &Value) -> Result<Value> {
        self.get_property_with_this(object, key, object)
    }

    fn get_property_with_this(&mut self, object: &Value, key: &Value, this: &Value) -> Result<Value> {
        match object {
            Value::Null | Value::Undefined => {
                return Err(Error::TypeError(format!(
                    "Cannot read properties of {} (reading '{}')",
                    self.value_to_string(object),
                    self.value_to_string(key)
                )));
            }
            Value::Object(obj_idx) => {
                if let HeapValue::Object(obj) = &self.heap[*obj_idx] {
                    if let Value::String(key_str) = key {
                        if let Some(val) = obj.properties.get(key_str) {
                            return Ok(val.clone());
                        }
                        let getter_key = format!("__getter_{}", key_str);
                        if let Some(getter_val) = obj.properties.get(&getter_key).cloned() {
                            return self.call_value(&getter_val, this, &[]);
                        }
                        if let Some(proto_idx) = obj.prototype {
                            let proto_val = Value::Object(proto_idx);
                            return self.get_property_with_this(&proto_val, key, this);
                        }
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
                                return Ok(arr.elements.get(index).cloned().unwrap_or(Value::Undefined));
                            }
                            return self.get_array_method(key_str);
                        }
                        Value::Integer(index) => {
                            let idx = *index as usize;
                            return Ok(arr.elements.get(idx).cloned().unwrap_or(Value::Undefined));
                        }
                        Value::Float(f) => {
                            let idx = *f as usize;
                            return Ok(arr.elements.get(idx).cloned().unwrap_or(Value::Undefined));
                        }
                        _ => {}
                    }
                }
            }
            Value::String(s) => {
                return self.get_property_from_primitive_string(s, key);
            }
            Value::Integer(_) | Value::Float(_) => {
                return self.get_property_from_primitive_number(object, key);
            }
            Value::Boolean(_) => {
                return self.get_property_from_primitive_boolean(object, key);
            }
            Value::Function(func_idx) => {
                if let Value::String(key_str) = key {
                    if key_str == "prototype" {
                        if let HeapValue::Function(f) = &self.heap[*func_idx] {
                            if let Some(proto_idx) = f.prototype {
                                return Ok(Value::Object(proto_idx));
                            }
                        }
                    }
                    if let HeapValue::Function(f) = &self.heap[*func_idx] {
                        if let Some(val) = f.properties.get(key_str) {
                            return Ok(val.clone());
                        }
                    }
                }
            }
            Value::Promise(promise_idx) => {
                if let Value::String(key_str) = key {
                    match key_str.as_str() {
                        "then" => return Ok(Value::NativeFunction(78)),
                        "catch" => return Ok(Value::NativeFunction(79)),
                        "finally" => return Ok(Value::NativeFunction(80)),
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
            }
            Value::NativeFunction(idx) => {
                if let Value::String(key_str) = key {
                    if *idx == 77 {
                        match key_str.as_str() {
                            "resolve" => return Ok(Value::NativeFunction(81)),
                            "reject" => return Ok(Value::NativeFunction(82)),
                            "all" => return Ok(Value::NativeFunction(83)),
                            "race" => return Ok(Value::NativeFunction(84)),
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(Value::Undefined)
    }

    fn get_array_method(&self, name: &str) -> Result<Value> {
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
            _ => return Ok(Value::Undefined),
        };
        Ok(Value::NativeFunction(idx))
    }

    fn get_property_from_primitive_string(&self, s: &str, key: &Value) -> Result<Value> {
        if let Value::String(key_str) = key {
            match key_str.as_str() {
                "length" => return Ok(Value::Float(s.len() as f64)),
                _ => {}
            }
            return self.get_string_method(key_str);
        }
        Ok(Value::Undefined)
    }

    fn get_string_method(&self, name: &str) -> Result<Value> {
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
            _ => return Ok(Value::Undefined),
        };
        Ok(Value::NativeFunction(idx))
    }

    fn get_property_from_primitive_number(&self, n: &Value, key: &Value) -> Result<Value> {
        if let Value::String(key_str) = key {
            match key_str.as_str() {
                "toString" | "toFixed" | "valueOf" => {
                    return Ok(self.make_native_number_method(key_str));
                }
                _ => {}
            }
        }
        Ok(Value::Undefined)
    }

    fn get_property_from_primitive_boolean(&self, b: &Value, key: &Value) -> Result<Value> {
        if let Value::String(key_str) = key {
            match key_str.as_str() {
                "toString" | "valueOf" => {
                    return Ok(self.make_native_boolean_method(key_str));
                }
                _ => {}
            }
        }
        Ok(Value::Undefined)
    }

    fn make_native_number_method(&self, _name: &str) -> Value {
        Value::NativeFunction(0)
    }

    fn make_native_boolean_method(&self, _name: &str) -> Value {
        Value::NativeFunction(0)
    }

    fn delete_property(&mut self, object: &Value, key: &Value) -> Value {
        match object {
            Value::Object(obj_idx) => {
                if let HeapValue::Object(obj) = &mut self.heap[*obj_idx] {
                    if let Value::String(key_str) = key {
                        if obj.properties.remove(key_str).is_some() {
                            return Value::Boolean(true);
                        }
                    }
                }
                Value::Boolean(false)
            }
            Value::Array(arr_idx) => {
                if let Value::String(key_str) = key {
                    if let Ok(index) = key_str.parse::<usize>() {
                        if let HeapValue::Array(arr) = &mut self.heap[*arr_idx] {
                            if index < arr.elements.len() {
                                arr.elements[index] = Value::Undefined;
                                return Value::Boolean(true);
                            }
                        }
                    }
                }
                Value::Boolean(false)
            }
            _ => Value::Boolean(true),
        }
    }

    fn instanceof_check(&mut self, left: &Value, right: &Value) -> Result<Value> {
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
                Value::Array(arr_idx) => {
                    break;
                }
                _ => break,
            }
        }
        Ok(Value::Boolean(false))
    }

    fn in_check(&self, key: &Value, object: &Value) -> Result<Value> {
        match object {
            Value::Object(obj_idx) => {
                if let HeapValue::Object(obj) = &self.heap[*obj_idx] {
                    if let Value::String(key_str) = key {
                        if obj.properties.contains_key(key_str) {
                            return Ok(Value::Boolean(true));
                        }
                        if let Some(proto_idx) = obj.prototype {
                            let proto_val = Value::Object(proto_idx);
                            return self.in_check(key, &proto_val);
                        }
                    }
                }
                Ok(Value::Boolean(false))
            }
            Value::Array(arr_idx) => {
                if let HeapValue::Array(arr) = &self.heap[*arr_idx] {
                    if let Value::String(key_str) = key {
                        if key_str == "length" {
                            return Ok(Value::Boolean(true));
                        }
                        if let Ok(index) = key_str.parse::<usize>() {
                            return Ok(Value::Boolean(index < arr.elements.len()));
                        }
                    }
                }
                Ok(Value::Boolean(false))
            }
            Value::String(s) => {
                if let Value::String(key_str) = key {
                    if key_str == "length" {
                        return Ok(Value::Boolean(true));
                    }
                    if let Ok(index) = key_str.parse::<usize>() {
                        return Ok(Value::Boolean(index < s.len()));
                    }
                }
                Ok(Value::Boolean(false))
            }
            _ => Ok(Value::Boolean(false)),
        }
    }
    
    fn add(&self, left: Value, right: Value) -> Result<Value> {
        match (&left, &right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a + *b as f64)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
            _ => Err(Error::TypeError(format!(
                "Cannot add {} and {}",
                self.value_to_string(&left),
                self.value_to_string(&right)
            ))),
        }
    }
    
    fn sub(&self, left: Value, right: Value) -> Result<Value> {
        match (&left, &right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a - b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 - b)),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a - *b as f64)),
            _ => Err(Error::TypeError(format!(
                "Cannot subtract {} from {}",
                self.value_to_string(&right),
                self.value_to_string(&left)
            ))),
        }
    }
    
    fn mul(&self, left: Value, right: Value) -> Result<Value> {
        match (&left, &right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a * b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 * b)),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a * *b as f64)),
            _ => Err(Error::TypeError(format!(
                "Cannot multiply {} and {}",
                self.value_to_string(&left),
                self.value_to_string(&right)
            ))),
        }
    }
    
    fn div(&self, left: Value, right: Value) -> Result<Value> {
        match (&left, &right) {
            (Value::Integer(a), Value::Integer(b)) => {
                if *b == 0 {
                    return Err(Error::RuntimeError("Division by zero".into()));
                }
                Ok(Value::Float(*a as f64 / *b as f64))
            }
            (Value::Float(a), Value::Float(b)) => {
                if *b == 0.0 {
                    return Err(Error::RuntimeError("Division by zero".into()));
                }
                Ok(Value::Float(a / b))
            }
            (Value::Integer(a), Value::Float(b)) => {
                if *b == 0.0 {
                    return Err(Error::RuntimeError("Division by zero".into()));
                }
                Ok(Value::Float(*a as f64 / b))
            }
            (Value::Float(a), Value::Integer(b)) => {
                if *b == 0 {
                    return Err(Error::RuntimeError("Division by zero".into()));
                }
                Ok(Value::Float(a / *b as f64))
            }
            _ => Err(Error::TypeError(format!(
                "Cannot divide {} by {}",
                self.value_to_string(&left),
                self.value_to_string(&right)
            ))),
        }
    }
    
    fn modulo(&self, left: Value, right: Value) -> Result<Value> {
        match (&left, &right) {
            (Value::Integer(a), Value::Integer(b)) => {
                if *b == 0 {
                    return Err(Error::RuntimeError("Division by zero".into()));
                }
                Ok(Value::Integer(a % b))
            }
            (Value::Float(a), Value::Float(b)) => {
                if *b == 0.0 {
                    return Err(Error::RuntimeError("Division by zero".into()));
                }
                Ok(Value::Float(a % b))
            }
            _ => Err(Error::TypeError(format!(
                "Cannot apply modulo to {} and {}",
                self.value_to_string(&left),
                self.value_to_string(&right)
            ))),
        }
    }
    
    fn power(&self, left: Value, right: Value) -> Result<Value> {
        match (&left, &right) {
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.powf(*b))),
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Float((*a as f64).powf(*b as f64))),
            _ => Err(Error::TypeError(format!(
                "Cannot raise {} to the power of {}",
                self.value_to_string(&left),
                self.value_to_string(&right)
            ))),
        }
    }
    
    fn negate(&self, value: Value) -> Result<Value> {
        match value {
            Value::Integer(n) => Ok(Value::Integer(-n)),
            Value::Float(n) => Ok(Value::Float(-n)),
            _ => Err(Error::TypeError(format!("Cannot negate {}", self.value_to_string(&value)))),
        }
    }
    
    pub(crate) fn is_truthy(&self, value: &Value) -> bool {
        match value {
            Value::Undefined => false,
            Value::Null => false,
            Value::Boolean(b) => *b,
            Value::Integer(n) => *n != 0,
            Value::Float(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::BigInt(n) => *n != 0,
            _ => true,
        }
    }
    
    fn is_equal(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Undefined, Value::Undefined) => true,
            (Value::Null, Value::Null) => true,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Integer(a), Value::Float(b)) => *a as f64 == *b,
            (Value::Float(a), Value::Integer(b)) => *a == *b as f64,
            (Value::String(a), Value::String(b)) => a == b,
            _ => false,
        }
    }
    
    fn less_than(&self, left: &Value, right: &Value) -> Result<bool> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(a < b),
            (Value::Float(a), Value::Float(b)) => Ok(a < b),
            (Value::Integer(a), Value::Float(b)) => Ok((*a as f64) < *b),
            (Value::Float(a), Value::Integer(b)) => Ok(*a < (*b as f64)),
            _ => Err(Error::TypeError(format!(
                "Cannot compare {} and {}",
                self.value_to_string(left),
                self.value_to_string(right)
            ))),
        }
    }
    
    fn greater_than(&self, left: &Value, right: &Value) -> Result<bool> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(a > b),
            (Value::Float(a), Value::Float(b)) => Ok(a > b),
            (Value::Integer(a), Value::Float(b)) => Ok((*a as f64) > *b),
            (Value::Float(a), Value::Integer(b)) => Ok(*a > (*b as f64)),
            _ => Err(Error::TypeError(format!(
                "Cannot compare {} and {}",
                self.value_to_string(left),
                self.value_to_string(right)
            ))),
        }
    }
    
    fn less_than_or_equal(&self, left: &Value, right: &Value) -> Result<bool> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(a <= b),
            (Value::Float(a), Value::Float(b)) => Ok(a <= b),
            (Value::Integer(a), Value::Float(b)) => Ok((*a as f64) <= *b),
            (Value::Float(a), Value::Integer(b)) => Ok(*a <= (*b as f64)),
            _ => Err(Error::TypeError(format!(
                "Cannot compare {} and {}",
                self.value_to_string(left),
                self.value_to_string(right)
            ))),
        }
    }
    
    fn greater_than_or_equal(&self, left: &Value, right: &Value) -> Result<bool> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(a >= b),
            (Value::Float(a), Value::Float(b)) => Ok(a >= b),
            (Value::Integer(a), Value::Float(b)) => Ok((*a as f64) >= *b),
            (Value::Float(a), Value::Integer(b)) => Ok(*a >= (*b as f64)),
            _ => Err(Error::TypeError(format!(
                "Cannot compare {} and {}",
                self.value_to_string(left),
                self.value_to_string(right)
            ))),
        }
    }
    
    fn to_number(&self, value: &Value) -> Result<f64> {
        match value {
            Value::Integer(n) => Ok(*n as f64),
            Value::Float(n) => Ok(*n),
            Value::Boolean(b) => Ok(if *b { 1.0 } else { 0.0 }),
            Value::Null => Ok(0.0),
            Value::Undefined => Ok(f64::NAN),
            Value::String(s) => {
                if s.is_empty() { Ok(0.0) }
                else { Ok(s.parse::<f64>().unwrap_or(f64::NAN)) }
            }
            _ => Ok(f64::NAN),
        }
    }

    fn value_to_string_raw(&self, value: &Value) -> String {
        match value {
            Value::Undefined => "undefined".to_string(),
            Value::Null => "null".to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::Integer(n) => n.to_string(),
            Value::Float(n) => {
                if *n == (*n as i64) as f64 {
                    (*n as i64).to_string()
                } else {
                    n.to_string()
                }
            }
            Value::String(s) => s.clone(),
            Value::BigInt(n) => format!("{}n", n),
            Value::Function(_) => "[Function]".to_string(),
            Value::NativeFunction(_) => "[Native Function]".to_string(),
            Value::Object(_) => "[Object]".to_string(),
            Value::Array(_) => "[Array]".to_string(),
            Value::Promise(_) => "[Promise]".to_string(),
        }
    }

    fn value_to_string(&self, value: &Value) -> String {
        match value {
            Value::Undefined => "undefined".to_string(),
            Value::Null => "null".to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::Integer(n) => n.to_string(),
            Value::Float(n) => n.to_string(),
            Value::String(s) => format!("\"{}\"", s),
            Value::BigInt(n) => format!("{}n", n),
            Value::Function(_) => "[Function]".to_string(),
            Value::NativeFunction(_) => "[Native Function]".to_string(),
            Value::Object(_) => "[Object]".to_string(),
            Value::Array(_) => "[Array]".to_string(),
            Value::Promise(_) => "[Promise]".to_string(),
        }
    }

    pub(crate) fn drain_microtasks(&mut self) {
        while let Some(task) = self.async_runtime.dequeue_microtask() {
            let _ = self.call_value(&task.callback, &Value::Undefined, &[task.arg]);
        }
    }

    pub(crate) fn create_resolve_fn(&mut self, promise_idx: usize) -> Value {
        let proto_idx = self.heap.len();
        self.heap.push(HeapValue::Object(JsObject::new()));
        let heap_idx = self.heap.len();
        self.heap.push(HeapValue::Function(JsFunction {
            name: Some("resolve".into()),
            params: vec!["value".into()],
            bytecode_index: usize::MAX,
            closure: vec![Value::Promise(promise_idx)],
            prototype: Some(proto_idx),
            super_class: None,
            properties: HashMap::new(),
        }));
        Value::Function(heap_idx)
    }

    pub(crate) fn create_reject_fn(&mut self, promise_idx: usize) -> Value {
        let proto_idx = self.heap.len();
        self.heap.push(HeapValue::Object(JsObject::new()));
        let heap_idx = self.heap.len();
        self.heap.push(HeapValue::Function(JsFunction {
            name: Some("reject".into()),
            params: vec!["reason".into()],
            bytecode_index: usize::MAX,
            closure: vec![Value::Promise(promise_idx)],
            prototype: Some(proto_idx),
            super_class: None,
            properties: HashMap::new(),
        }));
        Value::Function(heap_idx)
    }

    pub(crate) fn resolve_promise(&mut self, promise_idx: usize, value: Value) {
        if let HeapValue::Promise(promise) = &mut self.heap[promise_idx] {
            if promise.state == PromiseState::Pending {
                promise.state = PromiseState::Fulfilled(value.clone());
                let handlers: Vec<Value> = promise.then_handlers.iter()
                    .map(|h| Value::Function(h.callback))
                    .collect();
                promise.then_handlers.clear();
                for handler in handlers {
                    self.async_runtime.enqueue_microtask_with_arg(handler, value.clone());
                }
                let finally_handlers: Vec<Value> = promise.finally_handlers.iter()
                    .map(|h| Value::Function(h.callback))
                    .collect();
                promise.finally_handlers.clear();
                for handler in finally_handlers {
                    self.async_runtime.enqueue_microtask(handler);
                }
            }
        }
    }

    pub(crate) fn reject_promise(&mut self, promise_idx: usize, reason: Value) {
        if let HeapValue::Promise(promise) = &mut self.heap[promise_idx] {
            if promise.state == PromiseState::Pending {
                promise.state = PromiseState::Rejected(reason.clone());
                let handlers: Vec<Value> = promise.catch_handlers.iter()
                    .map(|h| Value::Function(h.callback))
                    .collect();
                promise.catch_handlers.clear();
                for handler in handlers {
                    self.async_runtime.enqueue_microtask_with_arg(handler, reason.clone());
                }
                let finally_handlers: Vec<Value> = promise.finally_handlers.iter()
                    .map(|h| Value::Function(h.callback))
                    .collect();
                promise.finally_handlers.clear();
                for handler in finally_handlers {
                    self.async_runtime.enqueue_microtask(handler);
                }
            }
        }
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new().expect("Failed to create default interpreter")
    }
}
