mod builtins;
mod call_frame;
mod calls;
mod class_ops;
mod control_flow;
mod error_format;
mod exception_handling;
mod function_ops;
mod heap_types;
mod instructions;
mod iterators;
mod modules;
pub(crate) mod native_loader;
mod ops;
mod promise_runtime;
mod property_access;
pub mod safe_function;
pub mod safe_library;
mod value_ops;

pub(crate) use call_frame::{CallFrame, ExceptionHandler};
pub use heap_types::{
    HeapValue, JsArray, JsCompiledRegex, JsFunction, JsGenerator, JsIterator, JsObject, JsProxyData, JsRegExp,
};

use crate::compiler::{CompiledModule, Instruction};
use crate::errors::runtime_errors::runtime_error_stack_overflow;
use crate::errors::{Error, Result};
use crate::objects::js_promise::PromiseState;
use crate::objects::{ConsString, Value};
use crate::runtime_env::async_runtime::AsyncRuntime;
use crate::vm::interpreter::control_flow::ControlFlowOutcome;
use rustc_hash::FxHashMap;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;
use std::cell::RefCell;

// ── Event-loop integration trait ──────────────────────────────────────────
//
// Native modules that produce long-lived async work (HTTP servers, TCP
// connections, WebSocket clients, …) implement this trait and register an
// instance via `interp.pending_event_sources.push(Box::new(…))` during
// their `listen` / `connect` calls.
//
// After the top-level script finishes, `TailsRuntime::run_event_loop`
// drains the pending queue and polls every registered source until none
// report pending work.

/// A source of async I/O that keeps the event loop alive.
///
/// Implementors are owned by [`TailsRuntime`] and polled each tick with a
/// mutable reference to the interpreter so they can invoke JS callbacks.
pub trait EventSource: 'static {
    /// Returns `true` if this source still has open handles / pending work.
    fn is_active(&self) -> bool;

    /// Process one non-blocking poll cycle.
    ///
    /// Implementations may call into the interpreter (e.g. to fire JS request
    /// handlers or event callbacks).
    fn poll(&mut self, interp: &mut Interpreter) -> Result<()>;
}

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
    pub(crate) globals: FxHashMap<String, Value>,
    pub(crate) stack: Vec<Value>,
    pub(crate) heap: Vec<HeapValue>,
    pub(crate) gc: crate::vm::gc::GarbageCollector,
    pub(crate) call_stack: Vec<CallFrame>,
    pub(crate) current_module: Option<Rc<CompiledModule>>,
    pub(crate) exception_handlers: Vec<ExceptionHandler>,
    pending_exception: Option<Value>,
    pub(crate) async_runtime: AsyncRuntime,
    pub(crate) _promise_stack: Vec<usize>,
    _timer_id_counter: u32,
    pub(crate) module_registry: HashMap<String, FxHashMap<String, Value>>,
    pub(crate) module_exports: FxHashMap<String, Value>,
    pub(crate) current_module_path: Option<String>,
    pub(crate) module_globals: Option<FxHashMap<String, Value>>,
    pub(crate) module_globals_rc: Option<Rc<FxHashMap<String, Value>>>,
    pub(crate) require_cache: FxHashMap<String, Value>,
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
    pub(crate) max_call_stack_depth: usize,
    pub(crate) dynamic_native_fns: Vec<usize>,
    pub(crate) native_object_methods: HashMap<u32, FxHashMap<String, Value>>,
    pub(crate) native_class_registry: HashMap<String, FxHashMap<String, Value>>,
    /// Holds event sources registered by native modules (http, net, websocket, …)
    /// during their listen/connect calls. Drained by the event loop in
    /// [`TailsRuntime::run_event_loop`] after script execution finishes.
    pub(crate) pending_event_sources: Vec<Box<dyn EventSource>>,
}

impl Interpreter {
    pub fn new() -> Result<Self> {
        let mut interp = Self {
            globals: FxHashMap::default(),
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
            module_exports: FxHashMap::default(),
            module_globals: None,
            module_globals_rc: None,
            current_module_path: None,
            require_cache: FxHashMap::default(),
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
            max_call_stack_depth: 10_000,
            dynamic_native_fns: Vec::new(),
            native_object_methods: HashMap::new(),
            native_class_registry: HashMap::new(),
            pending_event_sources: Vec::new(),
        };
        interp.init_builtins();
        Ok(interp)
    }

    // ── Allocation Helpers ───────────────────────────────────────────────────
    //
    // Convenience wrappers around `gc.allocate` for common heap object types.
    // These reduce boilerplate when creating objects, arrays, and strings
    // in native function implementations.

