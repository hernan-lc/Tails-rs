use std::sync::{Mutex, OnceLock};

use crate::errors::Result;
use crate::objects::Value;
use crate::vm::interpreter::Interpreter;

use super::helpers::to_string_value;

// ---------------------------------------------------------------------------
// `process.on('exit', ...)` handler registry
// ---------------------------------------------------------------------------
//
// Unlike an `EventEmitter`, exit handlers are *append-only* and the
// only event ever emitted is `"exit"`. We use a process-global
// `Mutex<Vec<Value>>` so a script can call `process.on('exit', fn)`
// from any module without needing a per-interpreter registry.
//
// Handlers are invoked in LIFO order (matching Node's behaviour: most
// recently registered runs first).

static EXIT_HANDLERS: OnceLock<Mutex<Vec<Value>>> = OnceLock::new();

fn exit_handlers() -> &'static Mutex<Vec<Value>> {
    EXIT_HANDLERS.get_or_init(|| Mutex::new(Vec::new()))
}

/// Drains and runs all registered exit handlers. Called by
/// [`native_process_exit`] and also exposed for the runtime's graceful
/// shutdown path so handlers run on a normal `tails` shutdown as well.
pub fn run_exit_handlers(interp: &mut Interpreter) {
    // Move the handlers out so that any handler that itself registers
    // more handlers (or throws) does not reentrantly re-run the same
    // callbacks.
    let handlers = {
        let mut g = match exit_handlers().lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        std::mem::take(&mut *g)
    };
    for handler in handlers {
        let _ = interp.call_value(&handler, &Value::Undefined, &[]);
    }
}

pub(super) fn native_process_exit(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let code = match args.first() {
        Some(Value::Integer(n)) => *n as i32,
        Some(Value::Float(n)) => *n as i32,
        _ => 0,
    };
    // Run user-registered exit handlers before terminating. We do
    // not propagate their return value (Node also discards it) and
    // swallow any errors so a misbehaving handler does not prevent
    // the process from actually exiting.
    run_exit_handlers(interp);
    std::process::exit(code);
}

pub(super) fn native_process_cwd(
    _interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    match tails_process::cwd() {
        Ok(path) => Ok(Value::String(path)),
        Err(e) => Err(crate::errors::Error::RuntimeError(format!(
            "cwd failed: {}",
            e
        ))),
    }
}

pub(super) fn native_process_chdir(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let dir = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    match tails_process::chdir(&dir) {
        Ok(()) => Ok(Value::Undefined),
        Err(e) => Err(crate::errors::Error::RuntimeError(format!(
            "chdir failed: {}",
            e
        ))),
    }
}

pub(super) fn native_process_stdout_write(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let data = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    let _ = tails_process::stdout_write(&data);
    Ok(Value::Boolean(true))
}

pub(super) fn native_process_hrtime(
    interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let (secs, nanos) = tails_process::hrtime();
    let arr_idx = interp.heap.len();
    interp.heap.push(crate::vm::interpreter::HeapValue::Array(
        crate::vm::interpreter::JsArray {
            elements: vec![Value::Integer(secs as i64), Value::Integer(nanos as i64)],
        },
    ));
    Ok(Value::Array(arr_idx))
}

pub(super) fn native_process_hrtime_bigint(
    _interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    Ok(Value::BigInt(tails_process::hrtime_bigint() as i128))
}

pub(super) fn native_process_next_tick(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    if let Some(callback) = args.first() {
        let _ = interp.call_value(callback, &Value::Undefined, &[]);
    }
    Ok(Value::Undefined)
}

// ===========================================================================
// New API: process.kill / process.uptime / process.memoryUsage /
//          process.on('exit', ...)
// ===========================================================================

/// `process.kill(pid, signal)` — dispatches a signal to a process.
/// Accepts either a numeric signal (e.g. `9`, `15`) or a POSIX signal
/// name (e.g. `"SIGTERM"`, `"SIGKILL"`). Returns `true` on success and
/// `false` on any failure (invalid pid, unsupported signal, …).
pub(super) fn native_process_kill(
    _interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let pid = match args.first() {
        Some(Value::Integer(n)) => *n as u32,
        Some(Value::Float(n)) => *n as u32,
        _ => return Ok(Value::Boolean(false)),
    };
    let signal = args
        .get(1)
        .map(|v| to_string_value(_interp, v))
        .unwrap_or_else(|| "SIGTERM".to_string());
    Ok(Value::Boolean(tails_process::kill(pid, &signal).is_ok()))
}

/// `process.uptime()` — wall-clock seconds the current process has
/// been running. Distinct from `os.uptime()` (system uptime).
pub(super) fn native_process_uptime(
    _interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    Ok(Value::Float(tails_process::process_uptime_secs()))
}

/// `process.memoryUsage()` — returns an object with
/// `{rss, heapTotal, heapUsed, external, arrayBuffers}`. The shape
/// matches Node's `process.memoryUsage()` so user code can be ported
/// verbatim.
pub(super) fn native_process_memory_usage(
    interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    let mu = tails_process::memory_usage();
    let mut props = rustc_hash::FxHashMap::default();
    props.insert("rss".into(), Value::Integer(mu.rss as i64));
    props.insert("heapTotal".into(), Value::Integer(mu.heap_total as i64));
    props.insert("heapUsed".into(), Value::Integer(mu.heap_used as i64));
    props.insert("external".into(), Value::Integer(mu.external as i64));
    props.insert(
        "arrayBuffers".into(),
        Value::Integer(mu.array_buffers as i64),
    );
    let obj_idx = interp.heap.len();
    interp.heap.push(crate::vm::interpreter::HeapValue::Object(
        crate::vm::interpreter::JsObject {
            properties: props,
            prototype: None,
            extensible: true,
        },
    ));
    Ok(Value::Object(obj_idx))
}

/// `process.on(event, listener)` — currently only the `"exit"`
/// event is supported. Other event names are accepted and the
/// listener is silently dropped (matching Node, where non-exit
/// events on `process` are simply not implemented).
///
/// Multiple listeners can be registered for `"exit"`; they are run
/// in LIFO order at process exit.
pub(super) fn native_process_on(
    interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    let event = args
        .first()
        .map(|v| to_string_value(interp, v))
        .unwrap_or_default();
    if event != "exit" {
        // Silently ignore non-exit events.
        return Ok(Value::Undefined);
    }
    let listener = args.get(1).cloned().unwrap_or(Value::Undefined);
    let mut g = match exit_handlers().lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };
    // LIFO order: push to the front so the most recent handler runs
    // first (matching Node's behaviour for `process.on('exit', ...)`).
    g.insert(0, listener);
    Ok(Value::Undefined)
}
