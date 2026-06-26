mod heap_types;
mod call_frame;
mod builtins;
mod property_access;
mod value_ops;
mod promise_runtime;

pub use heap_types::{JsObject, JsArray, JsFunction, HeapValue, JsProxyData};
pub(crate) use call_frame::{CallFrame, ExceptionHandler};

use std::collections::HashMap;
use crate::compiler::{CompiledModule, Instruction};
use crate::errors::{Error, Result};
use crate::objects::Value;
use crate::runtime_env::native_fns::NATIVE_TABLE;
use crate::runtime_env::async_runtime::AsyncRuntime;
use crate::objects::js_promise::{JsPromise, PromiseState};

pub struct Interpreter {
    pub(crate) globals: HashMap<String, Value>,
    pub(crate) stack: Vec<Value>,
    pub(crate) heap: Vec<HeapValue>,
    pub(crate) gc: crate::vm::gc::GarbageCollector,
    pub(crate) call_stack: Vec<CallFrame>,
    pub(crate) current_module: Option<CompiledModule>,
    exception_handlers: Vec<ExceptionHandler>,
    pending_exception: Option<Value>,
    pub(crate) async_runtime: AsyncRuntime,
    pub(crate) promise_stack: Vec<usize>,
    timer_id_counter: u32,
    pub(crate) module_registry: HashMap<String, HashMap<String, Value>>,
    pub(crate) module_exports: HashMap<String, Value>,
    pub(crate) current_module_path: Option<String>,
}

