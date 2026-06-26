mod heap_types;
mod call_frame;
mod builtins;
mod property_access;
mod value_ops;
mod promise_runtime;
mod instructions;
mod modules;
mod calls;

pub use heap_types::{JsObject, JsArray, JsFunction, HeapValue, JsProxyData};
pub(crate) use call_frame::{CallFrame, ExceptionHandler};

use std::collections::HashMap;
use std::rc::Rc;
use crate::compiler::{CompiledModule, Instruction};
use crate::errors::{Error, Result};
use crate::objects::Value;
use crate::runtime_env::async_runtime::AsyncRuntime;
use crate::objects::js_promise::PromiseState;

pub struct Interpreter {
    pub(crate) globals: HashMap<String, Value>,
    pub(crate) stack: Vec<Value>,
    pub(crate) heap: Vec<HeapValue>,
    pub(crate) gc: crate::vm::gc::GarbageCollector,
    pub(crate) call_stack: Vec<CallFrame>,
    pub(crate) current_module: Option<Rc<CompiledModule>>,
    exception_handlers: Vec<ExceptionHandler>,
    pending_exception: Option<Value>,
    pub(crate) async_runtime: AsyncRuntime,
    pub(crate) promise_stack: Vec<usize>,
    timer_id_counter: u32,
    pub(crate) module_registry: HashMap<String, HashMap<String, Value>>,
    pub(crate) module_exports: HashMap<String, Value>,
    pub(crate) current_module_path: Option<String>,
    pub(crate) module_globals: Option<HashMap<String, Value>>,
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
            module_globals: None,
            current_module_path: None,
        };
        interp.init_builtins();
        Ok(interp)
    }

    pub fn execute(&mut self, module: &CompiledModule) -> Result<Value> {
        self.current_module = Some(Rc::new(module.clone()));
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

        loop {
            if pc >= module.instructions.len() {
                break;
            }

            if self.gc.should_collect() {
                self.collect_garbage();
            }

            let instruction = module.instructions[pc].clone();

            match &instruction {
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
                Instruction::Call(argc) => {
                    let callee = self.stack.pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let mut args = Vec::new();
                    for _ in 0..*argc {
                        args.push(self.stack.pop()
                            .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?);
                    }
                    args.reverse();
                    match &callee {
                        Value::Function(func_idx) => {
                            if let HeapValue::Function(f) = &self.heap[*func_idx] {
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
                                let same_module = match (&f.owner_module, &self.current_module) {
                                    (Some(om), Some(cm)) => Rc::ptr_eq(om, cm),
                                    (None, None) => true,
                                    _ => false,
                                };
                                if same_module {
                                    let func = self.heap[*func_idx].clone();
                                    if let HeapValue::Function(f) = func {
                                        let return_address = pc + 1;
                                        let base_pointer = self.stack.len();
                                        let closure_count = f.closure.len();
                                        self.call_stack.push(CallFrame {
                                            return_address,
                                            base_pointer,
                                            closure_var_count: closure_count,
                                            func_heap_idx: Some(*func_idx),
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
                            }
                            let result = self.call_value(&callee, &Value::Undefined, &args)?;
                            self.stack.push(result);
                        }
                        Value::NativeFunction(native_idx) => {
                            let result = self.call_native(*native_idx, &Value::Undefined, &args)?;
                            self.stack.push(result);
                        }
                        Value::Proxy(proxy_idx) => {
                            if let HeapValue::Proxy(proxy) = &self.heap[*proxy_idx] {
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
                                let same_module = match (&f_clone.owner_module, &self.current_module) {
                                    (Some(om), Some(cm)) => Rc::ptr_eq(om, cm),
                                    (None, None) => true,
                                    _ => false,
                                };
                                if same_module {
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
                                let result = self.call_value(&method, &object, &args)?;
                                self.stack.push(result);
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
                                    let same_module = match (&f_clone.owner_module, &self.current_module) {
                                        (Some(om), Some(cm)) => Rc::ptr_eq(om, cm),
                                        (None, None) => true,
                                        _ => false,
                                    };
                                    if same_module {
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
                                    let result = self.call_value(&constructor, &this_val, &args)?;
                                    match result {
                                        Value::Object(_) | Value::Array(_) | Value::Function(_) | Value::Promise(_) | Value::Proxy(_) => {
                                            self.stack.push(result);
                                        }
                                        _ => {
                                            self.stack.push(this_val);
                                        }
                                    }
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
                Instruction::MakeClass(class_info_idx) => {
                    self.handle_make_class(class_info_idx, module)?;
                }
                Instruction::SuperConstruct(argc) => {
                    self.handle_super_construct(*argc, pc, module)?;
                    // Note: handle_super_construct may modify pc, but since we can't
                    // easily propagate that here, we handle it inline
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
                    let _this = self.stack.pop()
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
                    let module_path = match self.resolve_module_path(source) {
                        Ok(p) => p,
                        Err(_) => {
                            self.stack.push(Value::Undefined);
                            pc += 1;
                            continue;
                        }
                    };
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
                    let module_path = match self.resolve_module_path(source) {
                        Ok(p) => p,
                        Err(_) => {
                            self.globals.insert(local_name.clone(), Value::Undefined);
                            pc += 1;
                            continue;
                        }
                    };
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
                    let module_path = match self.resolve_module_path(source) {
                        Ok(p) => p,
                        Err(_) => {
                            self.globals.insert(local_name.clone(), Value::Undefined);
                            pc += 1;
                            continue;
                        }
                    };
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
                    let module_path = match self.resolve_module_path(source) {
                        Ok(p) => p,
                        Err(_) => {
                            let heap_idx = self.heap.len();
                            self.heap.push(HeapValue::Object(JsObject::new()));
                            self.globals.insert(local_name.clone(), Value::Object(heap_idx));
                            pc += 1;
                            continue;
                        }
                    };
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
                    // Delegate to instruction handler modules
                    if self.exec_load_store(&instruction, module)? {
                        // handled
                    } else if self.exec_arithmetic(&instruction)? {
                        // handled
                    } else if self.exec_comparison(&instruction)? {
                        // handled
                    } else if self.exec_property_ops(&instruction)? {
                        // handled
                    } else if self.exec_make_function(&instruction, module)? {
                        // handled
                    } else if self.exec_exception(&instruction, &mut pc)? {
                        if pc != start_pc + 1 {
                            continue;
                        }
                    } else {
                        return Err(Error::RuntimeError(format!("Unhandled instruction: {:?}", instruction)));
                    }
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

    fn handle_make_class(&mut self, class_info_idx: &u32, module: &CompiledModule) -> Result<()> {
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
            let owner = self.current_module.clone();
            let idx = self.gc.allocate(&mut self.heap, HeapValue::Function(JsFunction {
                name: func_info.name,
                params: func_info.params,
                bytecode_index: func_info.bytecode_index,
                closure: Vec::new(),
                prototype: Some(proto_obj_idx),
                super_class: Some(super_val.clone()),
                properties: HashMap::new(),
                owner_module: owner,
                module_scope: None,
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
                module_scope: None,
            }));
            idx
        };
        if let HeapValue::Object(proto_obj) = &mut self.heap[proto_obj_idx] {
            proto_obj.properties.insert("constructor".to_string(), Value::Function(ctor_heap_idx));
        }
        for method_info in &class_info.methods {
            let method_func_info = module.functions[method_info.func_idx as usize].clone();
            let method_proto_idx = self.gc.allocate(&mut self.heap, HeapValue::Object(JsObject::new()));
            let owner = self.current_module.clone();
            let method_heap_idx = self.gc.allocate(&mut self.heap, HeapValue::Function(JsFunction {
                name: Some(method_info.name.clone()),
                params: method_func_info.params,
                bytecode_index: method_func_info.bytecode_index,
                closure: Vec::new(),
                prototype: Some(method_proto_idx),
                super_class: None,
                properties: HashMap::new(),
                owner_module: owner,
                module_scope: None,
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
        Ok(())
    }

    fn handle_super_construct(&mut self, _argc: u16, _pc: usize, _module: &CompiledModule) -> Result<()> {
        // This is handled inline in execute_from() due to pc control flow
        Ok(())
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new().expect("Failed to create default interpreter")
    }
}
