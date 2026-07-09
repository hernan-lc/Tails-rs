use tails_native_macros::{tails_function, tails_module};

// ============================================================================
// Public API for direct Rust usage
// ============================================================================

pub fn cwd() -> std::io::Result<String> {
    Ok(std::env::current_dir()?.to_string_lossy().to_string())
}

pub fn chdir(dir: &str) -> std::io::Result<()> {
    std::env::set_current_dir(dir)
}

pub fn stdout_write(data: &str) -> std::io::Result<usize> {
    use std::io::Write;
    let mut stdout = std::io::stdout();
    stdout.write_all(data.as_bytes())?;
    stdout.flush()?;
    Ok(data.len())
}

pub fn hrtime() -> (u64, u32) {
    let dur = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    (dur.as_secs(), dur.subsec_nanos())
}

pub fn hrtime_bigint() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}

pub fn platform() -> &'static str {
    if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "macos") {
        "darwin"
    } else if cfg!(target_os = "windows") {
        "win32"
    } else {
        "unknown"
    }
}

pub fn arch() -> &'static str {
    if cfg!(target_arch = "x86_64") {
        "x64"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else if cfg!(target_arch = "x86") {
        "x86"
    } else {
        "unknown"
    }
}

pub fn pid() -> u32 {
    std::process::id()
}

pub fn env_vars() -> Vec<(String, String)> {
    std::env::vars().collect()
}

pub fn argv() -> Vec<String> {
    std::env::args().collect()
}

/// Send a signal to a process. `signal` follows the same names as
/// Node.js (e.g. `"SIGTERM"`, `"SIGKILL"`, `"SIGINT"`, `"SIGHUP"`) but
/// is also accepted as a raw libc signal number (as a string of digits).
///
/// Returns `Ok(())` if the kill syscall was dispatched. Note that
/// signal delivery is best-effort.
///
/// Uses the safe `nix` wrappers on Unix (no local `unsafe`).
pub fn kill(pid: u32, signal: &str) -> std::io::Result<()> {
    let sig = parse_signal(signal);
    #[cfg(unix)]
    {
        use nix::sys::signal::{kill as nix_kill, Signal};
        use nix::unistd::Pid;
        use std::convert::TryFrom;

        let signal = Signal::try_from(sig).map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("invalid signal number: {}", sig),
            )
        })?;
        nix_kill(Pid::from_raw(pid as i32), signal)
            .map_err(|e| std::io::Error::from_raw_os_error(e as i32))
    }
    #[cfg(not(unix))]
    {
        let _ = (pid, sig);
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "process.kill is only supported on Unix",
        ))
    }
}

fn parse_signal(signal: &str) -> i32 {
    // Accept numeric strings first (e.g. "9", "15") — Node's
    // `process.kill` also does this.
    if let Ok(n) = signal.parse::<i32>() {
        return n;
    }
    // Common POSIX signal names. Mirrors the most common subset of
    // Node's signal table; we keep this small to avoid pulling in
    // `libc::SIG*` constants that may not be defined on every target.
    match signal.to_ascii_uppercase().as_str() {
        "SIGHUP" => 1,
        "SIGINT" => 2,
        "SIGQUIT" => 3,
        "SIGILL" => 4,
        "SIGTRAP" => 5,
        "SIGABRT" | "SIGIOT" => 6,
        "SIGBUS" => 7,
        "SIGFPE" => 8,
        "SIGKILL" => 9,
        "SIGUSR1" => 10,
        "SIGSEGV" => 11,
        "SIGUSR2" => 12,
        "SIGPIPE" => 13,
        "SIGALRM" => 14,
        "SIGTERM" => 15,
        "SIGSTKFLT" => 16,
        "SIGCHLD" | "SIGCLD" => 17,
        "SIGCONT" => 18,
        "SIGSTOP" => 19,
        "SIGTSTP" => 20,
        "SIGTTIN" => 21,
        "SIGTTOU" => 22,
        "SIGURG" => 23,
        "SIGXCPU" => 24,
        "SIGXFSZ" => 25,
        "SIGVTALRM" => 26,
        "SIGPROF" => 27,
        "SIGWINCH" => 28,
        "SIGIO" | "SIGPOLL" => 29,
        "SIGPWR" => 30,
        "SIGSYS" => 31,
        _ => 15, // Default to SIGTERM, matching Node's behaviour.
    }
}

/// Wall-clock seconds the process has been running. Distinct from
/// `os.uptime()` which is the system uptime.
pub fn process_uptime_secs() -> f64 {
    static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    let start = START.get_or_init(std::time::Instant::now);
    start.elapsed().as_secs_f64()
}

/// Process memory stats in bytes. The `rss` field is always populated
/// on Linux and macOS. `heapTotal` / `heapUsed` mirror the same-named
/// Node fields; `external` is `0` since this runtime does not use V8.
#[derive(Debug, Clone, Copy)]
pub struct MemoryUsage {
    pub rss: u64,
    pub heap_total: u64,
    pub heap_used: u64,
    pub external: u64,
    pub array_buffers: u64,
}

pub fn memory_usage() -> MemoryUsage {
    let (rss, vm_size) = read_rss_bytes();
    MemoryUsage {
        rss,
        heap_total: vm_size.unwrap_or(rss),
        heap_used: rss,
        external: 0,
        array_buffers: 0,
    }
}

