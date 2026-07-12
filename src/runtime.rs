use crate::compiler::type_checker::Type;
use crate::compiler::{CompiledModule, Compiler};
use crate::errors::Result;
use crate::objects::Value;
use crate::vm::{EventSource, Interpreter};
use crate::well_known as wk;
use rustc_hash::FxHashMap;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::rc::Rc;

pub struct RuntimeConfig {
    pub enable_type_checking: bool,
    pub max_heap_size: usize,
    pub max_call_stack_depth: usize,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            enable_type_checking: false,
            max_heap_size: 0,
            max_call_stack_depth: wk::DEFAULT_MAX_CALL_STACK_DEPTH,
        }
    }
}

pub struct TailsRuntime {
    interpreter: Interpreter,
    config: RuntimeConfig,
    /// Long-lived event sources registered by native modules (http, net, …).
    /// Polled by [`run_event_loop`] after the top-level script finishes.
    event_sources: Vec<Box<dyn EventSource>>,
    /// Source-hash → compiled module cache for repeated `eval` of the same script.
    eval_cache: FxHashMap<u64, Rc<CompiledModule>>,
    eval_cache_order: Vec<u64>,
}

impl TailsRuntime {
    pub fn new(config: RuntimeConfig) -> Result<Self> {
        let mut interpreter = Interpreter::new()?;
        if config.max_call_stack_depth > 0 {
            interpreter.max_call_stack_depth = config.max_call_stack_depth;
        }
        Ok(Self {
            interpreter,
            config,
            event_sources: Vec::new(),
            eval_cache: FxHashMap::default(),
            eval_cache_order: Vec::new(),
        })
    }

    fn source_hash(source: &str) -> u64 {
        let mut hasher = rustc_hash::FxHasher::default();
        source.hash(&mut hasher);
        hasher.finish()
    }

    pub fn eval(&mut self, source: &str) -> Result<Value> {
        // Skip the compile cache when type-checking is on: known globals can
        // change between evals and would invalidate cached bytecode types.
        let compiled = if self.config.enable_type_checking {
            let mut compiler = Compiler::new(true);
            let globals: FxHashMap<String, Type> = self
                .interpreter
                .globals
                .keys()
                .map(|k| (k.clone(), Type::Any))
                .collect();
            compiler.set_known_globals(globals);
            Rc::new(compiler.compile(source)?)
        } else {
            let hash = Self::source_hash(source);
            if let Some(cached) = self.eval_cache.get(&hash) {
                cached.clone()
            } else {
                let compiler = Compiler::new(false);
                let module = Rc::new(compiler.compile(source)?);
                if self.eval_cache.len() >= wk::EVAL_CACHE_MAX {
                    if let Some(old) = self.eval_cache_order.first().copied() {
                        self.eval_cache.remove(&old);
                        self.eval_cache_order.remove(0);
                    }
                }
                self.eval_cache.insert(hash, module.clone());
                self.eval_cache_order.push(hash);
                module
            }
        };
        match self.interpreter.execute(&compiled) {
            Ok(val) => Ok(val),
            Err(e) => {
                let backtrace = self.interpreter.call_stack_backtrace();
                if backtrace.is_empty() {
                    Err(e)
                } else {
                    Err(e.with_backtrace(backtrace))
                }
            }
        }
    }

    pub fn eval_module(&mut self, source: &str, base_path: &Path) -> Result<Value> {
        let module_key = base_path.to_string_lossy().to_string();
        let prev = self.interpreter.current_module_path.clone();
        self.interpreter.current_module_path = Some(module_key.clone());
        let compiler = Compiler::new(self.config.enable_type_checking);
        let compiled = compiler.compile(source)?;
        let result = self.interpreter.execute_module(&compiled);
        // Register module exports
        let exports = std::mem::take(&mut self.interpreter.module_exports);
        let final_result = match result {
            Ok(val) => {
                if matches!(val, Value::Undefined) {
                    exports.get("default").cloned().unwrap_or(Value::Undefined)
                } else {
                    val
                }
            }
            Err(e) => {
                let backtrace = self.interpreter.call_stack_backtrace();
                self.interpreter.module_registry.insert(module_key, exports);
                self.interpreter.current_module_path = prev;
                if backtrace.is_empty() {
                    return Err(e);
                }
                return Err(e.with_backtrace(backtrace));
            }
        };
        self.interpreter.module_registry.insert(module_key, exports);
        self.interpreter.current_module_path = prev;
        Ok(final_result)
    }

    pub fn import(&mut self, module_path: &Path) -> Result<Value> {
        let source = std::fs::read_to_string(module_path).map_err(|e| {
            crate::errors::Error::RuntimeError(format!("Failed to read module: {}", e))
        })?;
        self.eval_module(&source, module_path)
    }

    pub fn get_global(&self, name: &str) -> Option<Value> {
        self.interpreter.get_global(name)
    }

    pub fn set_global(&mut self, name: &str, value: Value) {
        self.interpreter.set_global(name, value);
    }

    pub fn get_module_export(&self, module_path: &str, name: &str) -> Option<Value> {
        self.interpreter
            .module_registry
            .get(module_path)
            .and_then(|exports| exports.get(name).cloned())
    }

