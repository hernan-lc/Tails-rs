mod builtins;
mod call_frame;
mod calls;
mod heap_types;
mod instructions;
pub(crate) mod modules;
mod native_loader;
mod promise_runtime;
mod property_access;
mod value_ops;

pub(crate) use call_frame::{CallFrame, ExceptionHandler};
pub use heap_types::{
    HeapValue, JsArray, JsFunction, JsGenerator, JsObject, JsProxyData, JsRegExp,
};

use crate::compiler::{CompiledModule, Instruction};
use crate::errors::{Error, Result};
use crate::objects::js_promise::PromiseState;
use crate::objects::Value;
use crate::runtime_env::async_runtime::AsyncRuntime;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

#[derive(Clone)]
pub(crate) struct SuspendedFrame {
    pub(crate) promise_idx: usize,
    pub(crate) resume_pc: usize,
    pub(crate) stack_snapshot: Vec<Value>,
    pub(crate) call_stack_snapshot: Vec<CallFrame>,
    pub(crate) module: Option<Rc<CompiledModule>>,
    pub(crate) module_path: Option<String>,
    pub(crate) exception_handlers_snapshot: Vec<ExceptionHandler>,
    pub(crate) block_scope_stack_snapshot: Vec<usize>,
}

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
    pub(crate) _promise_stack: Vec<usize>,
    _timer_id_counter: u32,
    pub(crate) module_registry: HashMap<String, HashMap<String, Value>>,
    pub(crate) module_exports: HashMap<String, Value>,
    pub(crate) current_module_path: Option<String>,
    pub(crate) module_globals: Option<HashMap<String, Value>>,
    pub(crate) block_scope_stack: Vec<usize>,
    pub(crate) next_symbol_id: u64,
    pub(crate) symbol_registry: HashMap<String, u64>,
    pub(crate) date_proto_idx: Option<usize>,
    pub(crate) regexp_proto_idx: Option<usize>,
    pub(crate) buffer_proto_idx: Option<usize>,
    pub(crate) generator_proto_idx: Option<usize>,
    pub(crate) native_loader: native_loader::NativeModuleRegistry,
    pub(crate) current_pc: usize,
    pub(crate) suspended_frames: VecDeque<SuspendedFrame>,
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
            _promise_stack: Vec::new(),
            _timer_id_counter: 1,
            module_registry: HashMap::new(),
            module_exports: HashMap::new(),
            module_globals: None,
            current_module_path: None,
            block_scope_stack: Vec::new(),
            next_symbol_id: crate::objects::USER_SYMBOL_START,
            symbol_registry: HashMap::new(),
            date_proto_idx: None,
            regexp_proto_idx: None,
            buffer_proto_idx: None,
            generator_proto_idx: None,
            native_loader: native_loader::NativeModuleRegistry::new(),
            current_pc: 0,
            suspended_frames: VecDeque::new(),
        };
        interp.init_builtins();
        interp.init_builtins();
        Ok(interp)
    }

    pub fn execute(&mut self, module: &CompiledModule) -> Result<Value> {
        self.current_module = Some(Rc::new(module.clone()));
        let mut result = self.execute_from(module, 0);

        loop {
            self.drain_microtasks();

            let mut any_resumed = false;
            let mut i = 0;
            while i < self.suspended_frames.len() {
                let promise_idx = self.suspended_frames[i].promise_idx;
                let should_resume = if let HeapValue::Promise(p) = &self.heap[promise_idx] {
                    matches!(
                        p.state,
                        PromiseState::Fulfilled(_) | PromiseState::Rejected(_)
                    )
                } else {
                    false
                };

                if should_resume {
                    let frame = self.suspended_frames.remove(i).unwrap();
                    let resolved_value =
                        if let HeapValue::Promise(p) = &self.heap[frame.promise_idx] {
                            match &p.state {
                                PromiseState::Fulfilled(v) => v.clone(),
                                PromiseState::Rejected(reason) => {
                                    return Err(self.err_at_location(Error::RuntimeError(
                                        format!("Unhandled promise rejection: {:?}", reason),
                                    )));
                                }
                                _ => unreachable!(),
                            }
                        } else {
                            Value::Undefined
                        };

                    self.stack = frame.stack_snapshot;
                    self.stack.push(resolved_value);
                    self.call_stack = frame.call_stack_snapshot;
                    self.current_module = frame.module;
                    self.current_module_path = frame.module_path;
                    self.exception_handlers = frame.exception_handlers_snapshot;
                    self.block_scope_stack = frame.block_scope_stack_snapshot;

                    let module_ref = self.current_module.clone().unwrap();
                    result = self.execute_from(&module_ref, frame.resume_pc);
                    any_resumed = true;
                } else {
                    i += 1;
                }
            }

            let macrotasks: Vec<_> = self.async_runtime.run_macrotasks();
            for task in macrotasks {
                let _ = self.call_value(&task.callback, &Value::Undefined, &[]);
            }

            if !any_resumed && self.suspended_frames.is_empty() && self.async_runtime.is_idle() {
                break;
            }
        }

        result
    }

    pub(crate) fn collect_garbage(&mut self) {
        let globals_snapshot = self.globals.clone();
        let stack_snapshot = self.stack.clone();
        let call_stack_snapshot = self.call_stack.clone();
        self.gc.collect(
            &mut self.heap,
            &globals_snapshot,
            &stack_snapshot,
            &call_stack_snapshot,
        );
    }

    pub(crate) fn current_source_line(&self, pc: usize) -> Option<usize> {
        self.current_module
            .as_ref()
            .and_then(|m| m.source_lines.get(pc).copied().flatten())
    }

    pub(crate) fn err_at_location(&self, mut err: crate::errors::Error) -> crate::errors::Error {
        if err.span.is_some() {
            return err;
        }
        if let Some(line) = self.current_source_line(self.current_pc) {
            let file = self.current_module_path.clone();
            err.span = Some(crate::errors::Span::new(line, 1, 0));
            if err.file.is_none() {
                err.file = file;
            }
        }
        err
    }

    pub(crate) fn execute_from(
        &mut self,
        module: &CompiledModule,
        start_pc: usize,
    ) -> Result<Value> {
        let mut pc = start_pc;

        loop {
            if pc >= module.instructions.len() {
                break;
            }

            self.current_pc = pc;

            if self.gc.should_collect() {
                self.collect_garbage();
            }

            let instruction = module.instructions[pc].clone();

            if cfg!(debug_assertions) && std::env::var("GEN_TRACE").is_ok() {
                eprintln!("[GEN_TRACE] pc={}, instr={:?}", pc, instruction);
            }

            match &instruction {
                Instruction::Jump(target) => {
                    pc = *target as usize;
                    continue;
                }
                Instruction::JumpIf(target) => {
                    let value = self
                        .stack
                        .pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    if self.is_truthy(&value) {
                        pc = *target as usize;
                        continue;
                    }
                }
                Instruction::JumpIfNot(target) => {
                    let value = self
                        .stack
                        .pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    if !self.is_truthy(&value) {
                        pc = *target as usize;
                        continue;
                    }
                }
                Instruction::JumpIfUndefined(target) => {
                    let value = self
                        .stack
                        .pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    if matches!(value, Value::Undefined | Value::Null) {
                        pc = *target as usize;
                        continue;
                    }
                }
                Instruction::JumpIfNotUndefined(target) => {
                    let value = self
                        .stack
                        .last()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    if !matches!(value, Value::Undefined | Value::Null) {
                        pc = *target as usize;
                        continue;
                    }
                }
                Instruction::Return => {
                    let return_value = self.stack.pop().unwrap_or(Value::Undefined);
                    if let Some(frame) = self.call_stack.pop() {
                        if frame.closure_var_count > 0 {
                            if let Some(heap_idx) = frame.func_heap_idx {
                                if let HeapValue::Function(f) = &mut self.heap[heap_idx] {
                                    f.closure.clear();
                                    for i in 0..frame.closure_var_count {
                                        let val = self
                                            .stack
                                            .get(frame.base_pointer + i)
                                            .cloned()
                                            .unwrap_or(Value::Undefined);
                                        f.closure.push(val);
                                    }
                                }
                            }
                        }
                        let final_value = if frame.is_construct {
                            match &return_value {
                                Value::Object(_) | Value::Array(_) | Value::Function(_) => {
                                    return_value
                                }
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
                Instruction::Yield => {
                    let yield_value = self.stack.pop().unwrap_or(Value::Undefined);
                    if let Some(frame) = self.call_stack.last() {
                        let saved_pc = pc + 1;
                        let saved_stack: Vec<Value> = self.stack[frame.base_pointer..].to_vec();
                        if let Some(gen_heap_idx) = frame.generator_heap_idx {
                            if let HeapValue::Generator(gen) = &mut self.heap[gen_heap_idx] {
                                gen.yield_value = yield_value.clone();
                                gen.resume_pc = saved_pc;
                                gen.saved_stack = saved_stack;
                                gen.saved_block_scope_stack = self.block_scope_stack.clone();
                                gen.func_heap_idx = frame.func_heap_idx;
                            }
                        }
                        return Ok(yield_value);
                    }
                }
                Instruction::Call(argc) => {
                    let callee = self
                        .stack
                        .pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let mut args = Vec::new();
                    for _ in 0..*argc {
                        args.push(
                            self.stack
                                .pop()
                                .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?,
                        );
                    }
                    args.reverse();
                    match &callee {
                        Value::Function(func_idx) => {
                            // Clone needed values before any heap mutation
                            let (is_generator, bytecode_index, has_promise_resolve) =
                                if let HeapValue::Function(f) = &self.heap[*func_idx] {
                                    let has_promise = f.bytecode_index == usize::MAX
                                        && f.closure
                                            .first()
                                            .is_some_and(|v| matches!(v, Value::Promise(_)));
                                    (f.is_generator, f.bytecode_index, has_promise)
                                } else {
                                    (false, 0, false)
                                };

                            if is_generator {
                                if cfg!(debug_assertions) && std::env::var("GEN_TRACE").is_ok() {
                                    eprintln!("[GEN_TRACE] Detected generator, creating object");
                                }
                                // Create a generator object instead of executing immediately
                                let gen_idx = self.heap.len();
                                self.heap.push(HeapValue::Generator(JsGenerator {
                                    yield_value: Value::Undefined,
                                    resume_pc: bytecode_index,
                                    saved_stack: Vec::new(),
                                    saved_block_scope_stack: Vec::new(),
                                    func_heap_idx: Some(*func_idx),
                                }));
                                self.stack.push(Value::Generator(gen_idx));
                                pc += 1;
                                continue;
                            } else if has_promise_resolve {
                                if let HeapValue::Function(f) = &self.heap[*func_idx] {
                                    if let Some(Value::Promise(promise_idx)) = f.closure.first() {
                                        match f.name.as_deref() {
                                            Some("resolve") => {
                                                let val = args
                                                    .first()
                                                    .cloned()
                                                    .unwrap_or(Value::Undefined);
                                                self.resolve_promise(*promise_idx, val);
                                                self.stack.push(Value::Undefined);
                                            }
                                            Some("reject") => {
                                                let reason = args
                                                    .first()
                                                    .cloned()
                                                    .unwrap_or(Value::Undefined);
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
                            } else if bytecode_index == usize::MAX {
                                // Check if this is a bound function
                                if let HeapValue::Function(f) = &self.heap[*func_idx] {
                                    if f.name.as_deref() == Some("bound") && f.closure.len() >= 2 {
                                        // Bound function: closure[0] = original fn, closure[1] = bound this, closure[2..] = bound args
                                        let original_fn = f.closure[0].clone();
                                        let bound_this = f.closure[1].clone();
                                        let bound_args = f.closure[2..].to_vec();
                                        // Combine bound args with call args
                                        let mut combined_args = bound_args;
                                        combined_args.extend(args);
                                        let result = self.call_value(
                                            &original_fn,
                                            &bound_this,
                                            &combined_args,
                                        )?;
                                        self.stack.push(result);
                                        pc += 1;
                                        continue;
                                    }
                                }
                                self.stack.push(Value::Undefined);
                            } else {
                                let same_module =
                                    if let HeapValue::Function(f) = &self.heap[*func_idx] {
                                        match (&f.owner_module, &self.current_module) {
                                            (Some(om), Some(cm)) => Rc::ptr_eq(om, cm),
                                            (None, None) => true,
                                            _ => false,
                                        }
                                    } else {
                                        false
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
                                            source_name: self.current_module_path.clone(),
                                            generator_heap_idx: None,
                                            source_line: self.current_source_line(pc),
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
                                let arr_idx = self.gc.allocate(
                                    &mut self.heap,
                                    HeapValue::Array(JsArray { elements: args }),
                                );
                                let trap_result = self.call_proxy_trap(
                                    &handler,
                                    "apply",
                                    &[target, Value::Undefined, Value::Array(arr_idx)],
                                );
                                match trap_result {
                                    Ok(v) => self.stack.push(v),
                                    Err(e) => return Err(e),
                                }
                            } else {
                                return Err(self.err_at_location(Error::TypeError(format!(
                                    "{} is not a function",
                                    self.value_to_string(&callee)
                                ))));
                            }
                        }
                        _ => {
                            return Err(self.err_at_location(Error::TypeError(format!(
                                "{} is not a function",
                                self.value_to_string(&callee)
                            ))));
                        }
                    }
                }
                Instruction::CallMethod(argc) => {
                    let mut args = Vec::new();
                    for _ in 0..*argc {
                        args.push(
                            self.stack
                                .pop()
                                .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?,
                        );
                    }
                    args.reverse();
                    let key = self
                        .stack
                        .pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let object = self
                        .stack
                        .pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let method = self.get_property(&object, &key)?;
                    match method {
                        Value::Function(func_idx) => {
                            if let HeapValue::Function(f) = &self.heap[func_idx] {
                                let f_clone = f.clone();
                                let same_module =
                                    match (&f_clone.owner_module, &self.current_module) {
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
                                        source_name: self.current_module_path.clone(),
                                        generator_heap_idx: None,
                                        source_line: self.current_source_line(pc),
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
                            return Err(self.err_at_location(Error::TypeError(format!(
                                "{} is not a function",
                                self.value_to_string(&method)
                            ))));
                        }
                    }
                }
                Instruction::Construct(argc) => {
                    let mut args = Vec::new();
                    for _ in 0..*argc {
                        args.push(
                            self.stack
                                .pop()
                                .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?,
                        );
                    }
                    args.reverse();
                    let constructor = self
                        .stack
                        .pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    match &constructor {
                        Value::Function(func_idx) => {
                            let proto_idx = if let Value::Object(proto_obj_idx) = self
                                .get_property(
                                    &constructor,
                                    &Value::String("prototype".to_string()),
                                )? {
                                Some(proto_obj_idx)
                            } else {
                                None
                            };
                            let new_obj_heap_idx = self.gc.allocate(
                                &mut self.heap,
                                HeapValue::Object(JsObject::with_prototype(proto_idx)),
                            );
                            let this_val = Value::Object(new_obj_heap_idx);
                            if let HeapValue::Function(f) = &self.heap[*func_idx] {
                                if f.bytecode_index == usize::MAX {
                                    // Default constructor - auto-call super if subclass
                                    if let Some(ref super_val) = f.super_class {
                                        if let Value::Function(super_func_idx) = super_val {
                                            if let HeapValue::Function(super_f) =
                                                &self.heap[*super_func_idx]
                                            {
                                                if super_f.bytecode_index != usize::MAX {
                                                    let super_f_clone = super_f.clone();
                                                    let return_address = pc + 1;
                                                    let base_pointer = self.stack.len();
                                                    self.call_stack.push(CallFrame {
                                                        return_address,
                                                        base_pointer,
                                                        closure_var_count: 0,
                                                        func_heap_idx: Some(*super_func_idx),
                                                        this_value: Some(this_val.clone()),
                                                        is_construct: true,
                                                        source_name: self
                                                            .current_module_path
                                                            .clone(),
                                                        generator_heap_idx: None,
                                                        source_line: self.current_source_line(pc),
                                                    });
                                                    for arg in args {
                                                        self.stack.push(arg);
                                                    }
                                                    pc = super_f_clone.bytecode_index;
                                                    continue;
                                                }
                                            }
                                        } else if let Value::NativeFunction(super_native_idx) =
                                            super_val
                                        {
                                            let result = self.call_native(
                                                *super_native_idx,
                                                &this_val,
                                                &args,
                                            )?;
                                            match result {
                                                Value::Object(_)
                                                | Value::Array(_)
                                                | Value::Function(_)
                                                | Value::Promise(_)
                                                | Value::Proxy(_)
                                                | Value::Date(_)
                                                | Value::RegExp(_)
                                                | Value::Map(_)
                                                | Value::Set(_)
                                                | Value::TypedArray(_) => {
                                                    self.stack.push(result);
                                                }
                                                _ => {
                                                    self.stack.push(this_val);
                                                }
                                            }
                                            continue;
                                        }
                                    }
                                    self.stack.push(this_val);
                                } else {
                                    let f_clone = f.clone();
                                    let same_module =
                                        match (&f_clone.owner_module, &self.current_module) {
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
                                            source_name: self.current_module_path.clone(),
                                            generator_heap_idx: None,
                                            source_line: self.current_source_line(pc),
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
                                        Value::Object(_)
                                        | Value::Array(_)
                                        | Value::Function(_)
                                        | Value::Promise(_)
                                        | Value::Proxy(_) => {
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
                            let final_args = args.clone();
                            let proto_idx = self.find_native_prototype(*native_idx);
                            let new_obj_heap_idx = self.gc.allocate(
                                &mut self.heap,
                                HeapValue::Object(JsObject::with_prototype(proto_idx)),
                            );
                            let this_val = Value::Object(new_obj_heap_idx);
                            let result = self.call_native(*native_idx, &this_val, &final_args)?;
                            match result {
                                Value::Object(_)
                                | Value::Array(_)
                                | Value::Function(_)
                                | Value::Promise(_)
                                | Value::Proxy(_)
                                | Value::Date(_)
                                | Value::RegExp(_)
                                | Value::Map(_)
                                | Value::Set(_)
                                | Value::WeakMap(_)
                                | Value::WeakSet(_)
                                | Value::TypedArray(_) => {
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
                                let args_arr_idx = self.gc.allocate(
                                    &mut self.heap,
                                    HeapValue::Array(JsArray { elements: args }),
                                );
                                let trap_result = self.call_proxy_trap(
                                    &handler,
                                    "construct",
                                    &[target, Value::Array(args_arr_idx), constructor.clone()],
                                );
                                match trap_result {
                                    Ok(v) => self.stack.push(v),
                                    Err(e) => return Err(e),
                                }
                            } else {
                                return Err(self.err_at_location(Error::TypeError(format!(
                                    "{} is not a constructor",
                                    self.value_to_string(&constructor)
                                ))));
                            }
                        }
                        _ => {
                            return Err(self.err_at_location(Error::TypeError(format!(
                                "{} is not a constructor",
                                self.value_to_string(&constructor)
                            ))));
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
                        args.push(
                            self.stack
                                .pop()
                                .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?,
                        );
                    }
                    args.reverse();
                    let this_val = self
                        .stack
                        .pop()
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
                                let proto_val = self.get_property(
                                    &super_class,
                                    &Value::String("prototype".to_string()),
                                )?;
                                let proto_idx = if let Value::Object(pi) = proto_val {
                                    Some(pi)
                                } else {
                                    None
                                };
                                let new_obj_heap_idx = self.gc.allocate(
                                    &mut self.heap,
                                    HeapValue::Object(JsObject::with_prototype(proto_idx)),
                                );
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
                                    source_name: self.current_module_path.clone(),
                                    generator_heap_idx: None,
                                    source_line: self.current_source_line(pc),
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
                            return Err(self.err_at_location(Error::TypeError(
                                "Superclass is not a constructor".into(),
                            )));
                        }
                    }
                }
                Instruction::SuperGet => {
                    let key = self
                        .stack
                        .pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let _this = self
                        .stack
                        .pop()
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
                Instruction::ImportModule(source) => match self.load_and_run_module(source)? {
                    Some(module_path) => {
                        let exports = self
                            .module_registry
                            .get(&module_path)
                            .cloned()
                            .unwrap_or_default();
                        let heap_idx = self.heap.len();
                        let mut props = HashMap::new();
                        for (k, v) in &exports {
                            props.insert(k.clone(), v.clone());
                        }
                        self.heap.push(HeapValue::Object(JsObject {
                            properties: props,
                            prototype: None,
                            extensible: true,
                        }));
                        self.stack.push(Value::Object(heap_idx));
                    }
                    None => {
                        self.stack.push(Value::Undefined);
                    }
                },
                Instruction::ImportNamed(source, imported_name, local_name) => {
                    match self.load_and_run_module(source)? {
                        Some(module_path) => {
                            let exports = self
                                .module_registry
                                .get(&module_path)
                                .cloned()
                                .unwrap_or_default();
                            let val = exports
                                .get(imported_name)
                                .cloned()
                                .unwrap_or(Value::Undefined);
                            self.globals.insert(local_name.clone(), val);
                        }
                        None => {
                            self.globals.insert(local_name.clone(), Value::Undefined);
                        }
                    }
                }
                Instruction::ImportDefault(source, local_name) => {
                    match self.load_and_run_module(source)? {
                        Some(module_path) => {
                            let exports = self
                                .module_registry
                                .get(&module_path)
                                .cloned()
                                .unwrap_or_default();
                            let val = if let Some(v) = exports.get("default") {
                                v.clone()
                            } else if !exports.is_empty() {
                                // Create a default export object from named exports
                                let obj_idx = self.heap.len();
                                let mut obj_props = std::collections::HashMap::new();
                                for (name, val) in &exports {
                                    obj_props.insert(name.clone(), val.clone());
                                }
                                self.heap.push(HeapValue::Object(JsObject {
                                    properties: obj_props,
                                    prototype: None,
                                    extensible: true,
                                }));
                                // Cache it in the registry
                                if let Some(registry_exports) =
                                    self.module_registry.get_mut(&module_path)
                                {
                                    registry_exports
                                        .insert("default".to_string(), Value::Object(obj_idx));
                                }
                                Value::Object(obj_idx)
                            } else {
                                Value::Undefined
                            };
                            self.globals.insert(local_name.clone(), val);
                        }
                        None => {
                            self.globals.insert(local_name.clone(), Value::Undefined);
                        }
                    }
                }
                Instruction::ImportAll(source, local_name) => {
                    match self.load_and_run_module(source)? {
                        Some(module_path) => {
                            let exports = self
                                .module_registry
                                .get(&module_path)
                                .cloned()
                                .unwrap_or_default();
                            let heap_idx = self.heap.len();
                            let mut props = HashMap::new();
                            for (k, v) in &exports {
                                props.insert(k.clone(), v.clone());
                            }
                            self.heap.push(HeapValue::Object(JsObject {
                                properties: props,
                                prototype: None,
                                extensible: true,
                            }));
                            self.globals
                                .insert(local_name.clone(), Value::Object(heap_idx));
                        }
                        None => {
                            let heap_idx = self.heap.len();
                            self.heap.push(HeapValue::Object(JsObject::new()));
                            self.globals
                                .insert(local_name.clone(), Value::Object(heap_idx));
                        }
                    }
                }
                Instruction::NativeImport(source, local_name) => {
                    match self.load_and_run_module(source)? {
                        Some(module_path) => {
                            let exports = self
                                .module_registry
                                .get(&module_path)
                                .cloned()
                                .unwrap_or_default();
                            let val = exports.get("default").cloned().unwrap_or_else(|| {
                                let heap_idx = self.heap.len();
                                let mut props = HashMap::new();
                                for (k, v) in &exports {
                                    props.insert(k.clone(), v.clone());
                                }
                                self.heap.push(HeapValue::Object(JsObject {
                                    properties: props,
                                    prototype: None,
                                    extensible: true,
                                }));
                                Value::Object(heap_idx)
                            });
                            self.globals.insert(local_name.clone(), val);
                        }
                        None => {
                            self.globals.insert(local_name.clone(), Value::Undefined);
                        }
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
                    let value = self
                        .stack
                        .pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    if let Value::Promise(promise_idx) = &value {
                        if let HeapValue::Promise(p) = &self.heap[*promise_idx] {
                            match &p.state {
                                PromiseState::Fulfilled(v) => {
                                    self.stack.push(v.clone());
                                }
                                PromiseState::Rejected(reason) => {
                                    return Err(self.err_at_location(Error::RuntimeError(
                                        format!("Unhandled promise rejection: {:?}", reason),
                                    )));
                                }
                                PromiseState::Pending => {
                                    let frame = SuspendedFrame {
                                        promise_idx: *promise_idx,
                                        resume_pc: pc + 1,
                                        stack_snapshot: self.stack.clone(),
                                        call_stack_snapshot: self.call_stack.clone(),
                                        module: self.current_module.clone(),
                                        module_path: self.current_module_path.clone(),
                                        exception_handlers_snapshot: self
                                            .exception_handlers
                                            .clone(),
                                        block_scope_stack_snapshot: self.block_scope_stack.clone(),
                                    };
                                    self.suspended_frames.push_back(frame);
                                    return Ok(Value::Undefined);
                                }
                            }
                        } else {
                            self.stack.push(value);
                        }
                    } else {
                        self.stack.push(value);
                    }
                }
                Instruction::DynamicImport => {
                    let source = self
                        .stack
                        .pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let source_str = match &source {
                        Value::String(s) => s.clone(),
                        _ => {
                            let promise_idx = self.heap.len();
                            self.heap.push(HeapValue::Promise(
                                crate::objects::js_promise::JsPromise::rejected(Value::String(
                                    format!("Cannot resolve import source: {}", source),
                                )),
                            ));
                            self.stack.push(Value::Promise(promise_idx));
                            continue;
                        }
                    };
                    match self.load_and_run_module(&source_str) {
                        Ok(Some(module_path)) => {
                            let exports = self
                                .module_registry
                                .get(&module_path)
                                .cloned()
                                .unwrap_or_default();
                            let heap_idx = self.heap.len();
                            let mut props = HashMap::new();
                            for (k, v) in &exports {
                                props.insert(k.clone(), v.clone());
                            }
                            self.heap.push(HeapValue::Object(JsObject {
                                properties: props,
                                prototype: None,
                                extensible: true,
                            }));
                            let module_obj = Value::Object(heap_idx);
                            let promise_idx = self.heap.len();
                            self.heap.push(HeapValue::Promise(
                                crate::objects::js_promise::JsPromise::fulfilled(module_obj),
                            ));
                            self.stack.push(Value::Promise(promise_idx));
                        }
                        _ => {
                            let reason_idx = self.heap.len();
                            let mut props = HashMap::new();
                            props.insert(
                                "message".into(),
                                Value::String(format!("Module '{}' not found", source_str)),
                            );
                            self.heap.push(HeapValue::Object(JsObject {
                                properties: props,
                                prototype: None,
                                extensible: true,
                            }));
                            let promise_idx = self.heap.len();
                            self.heap.push(HeapValue::Promise(
                                crate::objects::js_promise::JsPromise::rejected(Value::Object(
                                    reason_idx,
                                )),
                            ));
                            self.stack.push(Value::Promise(promise_idx));
                        }
                    }
                }
                Instruction::GetIterator => {
                    let iterable = self
                        .stack
                        .pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;

                    // Check for built-in iterables (arrays, strings)
                    match &iterable {
                        Value::Array(arr_idx) => {
                            // Create an array iterator object
                            let arr_clone = if let HeapValue::Array(arr) = &self.heap[*arr_idx] {
                                arr.elements.clone()
                            } else {
                                Vec::new()
                            };
                            let data_idx = self.gc.allocate(
                                &mut self.heap,
                                HeapValue::Array(JsArray {
                                    elements: arr_clone,
                                }),
                            );
                            let mut props = HashMap::new();
                            props.insert("__type".to_string(), Value::String("array".to_string()));
                            props.insert("__index".to_string(), Value::Integer(0));
                            props.insert("__data".to_string(), Value::Array(data_idx));
                            // Iterator helper methods
                            props.insert("map".to_string(), Value::NativeFunction(230));
                            props.insert("filter".to_string(), Value::NativeFunction(231));
                            props.insert("take".to_string(), Value::NativeFunction(232));
                            props.insert("drop".to_string(), Value::NativeFunction(233));
                            props.insert("forEach".to_string(), Value::NativeFunction(234));
                            props.insert("toArray".to_string(), Value::NativeFunction(235));
                            let iter_idx = self.gc.allocate(
                                &mut self.heap,
                                HeapValue::Object(JsObject {
                                    properties: props,
                                    prototype: None,
                                    extensible: true,
                                }),
                            );
                            self.stack.push(Value::Object(iter_idx));
                        }
                        Value::String(s) => {
                            // Create a string iterator object
                            let chars: Vec<Value> =
                                s.chars().map(|c| Value::String(c.to_string())).collect();
                            let data_idx = self.gc.allocate(
                                &mut self.heap,
                                HeapValue::Array(JsArray { elements: chars }),
                            );
                            let mut props = HashMap::new();
                            props.insert("__type".to_string(), Value::String("string".to_string()));
                            props.insert("__index".to_string(), Value::Integer(0));
                            props.insert("__data".to_string(), Value::Array(data_idx));
                            // Iterator helper methods
                            props.insert("map".to_string(), Value::NativeFunction(230));
                            props.insert("filter".to_string(), Value::NativeFunction(231));
                            props.insert("take".to_string(), Value::NativeFunction(232));
                            props.insert("drop".to_string(), Value::NativeFunction(233));
                            props.insert("forEach".to_string(), Value::NativeFunction(234));
                            props.insert("toArray".to_string(), Value::NativeFunction(235));
                            let iter_idx = self.gc.allocate(
                                &mut self.heap,
                                HeapValue::Object(JsObject {
                                    properties: props,
                                    prototype: None,
                                    extensible: true,
                                }),
                            );
                            self.stack.push(Value::Object(iter_idx));
                        }
                        _ => {
                            // Look up Symbol.iterator on the iterable
                            let iterator_symbol = Value::Symbol(crate::objects::SYMBOL_ITERATOR);
                            let iterator_fn = self.get_property(&iterable, &iterator_symbol)?;
                            match iterator_fn {
                                Value::Function(_) | Value::NativeFunction(_) => {
                                    let iterator = self.call_value(&iterator_fn, &iterable, &[])?;
                                    self.stack.push(iterator);
                                }
                                _ => {
                                    return Err(Error::TypeError(
                                        "Value is not iterable (no Symbol.iterator method)".into(),
                                    ));
                                }
                            }
                        }
                    }
                }
                Instruction::GetAsyncIterator => {
                    let iterable = self
                        .stack
                        .pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;

                    // For async iteration, check Symbol.asyncIterator first, then Symbol.iterator
                    // Built-in arrays and strings use the regular iterator
                    match &iterable {
                        Value::Array(arr_idx) => {
                            // Create an array iterator (same as GetIterator)
                            let arr_clone = if let HeapValue::Array(arr) = &self.heap[*arr_idx] {
                                arr.elements.clone()
                            } else {
                                Vec::new()
                            };
                            let data_idx = self.gc.allocate(
                                &mut self.heap,
                                HeapValue::Array(JsArray {
                                    elements: arr_clone,
                                }),
                            );
                            let mut props = HashMap::new();
                            props.insert("__type".to_string(), Value::String("array".to_string()));
                            props.insert("__index".to_string(), Value::Integer(0));
                            props.insert("__data".to_string(), Value::Array(data_idx));
                            props.insert("map".to_string(), Value::NativeFunction(230));
                            props.insert("filter".to_string(), Value::NativeFunction(231));
                            props.insert("take".to_string(), Value::NativeFunction(232));
                            props.insert("drop".to_string(), Value::NativeFunction(233));
                            props.insert("forEach".to_string(), Value::NativeFunction(234));
                            props.insert("toArray".to_string(), Value::NativeFunction(235));
                            let iter_idx = self.gc.allocate(
                                &mut self.heap,
                                HeapValue::Object(JsObject {
                                    properties: props,
                                    prototype: None,
                                    extensible: true,
                                }),
                            );
                            self.stack.push(Value::Object(iter_idx));
                        }
                        Value::String(s) => {
                            let chars: Vec<Value> =
                                s.chars().map(|c| Value::String(c.to_string())).collect();
                            let data_idx = self.gc.allocate(
                                &mut self.heap,
                                HeapValue::Array(JsArray { elements: chars }),
                            );
                            let mut props = HashMap::new();
                            props.insert("__type".to_string(), Value::String("string".to_string()));
                            props.insert("__index".to_string(), Value::Integer(0));
                            props.insert("__data".to_string(), Value::Array(data_idx));
                            props.insert("map".to_string(), Value::NativeFunction(230));
                            props.insert("filter".to_string(), Value::NativeFunction(231));
                            props.insert("take".to_string(), Value::NativeFunction(232));
                            props.insert("drop".to_string(), Value::NativeFunction(233));
                            props.insert("forEach".to_string(), Value::NativeFunction(234));
                            props.insert("toArray".to_string(), Value::NativeFunction(235));
                            let iter_idx = self.gc.allocate(
                                &mut self.heap,
                                HeapValue::Object(JsObject {
                                    properties: props,
                                    prototype: None,
                                    extensible: true,
                                }),
                            );
                            self.stack.push(Value::Object(iter_idx));
                        }
                        _ => {
                            // Try Symbol.asyncIterator first, then Symbol.iterator
                            let async_iter_symbol =
                                Value::Symbol(crate::objects::SYMBOL_ASYNC_ITERATOR);
                            let async_iter_fn = self.get_property(&iterable, &async_iter_symbol)?;
                            let iterator_fn = if matches!(
                                async_iter_fn,
                                Value::Function(_) | Value::NativeFunction(_)
                            ) {
                                async_iter_fn
                            } else {
                                let iterator_symbol =
                                    Value::Symbol(crate::objects::SYMBOL_ITERATOR);
                                self.get_property(&iterable, &iterator_symbol)?
                            };
                            match iterator_fn {
                                Value::Function(_) | Value::NativeFunction(_) => {
                                    let iterator = self.call_value(&iterator_fn, &iterable, &[])?;
                                    self.stack.push(iterator);
                                }
                                _ => {
                                    return Err(Error::TypeError(
                                        "Value is not async iterable".into(),
                                    ));
                                }
                            }
                        }
                    }
                }
                Instruction::IteratorNext(target) => {
                    // Stack: [..., iterator]
                    let iterator = self
                        .stack
                        .last()
                        .cloned()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;

                    // Check if this is a built-in iterator (has __type property)
                    if let Value::Object(iter_idx) = &iterator {
                        if let HeapValue::Object(iter_obj) = &self.heap[*iter_idx] {
                            if let Some(Value::String(iter_type)) =
                                iter_obj.properties.get("__type")
                            {
                                let index = match iter_obj.properties.get("__index") {
                                    Some(Value::Integer(i)) => *i as usize,
                                    _ => 0,
                                };
                                if let Some(data_val) = iter_obj.properties.get("__data") {
                                    match (iter_type.as_str(), data_val) {
                                        ("array", Value::Array(arr_idx)) => {
                                            if let HeapValue::Array(arr) = &self.heap[*arr_idx] {
                                                if index >= arr.elements.len() {
                                                    // Done - pop iterator and jump
                                                    self.stack.pop();
                                                    pc = *target as usize;
                                                    continue;
                                                }
                                                let value = arr.elements[index].clone();
                                                // Update index
                                                if let HeapValue::Object(iter_obj_mut) =
                                                    &mut self.heap[*iter_idx]
                                                {
                                                    iter_obj_mut.properties.insert(
                                                        "__index".to_string(),
                                                        Value::Integer((index + 1) as i64),
                                                    );
                                                }
                                                self.stack.push(value);
                                            }
                                        }
                                        ("string", Value::Array(chars_idx)) => {
                                            if let HeapValue::Array(chars_arr) =
                                                &self.heap[*chars_idx]
                                            {
                                                if index >= chars_arr.elements.len() {
                                                    self.stack.pop();
                                                    pc = *target as usize;
                                                    continue;
                                                }
                                                let value = chars_arr.elements[index].clone();
                                                if let HeapValue::Object(iter_obj_mut) =
                                                    &mut self.heap[*iter_idx]
                                                {
                                                    iter_obj_mut.properties.insert(
                                                        "__index".to_string(),
                                                        Value::Integer((index + 1) as i64),
                                                    );
                                                }
                                                self.stack.push(value);
                                            }
                                        }
                                        _ => {
                                            // Unknown iterator type, fall through to generic handling
                                        }
                                    }
                                }
                                pc += 1;
                                continue;
                            }
                        }
                    }

                    // Generic iterator protocol - call iterator.next()
                    let next_fn =
                        self.get_property(&iterator, &Value::String("next".to_string()))?;
                    let next_result = self.call_value(&next_fn, &iterator, &[])?;
                    // Get done property
                    let done =
                        self.get_property(&next_result, &Value::String("done".to_string()))?;
                    match done {
                        Value::Boolean(true) => {
                            // Iterator is done - pop the iterator and jump
                            self.stack.pop();
                            pc = *target as usize;
                            continue;
                        }
                        _ => {
                            // Not done - extract value and push it (iterator stays below)
                            let value = self
                                .get_property(&next_result, &Value::String("value".to_string()))?;
                            self.stack.push(value);
                        }
                    }
                }
                Instruction::IteratorClose => {
                    // Pop the iterator and call .return() if it exists
                    let iterator = self
                        .stack
                        .pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    if let Ok(return_fn) =
                        self.get_property(&iterator, &Value::String("return".to_string()))
                    {
                        match return_fn {
                            Value::Function(_) | Value::NativeFunction(_) => {
                                let _ = self.call_value(&return_fn, &iterator, &[]);
                            }
                            _ => {}
                        }
                    }
                }
                Instruction::AsyncIteratorNext(target) => {
                    // Similar to IteratorNext but also awaits the value if it's a promise
                    let iterator = self
                        .stack
                        .last()
                        .cloned()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;

                    // First, check for built-in iterators (same as IteratorNext)
                    if let Value::Object(iter_idx) = &iterator {
                        let iter_idx = *iter_idx;
                        if let HeapValue::Object(ref iter_obj) = self.heap[iter_idx] {
                            if let Some(Value::String(_iter_type)) =
                                iter_obj.properties.get("__type")
                            {
                                let index = match iter_obj.properties.get("__index") {
                                    Some(Value::Integer(i)) => *i as usize,
                                    _ => 0,
                                };
                                if let Some(data_val) = iter_obj.properties.get("__data").cloned() {
                                    let done = match &data_val {
                                        Value::Array(arr_idx) => {
                                            if let HeapValue::Array(arr) = &self.heap[*arr_idx] {
                                                index >= arr.elements.len()
                                            } else {
                                                true
                                            }
                                        }
                                        _ => true,
                                    };
                                    if done {
                                        self.stack.pop();
                                        pc = *target as usize;
                                        continue;
                                    }
                                    let value = match &data_val {
                                        Value::Array(arr_idx) => {
                                            if let HeapValue::Array(arr) = &self.heap[*arr_idx] {
                                                arr.elements[index].clone()
                                            } else {
                                                Value::Undefined
                                            }
                                        }
                                        _ => Value::Undefined,
                                    };
                                    // Update index
                                    if let HeapValue::Object(ref mut obj) = self.heap[iter_idx] {
                                        obj.properties.insert(
                                            "__index".to_string(),
                                            Value::Integer((index + 1) as i64),
                                        );
                                    }
                                    // Await the value if it's a promise
                                    let awaited_value = if let Value::Promise(promise_idx) = &value
                                    {
                                        if let HeapValue::Promise(p) = &self.heap[*promise_idx] {
                                            match &p.state {
                                                PromiseState::Fulfilled(v) => v.clone(),
                                                PromiseState::Rejected(_) => Value::Undefined,
                                                PromiseState::Pending => value.clone(),
                                            }
                                        } else {
                                            value.clone()
                                        }
                                    } else {
                                        value.clone()
                                    };
                                    self.stack.push(awaited_value);
                                    pc += 1;
                                    continue;
                                }
                            }
                        }
                    }

                    // Generic iterator - call iterator.next()
                    let next_fn =
                        self.get_property(&iterator, &Value::String("next".to_string()))?;
                    let next_result = self.call_value(&next_fn, &iterator, &[])?;
                    // Check done property
                    let done =
                        self.get_property(&next_result, &Value::String("done".to_string()))?;
                    match done {
                        Value::Boolean(true) => {
                            self.stack.pop();
                            pc = *target as usize;
                            continue;
                        }
                        _ => {
                            let value = self
                                .get_property(&next_result, &Value::String("value".to_string()))?;
                            // Await the value if it's a promise
                            let awaited_value = if let Value::Promise(promise_idx) = &value {
                                if let HeapValue::Promise(p) = &self.heap[*promise_idx] {
                                    match &p.state {
                                        PromiseState::Fulfilled(v) => v.clone(),
                                        PromiseState::Rejected(_) => Value::Undefined,
                                        PromiseState::Pending => value.clone(),
                                    }
                                } else {
                                    value.clone()
                                }
                            } else {
                                value.clone()
                            };
                            self.stack.push(awaited_value);
                        }
                    }
                }
                _ => {
                    // Delegate to instruction handler modules
                    #[allow(clippy::if_same_then_else)]
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
                    } else {
                        let saved_pc = pc;
                        if self.exec_exception(&instruction, &mut pc)? {
                            if pc != saved_pc {
                                continue;
                            }
                        } else {
                            return Err(Error::RuntimeError(format!(
                                "Unhandled instruction: {:?}",
                                instruction
                            )));
                        }
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
            self.stack
                .pop()
                .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?
        } else {
            Value::Undefined
        };
        let proto_obj_idx = self
            .gc
            .allocate(&mut self.heap, HeapValue::Object(JsObject::new()));
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
                    f.prototype.map(Value::Object)
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
            self.gc.allocate(
                &mut self.heap,
                HeapValue::Function(JsFunction {
                    name: Some(class_info.name.clone()),
                    params: func_info.params,
                    bytecode_index: func_info.bytecode_index,
                    closure: Vec::new(),
                    prototype: Some(proto_obj_idx),
                    super_class: Some(super_val.clone()),
                    properties: HashMap::new(),
                    owner_module: owner,
                    module_scope: None,
                    is_generator: false,
                }),
            )
        } else {
            self.gc.allocate(
                &mut self.heap,
                HeapValue::Function(JsFunction {
                    name: Some(class_info.name.clone()),
                    params: Vec::new(),
                    bytecode_index: usize::MAX,
                    closure: Vec::new(),
                    prototype: Some(proto_obj_idx),
                    super_class: Some(super_val.clone()),
                    properties: HashMap::new(),
                    owner_module: None,
                    module_scope: None,
                    is_generator: false,
                }),
            )
        };
        if let HeapValue::Object(proto_obj) = &mut self.heap[proto_obj_idx] {
            proto_obj
                .properties
                .insert("constructor".to_string(), Value::Function(ctor_heap_idx));
        }
        for method_info in &class_info.methods {
            let method_func_info = module.functions[method_info.func_idx as usize].clone();
            let method_proto_idx = self
                .gc
                .allocate(&mut self.heap, HeapValue::Object(JsObject::new()));
            let owner = self.current_module.clone();
            let method_heap_idx = self.gc.allocate(
                &mut self.heap,
                HeapValue::Function(JsFunction {
                    name: Some(method_info.name.clone()),
                    params: method_func_info.params,
                    bytecode_index: method_func_info.bytecode_index,
                    closure: Vec::new(),
                    prototype: Some(method_proto_idx),
                    super_class: None,
                    properties: HashMap::new(),
                    owner_module: owner,
                    module_scope: None,
                    is_generator: false,
                }),
            );
            let method_val = Value::Function(method_heap_idx);
            if method_info.is_static {
                if let HeapValue::Function(ctor_func) = &mut self.heap[ctor_heap_idx] {
                    ctor_func
                        .properties
                        .insert(method_info.name.clone(), method_val);
                }
            } else {
                match &method_info.kind {
                    crate::compiler::ClassMethodKind::Getter => {
                        if let HeapValue::Object(proto_obj) = &mut self.heap[proto_obj_idx] {
                            proto_obj
                                .properties
                                .insert(format!("__getter_{}", method_info.name), method_val);
                        }
                    }
                    crate::compiler::ClassMethodKind::Setter => {
                        if let HeapValue::Object(proto_obj) = &mut self.heap[proto_obj_idx] {
                            proto_obj
                                .properties
                                .insert(format!("__setter_{}", method_info.name), method_val);
                        }
                    }
                    crate::compiler::ClassMethodKind::Method => {
                        if let HeapValue::Object(proto_obj) = &mut self.heap[proto_obj_idx] {
                            proto_obj
                                .properties
                                .insert(method_info.name.clone(), method_val);
                        }
                    }
                }
            }
        }
        self.stack.push(Value::Function(ctor_heap_idx));
        Ok(())
    }

    fn handle_super_construct(
        &mut self,
        _argc: u16,
        _pc: usize,
        _module: &CompiledModule,
    ) -> Result<()> {
        // This is handled inline in execute_from() due to pc control flow
        Ok(())
    }

    pub(crate) fn build_stack_trace(&self, error_name: &str, message: &str) -> String {
        let mut trace = format!(
            "{}{}",
            error_name,
            if message.is_empty() {
                String::new()
            } else {
                format!(": {}", message)
            }
        );

        for frame in self.call_stack.iter().rev() {
            let func_name = frame
                .func_heap_idx
                .and_then(|idx| {
                    if let HeapValue::Function(f) = &self.heap[idx] {
                        f.name.clone()
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "<anonymous>".to_string());

            let location = match (&frame.source_name, frame.source_line) {
                (Some(name), Some(line)) => format!(" ({}:{})", name, line),
                (Some(name), None) => format!(" ({})", name),
                (None, Some(line)) => format!(" (line {})", line),
                (None, None) => String::new(),
            };

            trace.push_str(&format!("\n    at {}{}", func_name, location));
        }

        trace
    }

    pub(crate) fn call_stack_backtrace(&self) -> String {
        let mut frames = Vec::new();
        for frame in self.call_stack.iter().rev() {
            let func_name = frame
                .func_heap_idx
                .and_then(|idx| {
                    if let HeapValue::Function(f) = &self.heap[idx] {
                        f.name.clone()
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "<anonymous>".to_string());

            let location = match (&frame.source_name, frame.source_line) {
                (Some(name), Some(line)) => format!("{}:{}", name, line),
                (Some(name), None) => name.clone(),
                (None, Some(line)) => format!("line {}", line),
                (None, None) => "<script>".to_string(),
            };

            frames.push(format!("    at {} ({})", func_name, location));
        }

        if frames.is_empty() {
            String::new()
        } else {
            format!("\nCall stack:\n{}", frames.join("\n"))
        }
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new().expect("Failed to create default interpreter")
    }
}
