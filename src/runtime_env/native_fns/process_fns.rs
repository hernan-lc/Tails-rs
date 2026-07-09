use std::cell::RefCell;
use std::io::Write;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use crate::errors::Result;
use crate::objects::Value;
use crate::props;
use crate::vm::interpreter::Interpreter;

use super::helpers::to_string_value;

/// Set by `process.exit` / SIGINT / SIGTERM so the event loop can stop
/// cooperatively when an immediate hard-exit is not used.
static EXIT_REQUESTED: AtomicBool = AtomicBool::new(false);
static EXIT_CODE: AtomicI32 = AtomicI32::new(0);

/// Returns `true` once `process.exit` or a termination signal has been seen.
#[inline]
pub fn exit_requested() -> bool {
    EXIT_REQUESTED.load(Ordering::SeqCst)
}

/// Exit status requested by `process.exit` / signal handlers.
#[inline]
pub fn take_exit_code() -> i32 {
    EXIT_CODE.load(Ordering::SeqCst)
}

/// Mark that the process should exit (used by signal handlers and tests).
pub fn request_exit(code: i32) {
    EXIT_CODE.store(code, Ordering::SeqCst);
    EXIT_REQUESTED.store(true, Ordering::SeqCst);
}

/// Install SIGINT/SIGTERM handlers that request a clean exit so the event
/// loop can unwind instead of dying with an uncaught signal (fish/bash then
/// report "terminated by signal SIGTERM").
pub fn install_signal_handlers() {
    // SAFETY: only registers simple signal handlers that touch atomics.
    // No heap allocation or locks in the handler path.
    unsafe {
        libc::signal(
            libc::SIGINT,
            signal_handler as *const () as libc::sighandler_t,
        );
        libc::signal(
            libc::SIGTERM,
            signal_handler as *const () as libc::sighandler_t,
        );
    }
}

extern "C" fn signal_handler(sig: libc::c_int) {
    // Node uses 128 + signal for terminal signals; request cooperative exit.
    let code = 128i32.saturating_add(sig);
    request_exit(code);
}

// ---------------------------------------------------------------------------
// `process.on('exit', ...)` handler registry
// ---------------------------------------------------------------------------
//
// Unlike an `EventEmitter`, exit handlers are *append-only* and the
// only event ever emitted is `"exit"`. The runtime is single-threaded,
// so a `thread_local!` `RefCell<Vec<Value>>` lets a script call
// `process.on('exit', fn)` from any module without needing a per-interpreter
// registry.
//
// Handlers are invoked in LIFO order (matching Node's behaviour: most
// recently registered runs first).

thread_local! {
    static EXIT_HANDLERS: RefCell<Vec<Value>> = const { RefCell::new(Vec::new()) };
}

fn with_exit_handlers<T>(f: impl FnOnce(&mut Vec<Value>) -> T) -> T {
    EXIT_HANDLERS.with(|h| f(&mut h.borrow_mut()))
}

/// Drains and runs all registered exit handlers. Called by
/// [`native_process_exit`] and also exposed for the runtime's graceful
/// shutdown path so handlers run on a normal `tails` shutdown as well.
pub fn run_exit_handlers(interp: &mut Interpreter) {
    // Move the handlers out so that any handler that itself registers
    // more handlers (or throws) does not reentrantly re-run the same
    // callbacks.
    let handlers = with_exit_handlers(std::mem::take);
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
        Some(Value::Boolean(b)) => {
            if *b {
                1
            } else {
                0
            }
        }
        Some(Value::String(s)) => s.parse::<i32>().unwrap_or(0),
        Some(Value::Cons(c)) => c.flatten().parse::<i32>().unwrap_or(0),
        _ => 0,
    };
    // Run user-registered exit handlers before terminating. We do
    // not propagate their return value (Node also discards it) and
    // swallow any errors so a misbehaving handler does not prevent
    // the process from actually exiting.
    run_exit_handlers(interp);
    request_exit(code);
    // Flush so `console.log("exiting")` is visible before the process dies.
    let _ = std::io::stdout().flush();
    let _ = std::io::stderr().flush();
    // Hard-exit like Node: drop everything and terminate immediately so
    // open HTTP listeners / the event loop cannot keep the process alive.
    std::process::exit(code);
}

pub(super) fn native_process_cwd(
    _interp: &mut Interpreter,
    _this: &Value,
    _args: &[Value],
) -> Result<Value> {
    match tails_process::cwd() {
        Ok(path) => Ok(Value::from_string(path)),
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

/// `tty.isatty(fd)` — returns whether the given file descriptor is a TTY.
/// Accepts a number (fd) or falls back to checking stdout/stderr by convention.
pub(super) fn native_tty_isatty(
    _interp: &mut Interpreter,
    _this: &Value,
    args: &[Value],
) -> Result<Value> {
    use std::io::IsTerminal;
    let fd = match args.first() {
        Some(Value::Integer(n)) => *n,
        Some(Value::Float(n)) => *n as i64,
        _ => return Ok(Value::Boolean(false)),
    };
    let is_tty = match fd {
        0 => std::io::stdin().is_terminal(),
        1 => std::io::stdout().is_terminal(),
        2 => std::io::stderr().is_terminal(),
        #[cfg(unix)]
        n if n >= 0 => {
            // Safety: isatty is a pure query on a file descriptor.
            unsafe { libc::isatty(n as i32) == 1 }
        }
        _ => false,
    };
    Ok(Value::Boolean(is_tty))
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
    let props = props! {
        "rss" => Value::Integer(mu.rss as i64),
        "heapTotal" => Value::Integer(mu.heap_total as i64),
        "heapUsed" => Value::Integer(mu.heap_used as i64),
        "external" => Value::Integer(mu.external as i64),
        "arrayBuffers" => Value::Integer(mu.array_buffers as i64),
    };
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
    // LIFO order: insert at the front so the most recent handler runs
    // first (matching Node's behaviour for `process.on('exit', ...)`).
    with_exit_handlers(|g| g.insert(0, listener));
    Ok(Value::Undefined)
}