    pub fn new_object(&mut self) -> Value {
        self.interpreter.new_object()
    }

    pub fn new_array(&mut self) -> Value {
        self.interpreter.new_array()
    }

    pub fn get_property(&mut self, object: &Value, key: &str) -> Option<Value> {
        self.interpreter.get_property_str(object, key)
    }

    pub fn set_property(&mut self, object: &Value, key: &str, value: Value) {
        self.interpreter.set_property_str(object, key, value);
    }

    pub fn get_array_length(&self, array: &Value) -> Option<i64> {
        self.interpreter.get_array_length(array)
    }

    pub fn get_array_element(&self, array: &Value, index: usize) -> Option<Value> {
        self.interpreter.get_array_element(array, index)
    }

    pub fn push_array_element(&mut self, array: &Value, value: Value) {
        self.interpreter.push_array_element(array, value);
    }

    pub fn call_function(&mut self, func: &Value, this: &Value, args: &[Value]) -> Result<Value> {
        self.interpreter.call_value(func, this, args)
    }

    pub fn call_global(&mut self, name: &str, args: &[Value]) -> Result<Value> {
        let func = self.get_global(name).ok_or_else(|| {
            crate::errors::Error::RuntimeError(format!("Function '{}' not found in globals", name))
        })?;
        self.call_function(&func, &Value::Undefined, args)
    }

    /// Returns `true` when the runtime still has work that keeps the process
    /// alive: registered event sources, pending timers, or queued microtasks.
    pub fn has_pending_work(&self) -> bool {
        self.event_sources.iter().any(|s| s.is_active())
            || !self.interpreter.pending_event_sources.is_empty()
            || self.interpreter.async_runtime.has_pending_timers()
            || !self.interpreter.async_runtime.is_idle()
    }

    /// Return the interpreter's current JS call-stack backtrace as a string
    /// (used for diagnostics when the event loop surfaces an error).
    pub fn get_interpreter_backtrace(&self) -> Option<String> {
        let bt = self.interpreter.call_stack_backtrace();
        if bt.is_empty() {
            None
        } else {
            Some(format!("\n{}", bt))
        }
    }

    /// Run the event loop until all registered event sources are idle and
    /// there are no more pending timers or microtasks.
    ///
    /// This is called after the top-level script/module finishes to keep the
    /// process alive for long-running services (HTTP servers, TCP connections,
    /// scheduled timers, …).
    ///
    /// Returns early when `process.exit` or SIGINT/SIGTERM request termination.
    /// The requested status is available via
    /// [`crate::runtime_env::native_fns::process_fns::take_exit_code`].
    pub fn run_event_loop(&mut self) -> Result<()> {
        use crate::runtime_env::native_fns::process_fns::{exit_requested, take_exit_code};

        // Drain any sources registered during script execution.
        self.drain_pending_sources();

        while self.has_pending_work() {
            // process.exit / Ctrl+C / SIGTERM — stop the loop cleanly.
            if exit_requested() {
                let code = take_exit_code();
                // Hard exit so open listeners cannot re-enter the loop.
                let _ = std::io::Write::flush(&mut std::io::stdout());
                let _ = std::io::Write::flush(&mut std::io::stderr());
                std::process::exit(code);
            }

            // Drain sources that may have been added during this tick.
            self.drain_pending_sources();

            // Poll every registered event source.
            for source in self.event_sources.iter_mut() {
                if source.is_active() {
                    source.poll(&mut self.interpreter)?;
                }
            }

            // Drain microtasks (Promise continuations, queueMicrotask, …).
            self.interpreter.drain_microtasks();

            // Fire any ready macrotasks (setTimeout callbacks).
            // process.exit inside a timer never returns (hard-exits).
            let macrotasks = self.interpreter.async_runtime.run_macrotasks();
            for task in macrotasks {
                let _ = self
                    .interpreter
                    .call_value(&task.callback, &Value::Undefined, &[]);
                if exit_requested() {
                    let code = take_exit_code();
                    let _ = std::io::Write::flush(&mut std::io::stdout());
                    let _ = std::io::Write::flush(&mut std::io::stderr());
                    std::process::exit(code);
                }
            }

            // Remove inactive sources so they are not polled again.
            self.event_sources.retain(|s| s.is_active());

            // Sleep until the next timer (capped) so we don't busy-spin, but
            // wake often enough to notice SIGINT/SIGTERM and due timers.
            let sleep_ms = self
                .interpreter
                .async_runtime
                .next_timer_delay_ms()
                .unwrap_or(50)
                .clamp(1, 50);
            std::thread::sleep(std::time::Duration::from_millis(sleep_ms));
        }

        Ok(())
    }

    /// Move event sources from the interpreter's pending queue into the
    /// runtime's active source list.
    fn drain_pending_sources(&mut self) {
        let pending = std::mem::take(&mut self.interpreter.pending_event_sources);
        self.event_sources.extend(pending);
    }
}

impl Default for TailsRuntime {
    fn default() -> Self {
        Self::new(RuntimeConfig::default()).expect("Failed to create default runtime")
    }
}
