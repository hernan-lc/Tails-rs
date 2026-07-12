mod builtins;
mod bytecode;
mod call_frame;
mod calls;
mod class_ops;
mod collection_ops;
mod control_flow;
mod error_format;
mod exception_handling;
mod exec_calls;
mod function_ops;
pub mod heap_types;
mod instructions;
mod iterators;
mod modules;
pub(crate) mod native_loader;
mod ops;
mod promise_runtime;
pub(crate) mod property_access;
pub mod safe_function;
pub mod safe_library;
mod value_ops;

#[allow(dead_code)]
pub(crate) const ERR_STACK_UNDERFLOW: &str = "Stack underflow";
pub(crate) const ERR_DIV_BY_ZERO: &str = "Division by zero";

pub(crate) use call_frame::{CallFrame, ExceptionHandler, SuspendedFrame};
pub use heap_types::{
    HeapValue, JsArray, JsCompiledRegex, JsFunction, JsGenerator, JsIterator, JsObject,
    JsProxyData, JsRegExp, PropertyStorage,
};

use crate::compiler::{CompiledModule, Instruction};
use crate::errors::runtime_errors::runtime_error_stack_overflow;
use crate::errors::{Error, Result};
use crate::objects::js_promise::PromiseState;
use crate::objects::{ConsString, Value};
use crate::runtime_env::async_runtime::AsyncRuntime;
use crate::vm::interpreter::control_flow::ControlFlowOutcome;
use rustc_hash::FxHashMap;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

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