#[cfg(target_os = "linux")]
fn read_rss_bytes() -> (u64, Option<u64>) {
    use std::fs;
    let content = match fs::read_to_string("/proc/self/status") {
        Ok(s) => s,
        Err(_) => return (0, None),
    };
    let mut rss_kb: Option<u64> = None;
    let mut vm_size_kb: Option<u64> = None;
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("VmRSS:") {
            rss_kb = rest
                .split_whitespace()
                .next()
                .and_then(|s| s.parse::<u64>().ok());
        } else if let Some(rest) = line.strip_prefix("VmSize:") {
            vm_size_kb = rest
                .split_whitespace()
                .next()
                .and_then(|s| s.parse::<u64>().ok());
        }
    }
    (rss_kb.unwrap_or(0) * 1024, vm_size_kb.map(|kb| kb * 1024))
}

#[cfg(target_os = "macos")]
fn read_rss_bytes() -> (u64, Option<u64>) {
    // On macOS `ps -o rss=` gives RSS in kilobytes. We shell out
    // because the Mach `task_info` API would need a per-call port.
    use std::process::Command;
    let output = Command::new("ps")
        .args(["-o", "rss=", "-p", &std::process::id().to_string()])
        .output();
    if let Ok(out) = output {
        if let Ok(s) = String::from_utf8(out.stdout) {
            if let Ok(kb) = s.trim().parse::<u64>() {
                return (kb * 1024, None);
            }
        }
    }
    (0, None)
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn read_rss_bytes() -> (u64, Option<u64>) {
    (0, None)
}

// ============================================================================
// Native module (cdylib FFI exports)
// ============================================================================

#[tails_module(name = "tails-process")]
mod process_native {
    use super::*;

    #[tails_function]
    pub fn cwd() -> String {
        super::cwd().unwrap_or_default()
    }

    #[tails_function]
    pub fn chdir(dir: String) -> bool {
        super::chdir(&dir).is_ok()
    }

    #[tails_function]
    pub fn stdout_write(data: String) -> bool {
        super::stdout_write(&data).is_ok()
    }

    #[tails_function]
    pub fn hrtime() -> String {
        let (secs, nanos) = super::hrtime();
        serde_json::json!([secs, nanos]).to_string()
    }

    #[tails_function]
    pub fn hrtime_bigint() -> String {
        super::hrtime_bigint().to_string()
    }

    #[tails_function]
    pub fn platform() -> String {
        super::platform().to_string()
    }

    #[tails_function]
    pub fn arch() -> String {
        super::arch().to_string()
    }

    #[tails_function]
    pub fn pid() -> f64 {
        super::pid() as f64
    }

    #[tails_function]
    pub fn env_vars() -> String {
        let vars: Vec<serde_json::Value> = super::env_vars()
            .into_iter()
            .map(|(k, v)| serde_json::json!({ "key": k, "value": v }))
            .collect();
        serde_json::to_string(&vars).unwrap_or_else(|_| "[]".to_string())
    }

    #[tails_function]
    pub fn argv() -> String {
        serde_json::to_string(&super::argv()).unwrap_or_else(|_| "[]".to_string())
    }

    /// `process.kill(pid, signal)` — dispatches a signal. Returns
    /// `true` on success, `false` on any error (invalid signal,
    /// non-existent pid, …). Accepts the same signal names as Node,
    /// and also accepts a raw integer signal number (e.g. `9`, `15`).
    ///
    /// The `signal` parameter is typed as `NativeValue` rather than
    /// `String`/`i64` because the JS caller may legitimately pass
    /// either `"SIGTERM"` (a string) or `15` (a number), and the
    /// `#[tails_function]` macro's `FromNativeValue` conversion would
    /// otherwise collapse the wrong case to an empty string and
    /// dispatch a default SIGTERM.
    #[tails_function]
    pub fn kill(pid: f64, signal: ::tails_abi::NativeValue) -> bool {
        // Convert the signal NativeValue into the textual form that
        // `super::kill` (and ultimately `libc::kill`) expects.
        let signal_str = match signal.tag {
            ::tails_abi::TAG_STRING => ::tails_abi::get_string(signal),
            ::tails_abi::TAG_NUMBER => {
                let n = ::tails_abi::get_number(signal);
                if n == (n as i64) as f64 {
                    // Integer-valued number — emit a numeric string so
                    // `parse_signal` takes the `parse::<i32>()` branch.
                    format!("{}", n as i64)
                } else {
                    String::new()
                }
            }
            _ => String::new(),
        };
        super::kill(pid as u32, &signal_str).is_ok()
    }

    /// `process.uptime()` — wall-clock seconds the process has been
    /// running. Distinct from `os.uptime()` which is the system
    /// uptime.
    #[tails_function]
    pub fn uptime() -> f64 {
        super::process_uptime_secs()
    }

    /// `process.memoryUsage()` — returns a JSON object with
    /// `{rss, heapTotal, heapUsed, external, arrayBuffers}`. The
    /// shape matches Node's `process.memoryUsage()` so user code can
    /// be ported verbatim.
    #[tails_function]
    pub fn memory_usage() -> String {
        let mu = super::memory_usage();
        serde_json::json!({
            "rss": mu.rss,
            "heapTotal": mu.heap_total,
            "heapUsed": mu.heap_used,
            "external": mu.external,
            "arrayBuffers": mu.array_buffers,
        })
        .to_string()
    }
}