impl Interpreter {
    pub fn new() -> Result<Self> {
        let mut interp = Self {
            globals: HashMap::new(),
            stack: Vec::new(),
            heap: Vec::new(),
            gc: crate::vm::gc::GarbageCollector::new(),
            call_stack: Vec::new(),
            current_module: None,
            exception_handlers: Vec::new(),
            pending_exception: None,
            async_runtime: AsyncRuntime::new(),
            promise_stack: Vec::new(),
            timer_id_counter: 1,
            module_registry: HashMap::new(),
            module_exports: HashMap::new(),
            current_module_path: None,
        };
        interp.init_builtins();
        Ok(interp)
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

    pub(crate) fn collect_garbage(&mut self) {
        let globals_snapshot = self.globals.clone();
        let stack_snapshot = self.stack.clone();
        let call_stack_snapshot = self.call_stack.clone();
        self.gc.collect(&mut self.heap, &globals_snapshot, &stack_snapshot, &call_stack_snapshot);
    }

    pub(crate) fn execute_from(&mut self, module: &CompiledModule, start_pc: usize) -> Result<Value> {
        let mut pc = start_pc;

        'main: loop {
            if pc >= module.instructions.len() {
                break;
            }

            if self.gc.should_collect() {
                self.collect_garbage();
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
                                        pc += 1;
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
                        Value::Proxy(proxy_idx) => {
                            if let HeapValue::Proxy(proxy) = &self.heap[proxy_idx] {
                                let handler = proxy.handler.clone();
                                let target = proxy.target.clone();
                                let arr_idx = self.gc.allocate(&mut self.heap, HeapValue::Array(JsArray { elements: args }));
                                let trap_result = self.call_proxy_trap(&handler, "apply", &[target, Value::Undefined, Value::Array(arr_idx)]);
                                match trap_result {
                                    Ok(v) => self.stack.push(v),
                                    Err(e) => return Err(e),
                                }
                            } else {
                                return Err(Error::TypeError(format!("{} is not a function", self.value_to_string(&callee))));
                            }
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

                            let new_obj_heap_idx = self.gc.allocate(&mut self.heap, HeapValue::Object(JsObject::with_prototype(proto_idx)));
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
                            let new_obj_heap_idx = self.gc.allocate(&mut self.heap, HeapValue::Object(JsObject::with_prototype(proto_idx)));
                            let this_val = Value::Object(new_obj_heap_idx);
                            let result = self.call_native(*native_idx, &this_val, &args)?;
                            match result {
                                Value::Object(_) | Value::Array(_) | Value::Function(_) | Value::Promise(_) | Value::Proxy(_) => {
                                    self.stack.push(result);
                                }
                                _ => {
                                    self.stack.push(this_val);
                                }
                            }
                        }
                        Value::Proxy(proxy_idx) => {
                            if let HeapValue::Proxy(proxy) = &self.heap[*proxy_idx] {
                                let handler = proxy.handler.clone();
                                let target = proxy.target.clone();
                                let args_arr_idx = self.gc.allocate(&mut self.heap, HeapValue::Array(JsArray { elements: args }));
                                let trap_result = self.call_proxy_trap(&handler, "construct", &[target, Value::Array(args_arr_idx), constructor.clone()]);
                                match trap_result {
                                    Ok(v) => self.stack.push(v),
                                    Err(e) => return Err(e),
                                }
                            } else {
                                return Err(Error::TypeError(format!("{} is not a constructor", self.value_to_string(&constructor))));
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

                    let proto_obj_idx = self.gc.allocate(&mut self.heap, HeapValue::Object(JsObject::new()));

                    let owner = self.current_module.as_ref().map(|m| std::rc::Rc::new(m.clone()));
                    let heap_idx = self.gc.allocate(&mut self.heap, HeapValue::Function(JsFunction {
                        name: func_info.name,
                        params: func_info.params,
                        bytecode_index: func_info.bytecode_index,
                        closure: Vec::new(),
                        prototype: Some(proto_obj_idx),
                        super_class: None,
                        properties: HashMap::new(),
                        owner_module: owner,
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

                    let proto_obj_idx = self.gc.allocate(&mut self.heap, HeapValue::Object(JsObject::new()));

                    let owner = self.current_module.as_ref().map(|m| std::rc::Rc::new(m.clone()));
                    let heap_idx = self.gc.allocate(&mut self.heap, HeapValue::Function(JsFunction {
                        name: func_info.name,
                        params: func_info.params,
                        bytecode_index: func_info.bytecode_index,
                        closure: closure_vars,
                        prototype: Some(proto_obj_idx),
                        super_class: None,
                        properties: HashMap::new(),
                        owner_module: owner,
                    }));

                    self.stack.push(Value::Function(heap_idx));
                }
                Instruction::NewObject => {
                    let heap_idx = self.gc.allocate(&mut self.heap, HeapValue::Object(JsObject::new()));
                    self.stack.push(Value::Object(heap_idx));
                }
                Instruction::SetProperty => {
                    let value = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let key = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let object = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;

                    match &object {
                        Value::Proxy(proxy_idx) => {
                            if let HeapValue::Proxy(proxy) = &self.heap[*proxy_idx] {
                                let handler = proxy.handler.clone();
                                let target = proxy.target.clone();
                                let trap = self.get_property(&handler, &Value::String("set".to_string()));
                                if let Ok(Value::Function(_)) | Ok(Value::NativeFunction(_)) = &trap {
                                    let trap_result = self.call_value(&trap?, &handler, &[target, key.clone(), value, object.clone()]);
                                    if let Err(e) = trap_result {
                                        return Err(e);
                                    }
                                } else {
                                    if let Value::Object(target_obj_idx) = &target {
                                        if let HeapValue::Object(obj) = &mut self.heap[*target_obj_idx] {
                                            if let Value::String(key_str) = &key {
                                                obj.properties.insert(key_str.clone(), value);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Value::Object(obj_idx) => {
                            if let HeapValue::Object(obj) = &mut self.heap[*obj_idx] {
                                if let Value::String(key_str) = &key {
                                    obj.properties.insert(key_str.clone(), value);
                                }
                            }
                        }
                        Value::Function(func_idx) => {
                            if let HeapValue::Function(f) = &mut self.heap[*func_idx] {
                                if let Value::String(key_str) = &key {
                                    f.properties.insert(key_str.clone(), value);
                                }
                            }
                        }
                        _ => {}
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
                    let heap_idx = self.gc.allocate(&mut self.heap, HeapValue::Array(JsArray { elements }));
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
                        Value::Object(_) | Value::Array(_) | Value::Promise(_) | Value::Proxy(_) => "object",
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

                    match &object {
                        Value::Proxy(proxy_idx) => {
                            if let HeapValue::Proxy(proxy) = &self.heap[*proxy_idx] {
                                let handler = proxy.handler.clone();
                                let target = proxy.target.clone();
                                let trap = self.get_property(&handler, &Value::String("deleteProperty".to_string()));
                                if let Ok(Value::Function(_)) | Ok(Value::NativeFunction(_)) = &trap {
                                    let trap_result = self.call_value(&trap?, &handler, &[target, key.clone()]);
                                    match trap_result {
                                        Ok(v) => self.stack.push(v),
                                        Err(e) => return Err(e),
                                    }
                                } else {
                                    let result = self.delete_property(&target, &key);
                                    self.stack.push(result);
                                }
                            } else {
                                let result = self.delete_property(&object, &key);
                                self.stack.push(result);
                            }
                        }
                        _ => {
                            let result = self.delete_property(&object, &key);
                            self.stack.push(result);
                        }
                    }
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

                    let result = self.in_check_mut(&left, &right)?;
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

                    let proto_obj_idx = self.gc.allocate(&mut self.heap, HeapValue::Object(JsObject::new()));

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
                        let owner = self.current_module.as_ref().map(|m| std::rc::Rc::new(m.clone()));

                        let idx = self.gc.allocate(&mut self.heap, HeapValue::Function(JsFunction {
                            name: func_info.name,
                            params: func_info.params,
                            bytecode_index: func_info.bytecode_index,
                            closure: Vec::new(),
                            prototype: Some(proto_obj_idx),
                            super_class: Some(super_val.clone()),
                            properties: HashMap::new(),
                            owner_module: owner,
                        }));
                        idx
                    } else {
                        let idx = self.gc.allocate(&mut self.heap, HeapValue::Function(JsFunction {
                            name: Some(class_info.name.clone()),
                            params: Vec::new(),
                            bytecode_index: usize::MAX,
                            closure: Vec::new(),
                            prototype: Some(proto_obj_idx),
                            super_class: Some(super_val.clone()),
                            properties: HashMap::new(),
                            owner_module: None,
                        }));
                        idx
                    };

                    if let HeapValue::Object(proto_obj) = &mut self.heap[proto_obj_idx] {
                        proto_obj.properties.insert("constructor".to_string(), Value::Function(ctor_heap_idx));
                    }

                    for method_info in &class_info.methods {
                        let method_func_info = module.functions[method_info.func_idx as usize].clone();

                        let method_proto_idx = self.gc.allocate(&mut self.heap, HeapValue::Object(JsObject::new()));
                        let owner = self.current_module.as_ref().map(|m| std::rc::Rc::new(m.clone()));

                        let method_heap_idx = self.gc.allocate(&mut self.heap, HeapValue::Function(JsFunction {
                            name: Some(method_info.name.clone()),
                            params: method_func_info.params,
                            bytecode_index: method_func_info.bytecode_index,
                            closure: Vec::new(),
                            prototype: Some(method_proto_idx),
                            super_class: None,
                            properties: HashMap::new(),
                            owner_module: owner,
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

                                let proto_val = self.get_property(&super_class, &Value::String("prototype".to_string()))?;
                                let proto_idx = if let Value::Object(pi) = proto_val { Some(pi) } else { None };
                                let new_obj_heap_idx = self.gc.allocate(&mut self.heap, HeapValue::Object(JsObject::with_prototype(proto_idx)));

                                let _constructed = Value::Object(new_obj_heap_idx);

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
                    let this = self.stack.pop()
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

                    if let Value::Function(func_idx) = &super_class {
                        if let HeapValue::Function(f) = &self.heap[*func_idx] {
                            if let Some(proto_idx) = f.prototype {
                                let proto_val = Value::Object(proto_idx);
                                let result = self.get_property(&proto_val, &key)?;
                                self.stack.push(result);
                                continue;
                            }
                        }
                    }
                    self.stack.push(Value::Undefined);
                }
                Instruction::ImportModule(source) => {
                    let module_path = self.resolve_module_path(source)?;
                    if let Some(exports) = self.module_registry.get(&module_path).cloned() {
                        let heap_idx = self.heap.len();
                        let mut props = HashMap::new();
                        for (k, v) in &exports {
                            props.insert(k.clone(), v.clone());
                        }
                        self.heap.push(HeapValue::Object(JsObject { properties: props, prototype: None }));
                        self.stack.push(Value::Object(heap_idx));
                    } else {
                        let source_code = std::fs::read_to_string(&module_path)
                            .map_err(|e| Error::RuntimeError(format!("Cannot read module '{}': {}", source, e)))?;
                        let compiler = crate::compiler::Compiler::new(false);
                        let compiled = compiler.compile(&source_code)?;
                        let prev_path = self.current_module_path.take();
                        self.current_module_path = Some(module_path.clone());
                        self.module_registry.insert(module_path.clone(), HashMap::new());
                        let _result = self.execute_module(&compiled)?;
                        let exports = std::mem::take(&mut self.module_exports);
                        *self.module_registry.entry(module_path.clone()).or_default() = exports;
                        self.current_module_path = prev_path;
                        let heap_idx = self.heap.len();
                        let mut props = HashMap::new();
                        if let Some(registry_exports) = self.module_registry.get(&module_path) {
                            for (k, v) in registry_exports {
                                props.insert(k.clone(), v.clone());
                            }
                        }
                        self.heap.push(HeapValue::Object(JsObject { properties: props, prototype: None }));
                        self.stack.push(Value::Object(heap_idx));
                    }
                }
                Instruction::ImportNamed(source, imported_name, local_name) => {
                    let module_path = self.resolve_module_path(source)?;
                    if !self.module_registry.contains_key(&module_path) {
                        let source_code = std::fs::read_to_string(&module_path)
                            .map_err(|e| Error::RuntimeError(format!("Cannot read module '{}': {}", source, e)))?;
                        let compiler = crate::compiler::Compiler::new(false);
                        let compiled = compiler.compile(&source_code)?;
                        let prev_path = self.current_module_path.take();
                        self.current_module_path = Some(module_path.clone());
                        self.module_registry.insert(module_path.clone(), HashMap::new());
                        let _result = self.execute_module(&compiled)?;
                        let exports = std::mem::take(&mut self.module_exports);
                        *self.module_registry.entry(module_path.clone()).or_default() = exports;
                        self.current_module_path = prev_path;
                    }
                    if let Some(exports) = self.module_registry.get(&module_path) {
                        let val = exports.get(imported_name).cloned().unwrap_or(Value::Undefined);
                        self.globals.insert(local_name.clone(), val);
                    }
                }
                Instruction::ImportDefault(source, local_name) => {
                    let module_path = self.resolve_module_path(source)?;
                    if !self.module_registry.contains_key(&module_path) {
                        let source_code = std::fs::read_to_string(&module_path)
                            .map_err(|e| Error::RuntimeError(format!("Cannot read module '{}': {}", source, e)))?;
                        let compiler = crate::compiler::Compiler::new(false);
                        let compiled = compiler.compile(&source_code)?;
                        let prev_path = self.current_module_path.take();
                        self.current_module_path = Some(module_path.clone());
                        self.module_registry.insert(module_path.clone(), HashMap::new());
                        let _result = self.execute_module(&compiled)?;
                        let exports = std::mem::take(&mut self.module_exports);
                        *self.module_registry.entry(module_path.clone()).or_default() = exports;
                        self.current_module_path = prev_path;
                    }
                    if let Some(exports) = self.module_registry.get(&module_path) {
                        let val = exports.get("default").cloned().unwrap_or(Value::Undefined);
                        self.globals.insert(local_name.clone(), val);
                    }
                }
                Instruction::ImportAll(source, local_name) => {
                    let module_path = self.resolve_module_path(source)?;
                    if !self.module_registry.contains_key(&module_path) {
                        let source_code = std::fs::read_to_string(&module_path)
                            .map_err(|e| Error::RuntimeError(format!("Cannot read module '{}': {}", source, e)))?;
                        let compiler = crate::compiler::Compiler::new(false);
                        let compiled = compiler.compile(&source_code)?;
                        let prev_path = self.current_module_path.take();
                        self.current_module_path = Some(module_path.clone());
                        self.module_registry.insert(module_path.clone(), HashMap::new());
                        let _result = self.execute_module(&compiled)?;
                        let exports = std::mem::take(&mut self.module_exports);
                        *self.module_registry.entry(module_path.clone()).or_default() = exports;
                        self.current_module_path = prev_path;
                    }
                    if let Some(exports) = self.module_registry.get(&module_path) {
                        let heap_idx = self.heap.len();
                        let mut props = HashMap::new();
                        for (k, v) in exports {
                            props.insert(k.clone(), v.clone());
                        }
                        self.heap.push(HeapValue::Object(JsObject { properties: props, prototype: None }));
                        self.globals.insert(local_name.clone(), Value::Object(heap_idx));
                    }
                }
                Instruction::ExportNamed(names) => {
                    for name in names {
                        if let Some(val) = self.globals.get(name) {
                            self.module_exports.insert(name.clone(), val.clone());
                        }
                    }
                }
                Instruction::ExportDefault => {
                    let val = self.stack.last().cloned().unwrap_or(Value::Undefined);
                    self.module_exports.insert("default".to_string(), val);
                }
                Instruction::StoreModuleExport(name) => {
                    if let Some(val) = self.globals.get(name) {
                        self.module_exports.insert(name.clone(), val.clone());
                    }
                }
                Instruction::PopModuleExports => {
                    self.module_exports.clear();
                }
                Instruction::Await => {
                    let value = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    if let Value::Promise(promise_idx) = &value {
                        if let HeapValue::Promise(p) = &self.heap[*promise_idx] {
                            match &p.state {
                                PromiseState::Fulfilled(v) => {
                                    self.stack.push(v.clone());
                                }
                                PromiseState::Rejected(_r) => {
                                    self.stack.push(Value::Undefined);
                                }
                                PromiseState::Pending => {
                                    self.stack.push(value);
                                }
                            }
                        } else {
                            self.stack.push(value);
                        }
                    } else {
                        self.stack.push(value);
                    }
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

    pub fn execute_module(&mut self, module: &CompiledModule) -> Result<Value> {
        let saved_module = self.current_module.take();
        self.current_module = Some(module.clone());
        let prev_exports = std::mem::take(&mut self.module_exports);
        let pre_keys: std::collections::HashSet<String> = self.globals.keys().cloned().collect();
        let result = self.execute(module);
        let post_keys: std::collections::HashSet<String> = self.globals.keys().cloned().collect();
        let export_keys: std::collections::HashSet<String> = self.module_exports.keys().cloned().collect();
        for key in post_keys.difference(&pre_keys) {
            if !export_keys.contains(key) {
                self.globals.remove(key);
            }
        }
        let exec_exports = std::mem::replace(&mut self.module_exports, prev_exports);
        for (k, v) in exec_exports {
            self.module_exports.insert(k, v);
        }
        self.current_module = saved_module;
        result
    }

    fn resolve_local_from_stack(&self, _name: &str) -> Option<usize> {
        None
    }

    fn load_and_run_module(&mut self, source: &str) -> Result<Option<String>> {
        let module_path = match self.resolve_module_path(source) {
            Ok(p) => p,
            Err(_) => return Ok(None),
        };
        if self.module_registry.contains_key(&module_path) {
            return Ok(Some(module_path));
        }
        let source_code = match std::fs::read_to_string(&module_path) {
            Ok(s) => s,
            Err(_) => return Ok(None),
        };
        let compiler = crate::compiler::Compiler::new(false);
        let compiled = compiler.compile(&source_code)?;
        let prev_path = self.current_module_path.take();
        self.current_module_path = Some(module_path.clone());
        self.module_registry.insert(module_path.clone(), HashMap::new());
        let result = self.execute_module(&compiled);
        let exports = std::mem::take(&mut self.module_exports);
        *self.module_registry.entry(module_path.clone()).or_default() = exports;
        self.current_module_path = prev_path;
        result?;
        Ok(Some(module_path))
    }

    fn resolve_module_path(&self, source: &str) -> Result<String> {
        let base = self.current_module_path.as_deref().unwrap_or(".");
        let base_path = std::path::Path::new(base);
        let parent = base_path.parent().unwrap_or(std::path::Path::new("."));
        let resolved = if source.starts_with("./") || source.starts_with("../") {
            parent.join(source)
        } else {
            std::path::PathBuf::from(source)
        };
        if resolved.exists() && resolved.is_file() {
            return Ok(resolved.to_string_lossy().to_string());
        }
        for ext in &[".ts", ".js"] {
            let stem = resolved.with_extension("");
            let candidate = std::path::PathBuf::from(format!("{}{}", stem.to_string_lossy(), ext));
            if candidate.exists() {
                return Ok(candidate.to_string_lossy().to_string());
            }
        }
        if resolved.is_dir() {
            for name in &["index.ts", "index.js"] {
                let idx = resolved.join(name);
                if idx.exists() {
                    return Ok(idx.to_string_lossy().to_string());
                }
            }
        }
        Err(Error::RuntimeError(format!("Module '{}' not found", source)))
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
            Value::Proxy(proxy_idx) => {
                if let HeapValue::Proxy(proxy) = &self.heap[*proxy_idx] {
                    let handler = proxy.handler.clone();
                    let target = proxy.target.clone();
                    let arr_idx = self.gc.allocate(&mut self.heap, HeapValue::Array(JsArray { elements: args.to_vec() }));
                    self.call_proxy_trap(&handler, "apply", &[target, this.clone(), Value::Array(arr_idx)])
                } else {
                    Err(Error::TypeError(format!("{} is not a function", self.value_to_string(callee))))
                }
            }
            _ => Err(Error::TypeError(format!("{} is not a function", self.value_to_string(callee)))),
        }
    }

    pub(crate) fn call_native(&mut self, idx: usize, this: &Value, args: &[Value]) -> Result<Value> {
        if idx < NATIVE_TABLE.len() {
            NATIVE_TABLE[idx](self, this, args)
        } else {
            Err(Error::RuntimeError(format!("Unknown native function index: {}", idx)))
        }
    }

    pub(crate) fn find_native_prototype(&self, native_idx: usize) -> Option<usize> {
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
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new().expect("Failed to create default interpreter")
    }
}