pub struct Interpreter {
    pub(crate) globals: FxHashMap<String, Value>,
    pub(crate) stack: Vec<Value>,
    pub(crate) heap: Vec<HeapValue>,
    pub(crate) gc: crate::vm::gc::GarbageCollector,
    pub(crate) call_stack: Vec<CallFrame>,
    pub(crate) current_module: Option<Rc<CompiledModule>>,
    pub(crate) exception_handlers: Vec<ExceptionHandler>,
    pub(crate) pending_exception: Option<Value>,
    pub(crate) async_runtime: AsyncRuntime,
    pub(crate) _promise_stack: Vec<usize>,
    _timer_id_counter: u32,
    pub(crate) module_registry: HashMap<String, FxHashMap<String, Value>>,
    pub(crate) module_exports: FxHashMap<String, Value>,
    pub(crate) current_module_path: Option<String>,
    pub(crate) module_globals: Option<Rc<RefCell<FxHashMap<String, Value>>>>,
    pub(crate) module_globals_rc: Option<Rc<RefCell<FxHashMap<String, Value>>>>,
    /// Names bound by `import * as ns` (module-namespace objects). These
    /// bindings must never be clobbered by an imported module's own export
    /// that happens to share the same name (e.g. `import * as parse` vs
    /// `export const parse`).
    pub(crate) namespace_globals: std::collections::HashSet<String>,
    pub(crate) require_cache: FxHashMap<String, Value>,
    pub(crate) block_scope_stack: Vec<usize>,
    pub(crate) next_symbol_id: u64,
    pub(crate) symbol_registry: HashMap<String, u64>,
    pub(crate) date_proto_idx: Option<usize>,
    pub(crate) regexp_proto_idx: Option<usize>,
    pub(crate) buffer_proto_idx: Option<usize>,
    pub(crate) generator_proto_idx: Option<usize>,
    /// `Error.prototype` heap index (shared by Error subclasses).
    pub(crate) error_proto_idx: Option<usize>,
    /// `Object.prototype` heap index (for module exports / plain objects).
    pub(crate) object_proto_idx: Option<usize>,
    pub(crate) boolean_proto_idx: Option<usize>,
    pub(crate) number_proto_idx: Option<usize>,
    pub(crate) string_proto_idx: Option<usize>,
    pub(crate) function_proto_idx: Option<usize>,
    pub(crate) array_proto_idx: Option<usize>,
    pub(crate) bigint_proto_idx: Option<usize>,
    pub(crate) symbol_proto_idx: Option<usize>,
    pub(crate) native_loader: native_loader::NativeModuleRegistry,
    pub(crate) current_pc: usize,
    pub(crate) suspended_frames: VecDeque<SuspendedFrame>,
    pub(crate) max_call_stack_depth: usize,
    /// Dynamically loaded native functions (typed C ABI — no transmute at call sites).
    pub(crate) dynamic_native_fns: Vec<tails_abi::NativeFn>,
    pub(crate) native_object_methods: HashMap<u32, FxHashMap<String, Value>>,
    pub(crate) native_class_registry: HashMap<String, FxHashMap<String, Value>>,
    /// Holds event sources registered by native modules (http, net, websocket, …)
    /// during their listen/connect calls. Drained by the event loop in
    /// [`TailsRuntime::run_event_loop`] after script execution finishes.
    pub(crate) pending_event_sources: Vec<Box<dyn EventSource>>,
    /// Baseline JIT compiler for hot loops (feature `jit`).
    #[cfg(feature = "jit")]
    pub(crate) jit: crate::vm::jit::JitCompiler,
    /// `Error.stackTraceLimit` — max frames for captureStackTrace (V8/Node).
    pub(crate) error_stack_trace_limit: i64,
    /// `Error.prepareStackTrace` — optional formatter `(err, sites) => any`.
    pub(crate) error_prepare_stack_trace: Option<Value>,
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
            namespace_globals: std::collections::HashSet::new(),
            current_module_path: None,
            require_cache: FxHashMap::default(),
            block_scope_stack: Vec::new(),
            next_symbol_id: crate::objects::USER_SYMBOL_START,
            symbol_registry: HashMap::new(),
            date_proto_idx: None,
            regexp_proto_idx: None,
            buffer_proto_idx: None,
            generator_proto_idx: None,
            error_proto_idx: None,
            object_proto_idx: None,
            boolean_proto_idx: None,
            number_proto_idx: None,
            string_proto_idx: None,
            function_proto_idx: None,
            array_proto_idx: None,
            bigint_proto_idx: None,
            symbol_proto_idx: None,
            native_loader: native_loader::NativeModuleRegistry::new(),
            current_pc: 0,
            suspended_frames: VecDeque::new(),
            max_call_stack_depth: 10_000,
            dynamic_native_fns: Vec::new(),
            native_object_methods: HashMap::new(),
            native_class_registry: HashMap::new(),
            pending_event_sources: Vec::new(),
            #[cfg(feature = "jit")]
            jit: crate::vm::jit::JitCompiler::new(),
            error_stack_trace_limit: 10,
            error_prepare_stack_trace: None,
        };
        interp.init_builtins();
        Ok(interp)
    }

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
            arguments: None,
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

            // Honor a Ctrl+C / SIGTERM / `process.exit` request even while this
            // inner async loop is the one driving timers. Otherwise the
            // cooperative exit flag set by the signal handler is only checked in
            // `TailsRuntime::run_event_loop`, which is never reached for scripts
            // that keep pending timers (setInterval / setTimeout) — so the
            // process would ignore Ctrl+C until the timers finished.
            #[cfg(feature = "process")]
            {
                use std::io::Write;
                if crate::runtime_env::native_fns::process_fns::exit_requested() {
                    let code =
                        crate::runtime_env::native_fns::process_fns::take_exit_code();
                    let _ = std::io::stdout().flush();
                    let _ = std::io::stderr().flush();
                    std::process::exit(code);
                }
            }

            if !any_resumed && self.suspended_frames.is_empty() && self.async_runtime.is_idle() {
                break;
            }

            // Don't busy-spin while waiting for the next timer: sleep until it is
            // due (capped) so we wake promptly for due timers and for signals.
            let sleep_ms = self
                .async_runtime
                .next_timer_delay_ms()
                .unwrap_or(200)
                .min(50)
                .max(1);
            std::thread::sleep(std::time::Duration::from_millis(sleep_ms));
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
        // GC path is hot under allocation-heavy workloads (closures, arrays).
        // Never print by default — stderr spam made benches look hung/leaky.
        // Enable with: TAILS_GC_TRACE=1
        static GC_TRACE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        let trace = *GC_TRACE.get_or_init(|| std::env::var_os("TAILS_GC_TRACE").is_some());
        if trace {
            eprintln!(
                "[GC] BEFORE heap={} globals={} stack={} allocs={} threshold={}",
                self.heap.len(),
                self.globals.len(),
                self.stack.len(),
                self.gc.allocation_count,
                self.gc.threshold
            );
        }
        // Build the GC root set for global bindings. `self.globals` holds the
        // built-in globals (Object, Array, …) as well as module-level function
        // and export bindings that were merged back after each module finished
        // executing. `module_globals_rc` holds the *currently executing*
        // module's transient globals. Both must be marked: otherwise live
        // module-level functions (e.g. a `const` holding a constructor) get
        // collected and their heap slot is later reused, causing dangling
        // references that read the wrong value.
        let mut root_globals = self.globals.clone();
        self.add_proto_roots(&mut root_globals);
        if let Some(ref mg) = self.module_globals_rc {
            for (k, v) in mg.borrow().iter() {
                root_globals.insert(k.clone(), v.clone());
            }
        }
        // The module registry holds every module's live exports (functions,
        // namespace objects, …) as heap references. These must be marked as
        // roots too, otherwise cross-module exports get collected and their
        // heap slots are reused, leaving dangling references.
        for exports in self.module_registry.values() {
            for v in exports.values() {
                // Insert under a synthetic key so the value is treated as a root.
                root_globals.insert(format!("__module_export_{}", root_globals.len()), v.clone());
            }
        }
        // Also keep any module-globals maps that are still held (e.g. the
        // currently executing module's own globals) reachable.
        let globals_ref = std::rc::Rc::new(std::cell::RefCell::new(root_globals));
        let globals_snapshot = globals_ref.borrow();
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
        let freed = self.gc.collect(
            &mut self.heap,
            &globals_snapshot,
            &stack_snapshot,
            &call_stack_snapshot,
        );
        if trace {
            eprintln!(
                "[GC] AFTER heap={} freed={} live={}",
                self.heap.len(),
                freed,
                self.gc.live_count(self.heap.len())
            );
        }
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

    pub fn get_global(&self, name: &str) -> Option<Value> {
        self.globals.get(name).cloned()
    }

    pub fn set_global(&mut self, name: &str, value: Value) {
        if let Some(ref mg) = self.module_globals {
            mg.borrow_mut().insert(name.to_string(), value.clone());
        }
        // Never clobber an `import * as ns` namespace binding with a plain
        // value (e.g. an imported module's own same-named export). The
        // namespace object itself is allowed through.
        if self.namespace_globals.contains(name) {
            let is_namespace = matches!(value, Value::Object(idx) if matches!(
                &self.heap[idx],
                HeapValue::Object(o) if o.properties.contains_key("__module_path")
            ));
            if !is_namespace {
                return;
            }
        }
        self.globals.insert(name.to_string(), value);
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new().expect("Failed to create default interpreter")
    }
}