    /// Allocate a new object on the heap with the given properties and prototype.
    pub(super) fn alloc_object(
        &mut self,
        properties: FxHashMap<String, Value>,
        prototype: Option<usize>,
    ) -> usize {
        self.gc.allocate(
            &mut self.heap,
            HeapValue::Object(JsObject {
                properties,
                prototype,
                extensible: true,
            }),
        )
    }

    /// Allocate a new array on the heap with the given elements.
    pub(super) fn alloc_array(&mut self, elements: Vec<Value>) -> usize {
        self.gc.allocate(
            &mut self.heap,
            HeapValue::Array(JsArray { elements }),
        )
    }

    /// Allocate a new string on the heap.
    pub(super) fn alloc_string(&mut self, value: String) -> usize {
        self.gc.allocate(&mut self.heap, HeapValue::String(value))
    }

    /// Allocate a new promise on the heap.
    pub(super) fn alloc_promise(&mut self, promise: crate::objects::js_promise::JsPromise) -> usize {
        self.gc.allocate(&mut self.heap, HeapValue::Promise(promise))
    }

    // ── End Allocation Helpers ───────────────────────────────────────────────

    pub fn execute(&mut self, module: &CompiledModule) -> Result<Value> {
        self.current_module = Some(Rc::new(module.clone()));
        let saved_call_stack_len = self.call_stack.len();
        if self.call_stack.len() >= self.max_call_stack_depth {
            return Err(runtime_error_stack_overflow());
        }
        self.call_stack.push(CallFrame {
            return_address: module.instructions.len(),
            base_pointer: 0,
            closure_var_count: 0,
            func_heap_idx: None,
            this_value: None,
            is_construct: false,
            source_name: self.current_module_path.clone(),
            generator_heap_idx: None,
            source_line: None,
            source_col: None,
            exception_handlers_snapshot: if self.exception_handlers.is_empty() {
                Vec::new()
            } else {
                self.exception_handlers.clone()
            },
            shared_closure_env: HashMap::new(),
        });
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
                    let promise_state = if let HeapValue::Promise(p) = &self.heap[frame.promise_idx]
                    {
                        p.state.clone()
                    } else {
                        PromiseState::Fulfilled(Value::Undefined)
                    };

                    self.stack = frame.stack_snapshot;
                    self.call_stack = frame.call_stack_snapshot;
                    self.current_module = frame.module;
                    self.current_module_path = frame.module_path;
                    self.exception_handlers = frame.exception_handlers_snapshot;
                    self.block_scope_stack = frame.block_scope_stack_snapshot;

                    match &promise_state {
                        PromiseState::Fulfilled(v) => {
                            self.stack.push(v.clone());
                            let module_ref = self.current_module.clone().unwrap();
                            result = self.execute_from(&module_ref, frame.resume_pc);
                            any_resumed = true;
                        }
                        PromiseState::Rejected(reason) => {
                            self.pending_exception = Some(reason.clone());
                            let mut handled = false;
                            while let Some(handler) = self.exception_handlers.last().cloned() {
                                if handler.catch_pc != 0 {
                                    self.exception_handlers.pop();
                                    self.stack.truncate(handler.stack_depth);
                                    let module_ref = self.current_module.clone().unwrap();
                                    result =
                                        self.execute_from(&module_ref, handler.catch_pc as usize);
                                    handled = true;
                                    break;
                                } else if handler.finally_pc != 0 {
                                    self.exception_handlers.pop();
                                    self.stack.truncate(handler.stack_depth);
                                    let module_ref = self.current_module.clone().unwrap();
                                    result =
                                        self.execute_from(&module_ref, handler.finally_pc as usize);
                                    handled = true;
                                    break;
                                } else {
                                    self.exception_handlers.pop();
                                }
                            }
                            if !handled {
                                let exc = self.pending_exception.take().unwrap();
                                let formatted = self.format_rejection_reason(&exc);
                                return Err(self.err_at_location(Error::RuntimeError(format!(
                                    "Unhandled promise rejection:\n{}",
                                    formatted
                                ))));
                            }
                            any_resumed = true;
                        }
                        _ => unreachable!(),
                    }
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

        if result.is_ok() {
            self.call_stack.truncate(saved_call_stack_len);
        }
        result
    }

