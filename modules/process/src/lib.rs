use std::io::Write;

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
    let mut stdout = std::io::stdout();
    stdout.write_all(data.as_bytes())?;
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
}