    pub(crate) fn add_proto_roots(&self, map: &mut FxHashMap<String, Value>) {
        if let Some(idx) = self.regexp_proto_idx {
            map.insert("__regexp_proto__".into(), Value::Object(idx));
        }
        if let Some(idx) = self.date_proto_idx {
            map.insert("__date_proto__".into(), Value::Object(idx));
        }
        if let Some(idx) = self.buffer_proto_idx {
            map.insert("__buffer_proto__".into(), Value::Object(idx));
        }
        if let Some(idx) = self.generator_proto_idx {
            map.insert("__generator_proto__".into(), Value::Object(idx));
        }
    }

    pub(crate) fn collect_garbage(&mut self) {
        let globals_snapshot = self.module_globals_rc.clone().unwrap_or_else(|| {
            let mut map = self.globals.clone();
            self.add_proto_roots(&mut map);
            std::rc::Rc::new(map)
        });
        // Phase 1.5: reserve the destination's capacity to the source's
        // length. The previous `self.stack.clone()` /
        // `self.call_stack.clone()` would grow a fresh `Vec` from capacity
        // 0 → 4 → 8 → 16 → 32 on deep call stacks (which the GC triggers
        // most often). Pre-sizing eliminates 4–5 reallocations on a
        // 100-deep call_stack and 1–2 reallocations on the common 5–10
        // deep case.
        let mut stack_snapshot = Vec::with_capacity(self.stack.len());
        stack_snapshot.extend(self.stack.iter().cloned());
        let mut call_stack_snapshot = Vec::with_capacity(self.call_stack.len());
        call_stack_snapshot.extend(self.call_stack.iter().cloned());
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

    pub(crate) fn current_source_col(&self, pc: usize) -> Option<usize> {
        self.current_module
            .as_ref()
            .and_then(|m| m.source_cols.get(pc).copied().flatten())
    }

    pub(crate) fn err_at_location(&self, mut err: crate::errors::Error) -> crate::errors::Error {
        if err.span.is_some() {
            return err;
        }
        let line = self.current_source_line(self.current_pc);
        let col = self.current_source_col(self.current_pc);
        if let Some(line) = line {
            let file = self.current_module_path.clone();
            err.span = Some(crate::errors::Span::new(line, col.unwrap_or(1), 0));
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

            if pc & 127 == 0 && self.gc.should_collect() {
                self.collect_garbage();
            }

            let instruction = &module.instructions[pc];

            if cfg!(debug_assertions) && std::env::var("GEN_TRACE").is_ok() {
                eprintln!("[GEN_TRACE] pc={}, instr={:?}", pc, instruction);
            }

            match instruction {
                // OPTIMIZATION (Phase 1C): Hot-path instructions are inlined
                // directly in the main match to skip the cascading
                // `_ => exec_load_store()` dispatch (one branch per iter).
                Instruction::LoadLocal(slot) => {
                    if let Some(frame) = self.call_stack.last() {
                        let idx = frame.base_pointer + *slot as usize;
                        if let Some(value) = self.stack.get(idx) {
                            self.stack.push(value.clone());
                            pc += 1;
                            continue;
                        }
                    } else if let Some(value) = self.stack.get(*slot as usize) {
                        self.stack.push(value.clone());
                        pc += 1;
                        continue;
                    }
                    self.stack.push(Value::Undefined);
                    pc += 1;
                    continue;
                }
                Instruction::StoreLocal(slot) => {
                    if let Some(value) = self.stack.pop() {
                        let base = self.call_stack.last().map(|f| f.base_pointer).unwrap_or(0);
                        let idx = base + *slot as usize;
                        if idx >= self.stack.len() {
                            self.stack.resize(idx + 1, Value::Undefined);
                        }
                        self.stack[idx] = value;
                    } else {
                        return Err(
                            self.err_at_location(Error::RuntimeError("Stack underflow".into()))
                        );
                    }
                    pc += 1;
                    continue;
                }
                Instruction::IncLocal(slot, delta) => {
                    if let Some(frame) = self.call_stack.last() {
                        let idx = frame.base_pointer + *slot as usize;
                        if idx < self.stack.len() {
                            match &self.stack[idx] {
                                Value::Integer(n) => {
                                    self.stack[idx] = Value::Integer(n + delta);
                                    pc += 1;
                                    continue;
                                }
                                Value::Float(n) => {
                                    self.stack[idx] = Value::Float(n + *delta as f64);
                                    pc += 1;
                                    continue;
                                }
                                _ => {}
                            }
                        }
                    }
                    // Cold path: fall through to exec_load_store.
                }
                Instruction::AddLocal(dst, src) => {
                    let base = self.call_stack.last().map(|f| f.base_pointer).unwrap_or(0);
                    let dst_idx = base + *dst as usize;
                    let src_idx = base + *src as usize;
                    if dst_idx < self.stack.len() && src_idx < self.stack.len() {
                        match (&self.stack[dst_idx], &self.stack[src_idx]) {
                            (Value::Integer(a), Value::Integer(b)) => {
                                if let Some(result) = a.checked_add(*b) {
                                    self.stack[dst_idx] = Value::Integer(result);
                                } else {
                                    self.stack[dst_idx] = Value::Float(*a as f64 + *b as f64);
                                }
                                pc += 1;
                                continue;
                            }
                            (Value::Float(a), Value::Float(b)) => {
                                self.stack[dst_idx] = Value::Float(a + b);
                                pc += 1;
                                continue;
                            }
                            (Value::Integer(a), Value::Float(b)) => {
                                self.stack[dst_idx] = Value::Float(*a as f64 + *b);
                                pc += 1;
                                continue;
                            }
                            (Value::Float(a), Value::Integer(b)) => {
                                self.stack[dst_idx] = Value::Float(*a + *b as f64);
                                pc += 1;
                                continue;
                            }
                            // Phase 1.7: String concat via ConsString rope.
                            (Value::String(a), Value::String(b)) => {
                                self.stack[dst_idx] = Value::Cons(ConsString::new(
                                    Value::String(a.clone()),
                                    Value::String(b.clone()),
                                ));
                                pc += 1;
                                continue;
                            }
                            (Value::Cons(c), Value::String(b)) => {
                                self.stack[dst_idx] = Value::Cons(ConsString::new(
                                    Value::Cons(c.clone()),
                                    Value::String(b.clone()),
                                ));
                                pc += 1;
                                continue;
                            }
                            (Value::String(a), Value::Cons(c)) => {
                                self.stack[dst_idx] = Value::Cons(ConsString::new(
                                    Value::String(a.clone()),
                                    Value::Cons(c.clone()),
                                ));
                                pc += 1;
                                continue;
                            }
                            (Value::Cons(a), Value::Cons(b)) => {
                                self.stack[dst_idx] = Value::Cons(ConsString::new(
                                    Value::Cons(a.clone()),
                                    Value::Cons(b.clone()),
                                ));
                                pc += 1;
                                continue;
                            }
                            _ => {
                                // Cold path: fall through to exec_load_store.
                            }
                        }
                    }
                }
                // OPTIMIZATION (Phase 1G): Inline the most common `LoadConst`
                // pattern (Integer / Float / small String / Null / Undefined /
                // Boolean) directly in the dispatch so the cascading
                // `_ => exec_load_store()` branch is skipped. The cold path
                // (Object / Array / Function / …) delegates to
                // `exec_load_store` explicitly so the value is actually pushed
                // — a fall-through would skip the push and cause a Stack
                // underflow on the next consumer (this was a real bug in an
                // earlier version of this optimisation).
                //
                // Phase 1.3: `BigInt` and `Symbol` are also immediate values
                // (16 bytes / 8 bytes respectively) so we push a clone inline
                // instead of going through `exec_load_store`. The clone is
                // cheap (one discriminant + payload memcpy) and the call
                // savings are real for code that uses `const BIG = 100n` /
                // `const S = Symbol("x")` in a hot loop.
                Instruction::LoadConst(idx) => {
                    let cidx = *idx as usize;
                    if let Some(value) = module.constants.get(cidx) {
                        match value {
                            Value::Integer(_)
                            | Value::Float(_)
                            | Value::String(_)
                            | Value::Null
                            | Value::Undefined
                            | Value::Boolean(_)
                            | Value::BigInt(_)
                            | Value::Symbol(_) => {
                                self.stack.push(value.clone());
                                pc += 1;
                                continue;
                            }
                            _ => {
                                // Object, Array, Function, … — fall through
                                // to `exec_load_store` below.
                            }
                        }
                    }
                    // Cold path: explicitly call `exec_load_store` so the
                    // value (a BigInt / Object / …) is actually pushed.
                    if self.exec_load_store(instruction, module)? {
                        // exec_load_store returned true — it handled the
                        // instruction. Fall through to `pc += 1` below.
                    }
                }
                // OPTIMIZATION (Phase 1G): `Pop` is one of the most frequent
                // instructions after every expression statement. Inline it
                // so the dispatch is one branch + one pop.
                Instruction::Pop => {
                    self.stack.pop();
                    pc += 1;
                    continue;
                }
                // OPTIMIZATION (Phase 1H): `Dup` (duplicate the top of the
                // stack) is emitted for every `i++` / `++i` and for compound
                // assignments like `x += y` after the right-hand side is
                // pushed. Inlining avoids the cascading `_ =>
                // exec_load_store()` dispatch (one branch per iter).
                Instruction::Dup => {
                    let val = self.stack.last().cloned().unwrap_or(Value::Undefined);
                    self.stack.push(val);
                    pc += 1;
                    continue;
                }
                // OPTIMIZATION (Phase 1H): `LoadThis` is emitted at the start
                // of every non-arrow function body and on `super.X` accesses.
                // Inlining skips the call to `exec_load_store` and avoids the
                // `Option::clone()` of `f.this_value` (which the existing
                // path in `exec_load_store` does unconditionally even when
                // it's `None`).
                Instruction::LoadThis => {
                    let this = self
                        .call_stack
                        .last()
                        .and_then(|f| f.this_value.clone())
                        .unwrap_or(Value::Undefined);
                    self.stack.push(this);
                    pc += 1;
                    continue;
                }
                // OPTIMIZATION (Phase 1H): `Rot3Right` rotates the top three
                // stack values (used by short-circuit boolean and ternary
                // operators). Inlining avoids the function-call round-trip
                // and the 3 × 32-byte clones the old `exec_load_store`
                // arm did. The rotation is `(a, b, c) → (b, c, a)`:
                //   - stack[len-3] should hold b (the second)
                //   - stack[len-2] should hold c (the third)
                //   - stack[len-1] should hold a (the first)
                // We achieve this with three `std::mem::replace` moves
                // (no clones) plus one direct write. The order matters
                // because each replace must read its slot BEFORE it is
                // overwritten by the next step.
                Instruction::Rot3Right => {
                    let len = self.stack.len();
                    if len >= 3 {
                        // Step 1: read `a` out of stack[len-3] (replacing
                        //         it with Undefined), put `a` into
                        //         stack[len-1] (replacing the original
                        //         `c` and reading it out as `c`).
                        let a = std::mem::replace(&mut self.stack[len - 3], Value::Undefined);
                        let c = std::mem::replace(&mut self.stack[len - 1], a);
                        // Step 2: read `b` out of stack[len-2] (replacing
                        //         it with the original `c`).
                        let b = std::mem::replace(&mut self.stack[len - 2], c);
                        // Step 3: write `b` into stack[len-3] (the slot
                        //         that currently holds Undefined).
                        self.stack[len - 3] = b;
                        // Final: stack[len-3] = b, stack[len-2] = c,
                        //        stack[len-1] = a  →  (b, c, a) ✓
                    }
                    pc += 1;
                    continue;
                }
                Instruction::Jump(_)
                | Instruction::JumpIf(_)
                | Instruction::JumpIfNot(_)
                | Instruction::JumpIfUndefined(_)
                | Instruction::JumpIfNotUndefined(_)
                | Instruction::Return
                | Instruction::Yield => {
                    let mut pc_mut = pc;
                    match self.exec_control_flow(instruction, module, &mut pc_mut)? {
                        ControlFlowOutcome::Continue => {
                            pc = pc_mut;
                            continue;
                        }
                        ControlFlowOutcome::Return(v) => return Ok(v),
                        ControlFlowOutcome::Next => {}
                        ControlFlowOutcome::Jump(target) => {
                            pc = target;
                            continue;
                        }
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
                                                .borrow()
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
                                let gen_idx = self.heap.len();
                                self.heap.push(HeapValue::Generator(JsGenerator {
                                    yield_value: Value::Undefined,
                                    resume_pc: bytecode_index,
                                    saved_stack: args.clone(),
                                    saved_block_scope_stack: Vec::new(),
                                    func_heap_idx: Some(*func_idx),
                                    generator_yielded: false,
                                }));
                                self.stack.push(Value::Generator(gen_idx));
                                pc += 1;
                                continue;
                            } else if has_promise_resolve {
                                if let HeapValue::Function(f) = &self.heap[*func_idx] {
                                    let promise_idx = {
                                        let closure = f.closure.borrow();
                                        closure.first().and_then(|v| {
                                            if let Value::Promise(idx) = v { Some(*idx) } else { None }
                                        })
                                    };
                                    if let Some(promise_idx) = promise_idx {
                                        match f.name.as_deref() {
                                            Some("resolve") => {
                                                let val = args
                                                    .first()
                                                    .cloned()
                                                    .unwrap_or(Value::Undefined);
                                                self.resolve_promise(promise_idx, val);
                                                self.stack.push(Value::Undefined);
                                            }
                                            Some("reject") => {
                                                let reason = args
                                                    .first()
                                                    .cloned()
                                                    .unwrap_or(Value::Undefined);
                                                self.reject_promise(promise_idx, reason);
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
                                    let len = f.closure.borrow().len();
                                    if f.name.as_deref() == Some("bound") && len >= 2 {
                                        let (original_fn, bound_this, bound_args) = {
                                            let closure = f.closure.borrow();
                                            (
                                                closure[0].clone(),
                                                closure[1].clone(),
                                                closure[2..].to_vec(),
                                            )
                                        };
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
                                        let func_info = {
                                            if let HeapValue::Function(f) = &self.heap[*func_idx] {
                                                let closure_rc = if f.closure.borrow().is_empty() {
                                                    Rc::new(RefCell::new(Vec::new()))
                                                } else {
                                                    f.closure.clone()
                                                };
                                                let captured_this = if f.is_arrow {
                                                    f.captured_this.clone()
                                                } else {
                                                    None
                                                };
                                                Some((
                                                    closure_rc,
                                                    f.bytecode_index,
                                                    f.is_arrow,
                                                    captured_this,
                                                    f.rest_param.is_some(),
                                                    f.params.len(),
                                                ))
                                            } else {
                                                None
                                            }
                                        };
                                    if let Some((
                                        closure_vars,
                                        bytecode_index,
                                        is_arrow,
                                        captured_this,
                                        has_rest,
                                        param_count,
                                    )) = func_info
                                    {
                                        let return_address = pc + 1;
                                        let base_pointer = self.stack.len();
                                        let closure_count = closure_vars.borrow().len();
                                        let this_for_frame = if is_arrow {
                                            captured_this.unwrap_or(Value::Undefined)
                                        } else {
                                            Value::Undefined
                                        };
                                        if self.call_stack.len() >= self.max_call_stack_depth {
                                            self.throw_stack_overflow(&mut pc)?;
                                            continue;
                                        }
                                        self.call_stack.push(CallFrame {
                                            return_address,
                                            base_pointer,
                                            closure_var_count: closure_count,
                                            func_heap_idx: Some(*func_idx),
                                            this_value: Some(this_for_frame),
                                            is_construct: false,
                                            source_name: self.current_module_path.clone(),
                                            generator_heap_idx: None,
                                            source_line: self.current_source_line(pc),
                                            source_col: self.current_source_col(pc),
                                            // Phase 2C (inline Call fast path):
                                            // skip the `Vec::clone()` of
                                            // `self.exception_handlers` when
                                            // empty (the common case for code
                                            // without try/catch).
                                            exception_handlers_snapshot: if self
                                                .exception_handlers
                                                .is_empty()
                                            {
                                                Vec::new()
                                            } else {
                                                self.exception_handlers.clone()
                                            },
                                            shared_closure_env: HashMap::new(),
                                        });
                                        for closure_var in closure_vars.borrow().iter().cloned() {
                                            self.stack.push(closure_var);
                                        }
                                        if has_rest {
                                            for arg in args.iter().take(param_count) {
                                                self.stack.push(arg.clone());
                                            }
                                            let rest_args: Vec<Value> =
                                                args[param_count..].to_vec();
                                            let rest_arr_idx = self.gc.allocate(
                                                &mut self.heap,
                                                HeapValue::Array(JsArray {
                                                    elements: rest_args,
                                                }),
                                            );
                                            self.stack.push(Value::Array(rest_arr_idx));
                                        } else {
                                            for arg in args {
                                                self.stack.push(arg);
                                            }
                                        }
                                        pc = bytecode_index;
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
                    // Phase 7D: pre-allocate the args Vec with the exact
                    // capacity so the first `push` does not reallocate from
                    // 0 → 1 → 2 → 4. Matters for hot method calls like
                    // `arr.push(...)` and `m.set(...)` in the `map_set` and
                    // `array_push` benchmarks. `*argc` is `u16` so cast to
                    // `usize` for `Vec::with_capacity`.
                    let mut args = Vec::with_capacity(usize::from(*argc));
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
                                let closure_rc = if f.closure.borrow().is_empty() {
                                    Rc::new(RefCell::new(Vec::new()))
                                } else {
                                    f.closure.clone()
                                };
                                let captured_this = if f.is_arrow {
                                    f.captured_this.clone()
                                } else {
                                    None
                                };
                                let func_info = Some((
                                    closure_rc,
                                    f.bytecode_index,
                                    f.is_arrow,
                                    captured_this,
                                    f.rest_param.is_some(),
                                    f.params.len(),
                                ));
                                let same_module = match (&f.owner_module, &self.current_module) {
                                    (Some(om), Some(cm)) => Rc::ptr_eq(om, cm),
                                    (None, None) => true,
                                    _ => false,
                                };
                                if same_module {
                                    if let Some((
                                        closure_vars,
                                        bytecode_index,
                                        is_arrow,
                                        captured_this,
                                        has_rest,
                                        param_count,
                                    )) = func_info
                                    {
                                        let return_address = pc + 1;
                                        let base_pointer = self.stack.len();
                                        let closure_count = closure_vars.borrow().len();
                                        let this_for_frame = if is_arrow {
                                            captured_this.unwrap_or_else(|| object.clone())
                                        } else {
                                            object.clone()
                                        };
                                        if self.call_stack.len() >= self.max_call_stack_depth {
                                            self.throw_stack_overflow(&mut pc)?;
                                            continue;
                                        }
                                        self.call_stack.push(CallFrame {
                                            return_address,
                                            base_pointer,
                                            closure_var_count: closure_count,
                                            func_heap_idx: Some(func_idx),
                                            this_value: Some(this_for_frame),
                                            is_construct: false,
                                            source_name: self.current_module_path.clone(),
                                            generator_heap_idx: None,
                                            source_line: self.current_source_line(pc),
                                            source_col: self.current_source_col(pc),
                                            // Phase 2C (inline CallMethod fast
                                            // path): skip the `Vec::clone()` of
                                            // `self.exception_handlers` when
                                            // empty (the common case for code
                                            // without try/catch).
                                            exception_handlers_snapshot: if self
                                                .exception_handlers
                                                .is_empty()
                                            {
                                                Vec::new()
                                            } else {
                                                self.exception_handlers.clone()
                                            },
                                            shared_closure_env: HashMap::new(),
                                        });
                                        for closure_var in closure_vars.borrow().iter().cloned() {
                                            self.stack.push(closure_var);
                                        }
                                        if has_rest {
                                            for arg in args.iter().take(param_count) {
                                                self.stack.push(arg.clone());
                                            }
                                            let rest_args: Vec<Value> =
                                                args[param_count..].to_vec();
                                            let rest_arr_idx = self.gc.allocate(
                                                &mut self.heap,
                                                HeapValue::Array(JsArray {
                                                    elements: rest_args,
                                                }),
                                            );
                                            self.stack.push(Value::Array(rest_arr_idx));
                                        } else {
                                            for arg in args {
                                                self.stack.push(arg);
                                            }
                                        }
                                        pc = bytecode_index;
                                        continue;
                                    }
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
                                                    if self.call_stack.len()
                                                        >= self.max_call_stack_depth
                                                    {
                                                        self.throw_stack_overflow(&mut pc)?;
                                                        continue;
                                                    }
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
                                                        source_col: self.current_source_col(pc),
                                                        exception_handlers_snapshot: self
                                                            .exception_handlers
                                                            .clone(),
                                                        shared_closure_env: HashMap::new(),
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
                                    let func_info = {
                                        if let HeapValue::Function(f) = &self.heap[*func_idx] {
                                            Some((f.closure.clone(), f.bytecode_index))
                                        } else {
                                            None
                                        }
                                    };
                                    if let Some((closure_vars, bytecode_index)) = func_info {
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
                                            let return_address = pc + 1;
                                            let base_pointer = self.stack.len();
                                            let closure_count = closure_vars.borrow().len();
                                            if self.call_stack.len() >= self.max_call_stack_depth {
                                                self.throw_stack_overflow(&mut pc)?;
                                                continue;
                                            }
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
                                                source_col: self.current_source_col(pc),
                                                exception_handlers_snapshot: self
                                                    .exception_handlers
                                                    .clone(),
                                                shared_closure_env: HashMap::new(),
                                            });
                                            for closure_var in closure_vars.borrow().iter().cloned() {
                                                self.stack.push(closure_var);
                                            }
                                            for arg in args {
                                                self.stack.push(arg);
                                            }
                                            pc = bytecode_index;
                                            continue;
                                        }
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
                                | Value::TypedArray(_)
                                | Value::NativeObject(_) => {
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
                Instruction::MakeClass(_)
                | Instruction::SuperConstruct(_)
                | Instruction::SuperGet => {
                    let mut pc_mut = pc;
                    self.exec_class_ops(instruction, &mut pc_mut, module)?;
                    if pc_mut != pc {
                        pc = pc_mut;
                        continue;
                    }
                }
                Instruction::ImportModule(source) => {
                    let module_obj = self.exec_import_module(source)?.unwrap_or(Value::Undefined);
                    self.stack.push(module_obj);
                }
                Instruction::ImportNamed(source, imported_name, local_name) => {
                    let _ = self.exec_import_named(source, imported_name, local_name)?;
                }
                Instruction::ImportDefault(source, local_name) => {
                    let _ = self.exec_import_default(source, local_name)?;
                }
                Instruction::ImportAll(source, local_name) => {
                    let _ = self.exec_import_all(source, local_name)?;
                }
                Instruction::NativeImport(source, local_name) => {
                    self.exec_native_import(source, local_name)?;
                }
                Instruction::ExportNamed(names) => {
                    self.exec_export_named(names.as_slice())?;
                }
                Instruction::ExportDefault => {
                    self.exec_export_default()?;
                }
                Instruction::StoreModuleExport(name) => {
                    self.exec_store_module_export(name)?;
                }
                Instruction::ReExportAll(source) => {
                    self.exec_reexport_all(source)?;
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
                                    self.pending_exception = Some(reason.clone());
                                    if self.handle_pending_exception(&mut pc)? {
                                        continue;
                                    }
                                }
                                PromiseState::Pending => {
                                    let frame = SuspendedFrame {
                                        promise_idx: *promise_idx,
                                        resume_pc: pc + 1,
                                        stack_snapshot: std::mem::take(&mut self.stack),
                                        call_stack_snapshot: std::mem::take(&mut self.call_stack),
                                        module: self.current_module.clone(),
                                        module_path: self.current_module_path.clone(),
                                        exception_handlers_snapshot: std::mem::take(
                                            &mut self.exception_handlers,
                                        ),
                                        block_scope_stack_snapshot: std::mem::take(
                                            &mut self.block_scope_stack,
                                        ),
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
                            let promise = self.build_module_promise(exports);
                            self.stack.push(promise);
                        }
                        Ok(None) => {
                            let promise = self
                                .build_error_promise(format!("Module '{}' not found", source_str));
                            self.stack.push(promise);
                        }
                        Err(e) => {
                            let promise = self.build_error_promise(format!(
                                "Module '{}' error: {}",
                                source_str, e
                            ));
                            self.stack.push(promise);
                        }
                    }
                }
                Instruction::GetIterator => {
                    let iterable = self
                        .stack
                        .pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let iter = self.exec_get_iterator(iterable)?;
                    self.stack.push(iter);
                }
                Instruction::GetAsyncIterator => {
                    let iterable = self
                        .stack
                        .pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    let iter = self.exec_get_async_iterator(iterable)?;
                    self.stack.push(iter);
                }
                Instruction::IteratorNext(target) => {
                    let iterator = self
                        .stack
                        .last()
                        .cloned()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    match self.exec_iterator_next(iterator, *target as usize)? {
                        ControlFlowOutcome::Jump(jump_target) => {
                            self.stack.pop();
                            pc = jump_target;
                            continue;
                        }
                        ControlFlowOutcome::Next => {}
                        _ => {}
                    }
                }
                Instruction::IteratorClose => {
                    let iterator = self
                        .stack
                        .pop()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    self.exec_iterator_close(iterator)?;
                }
                Instruction::AsyncIteratorNext(target) => {
                    let iterator = self
                        .stack
                        .last()
                        .cloned()
                        .ok_or_else(|| Error::RuntimeError("Stack underflow".into()))?;
                    match self.exec_async_iterator_next(iterator, *target as usize)? {
                        ControlFlowOutcome::Jump(jump_target) => {
                            self.stack.pop();
                            pc = jump_target;
                            continue;
                        }
                        ControlFlowOutcome::Next => {}
                        _ => {}
                    }
                }
                _ =>
                {
                    #[allow(clippy::if_same_then_else)]
                    if self.exec_load_store(instruction, module)? {
                    } else if self.exec_arithmetic(instruction)? {
                    } else if self.exec_comparison(instruction)? {
                    } else if self.exec_property_ops(instruction)? {
                    } else if self.exec_make_function(instruction, module, pc)? {
                    } else if self.exec_class_ops(instruction, &mut pc, module)? {
                    } else {
                        let saved_pc = pc;
                        if self.exec_exception(instruction, &mut pc)? {
                            if pc != saved_pc {
                                continue;
                            }
                        } else if self.handle_pending_exception(&mut pc)? {
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
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new().expect("Failed to create default interpreter")
    }
}
