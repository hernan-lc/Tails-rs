use tails_native_macros::{tails_function, tails_module};

// ============================================================================
// Public API for direct Rust usage
// ============================================================================

#[derive(Debug, Clone)]
pub struct CpuInfo {
    pub model: String,
    pub speed: f64,
    pub times: CpuTimes,
}

#[derive(Debug, Clone)]
pub struct CpuTimes {
    pub user: i64,
    pub nice: i64,
    pub sys: i64,
    pub idle: i64,
    pub irq: i64,
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

pub fn cpus() -> Vec<CpuInfo> {
    let count = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);

    (0..count)
        .map(|_| CpuInfo {
            model: "CPU".to_string(),
            speed: 0.0,
            times: CpuTimes {
                user: 0,
                nice: 0,
                sys: 0,
                idle: 0,
                irq: 0,
            },
        })
        .collect()
}

pub fn totalmem() -> f64 {
    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
            for line in content.lines() {
                if let Some(rest) = line.strip_prefix("MemTotal:") {
                    if let Some(kb_str) = rest.split_whitespace().next() {
                        if let Ok(kb) = kb_str.parse::<f64>() {
                            return kb * 1024.0;
                        }
                    }
                }
            }
        }
    }
    16.0 * 1024.0 * 1024.0 * 1024.0
}

pub fn freemem() -> f64 {
    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
            for line in content.lines() {
                if let Some(rest) = line.strip_prefix("MemAvailable:") {
                    if let Some(kb_str) = rest.split_whitespace().next() {
                        if let Ok(kb) = kb_str.parse::<f64>() {
                            return kb * 1024.0;
                        }
                    }
                }
            }
        }
    }
    8.0 * 1024.0 * 1024.0 * 1024.0
}

pub fn uptime() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}

pub fn hostname() -> std::io::Result<String> {
    if let Ok(hostname) = std::env::var("HOSTNAME") {
        if !hostname.is_empty() {
            return Ok(hostname);
        }
    }
    #[cfg(unix)]
    {
        // Safe wrapper from nix — no local `unsafe`.
        if let Ok(name) = nix::unistd::gethostname() {
            let hostname = name.to_string_lossy().into_owned();
            if !hostname.is_empty() {
                return Ok(hostname);
            }
        }
    }
    Ok("localhost".to_string())
}

pub fn os_type() -> &'static str {
    if cfg!(target_os = "linux") {
        "Linux"
    } else if cfg!(target_os = "macos") {
        "Darwin"
    } else if cfg!(target_os = "windows") {
        "Windows_NT"
    } else {
        "Unknown"
    }
}

pub fn release() -> String {
    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = std::fs::read_to_string("/proc/version") {
            return content.trim().to_string();
        }
    }
    "unknown".to_string()
}

pub fn homedir() -> String {
    if let Ok(home) = std::env::var("HOME") {
        if !home.is_empty() {
            return home;
        }
    }
    #[cfg(target_os = "windows")]
    {
        if let Ok(home) = std::env::var("USERPROFILE") {
            if !home.is_empty() {
                return home;
            }
        }
    }
    "/".to_string()
}

pub fn tmpdir() -> String {
    std::env::temp_dir().to_string_lossy().to_string()
}

// ============================================================================
// Native module (cdylib FFI exports)
// ============================================================================

#[tails_module(name = "tails-os")]
mod os_native {
    use super::*;
    #[tails_function]
    pub fn platform() -> String {
        super::platform().to_string()
    }

    #[tails_function]
    pub fn arch() -> String {
        super::arch().to_string()
    }

    #[tails_function]
    pub fn cpus() -> String {
        let cpus: Vec<serde_json::Value> = super::cpus()
            .into_iter()
            .map(|cpu| {
                serde_json::json!({
                    "model": cpu.model,
                    "speed": cpu.speed,
                    "times": {
                        "user": cpu.times.user,
                        "nice": cpu.times.nice,
                        "sys": cpu.times.sys,
                        "idle": cpu.times.idle,
                        "irq": cpu.times.irq
                    }
                })
            })
            .collect();
        serde_json::to_string(&cpus).unwrap_or_else(|_| "[]".to_string())
    }

    #[tails_function]
    pub fn totalmem() -> f64 {
        super::totalmem()
    }

    #[tails_function]
    pub fn freemem() -> f64 {
        super::freemem()
    }

    #[tails_function]
    pub fn uptime() -> f64 {
        super::uptime()
    }

    #[tails_function]
    pub fn hostname() -> String {
        super::hostname().unwrap_or_else(|_| "localhost".to_string())
    }

    #[tails_function]
    pub fn os_type() -> String {
        super::os_type().to_string()
    }

    #[tails_function]
    pub fn release() -> String {
        super::release()
    }

    #[tails_function]
    pub fn homedir() -> String {
        super::homedir()
    }

    #[tails_function]
    pub fn tmpdir() -> String {
        super::tmpdir()
    }

    #[tails_function]
    pub fn type_name() -> String {
        "Tails".to_string()
    }

    #[tails_function]
    pub fn endianness() -> String {
        if cfg!(target_endian = "little") {
            "LE".to_string()
        } else {
            "BE".to_string()
        }
    }

    #[tails_function]
    pub fn loadavg() -> String {
        #[cfg(target_os = "linux")]
        {
            if let Ok(content) = std::fs::read_to_string("/proc/loadavg") {
                let parts: Vec<&str> = content.split_whitespace().collect();
                if parts.len() >= 3 {
                    return serde_json::json!([
                        parts[0].parse::<f64>().unwrap_or(0.0),
                        parts[1].parse::<f64>().unwrap_or(0.0),
                        parts[2].parse::<f64>().unwrap_or(0.0)
                    ])
                    .to_string();
                }
            }
        }
        "[0.0, 0.0, 0.0]".to_string()
    }

    #[tails_function]
    pub fn env_var(name: String) -> String {
        std::env::var(&name).unwrap_or_default()
    }

    #[tails_function]
    pub fn env_vars() -> String {
        let vars: Vec<serde_json::Value> = std::env::vars()
            .map(|(k, v)| serde_json::json!({"key": k, "value": v}))
            .collect();
        serde_json::to_string(&vars).unwrap_or_else(|_| "[]".to_string())
    }

    #[tails_function]
    pub fn pid() -> f64 {
        std::process::id() as f64
    }

    #[tails_function]
    pub fn ppidd() -> f64 {
        #[cfg(unix)]
        {
            // nix::unistd::getppid is a safe wrapper over the syscall.
            nix::unistd::getppid().as_raw() as f64
        }
        #[cfg(not(unix))]
        {
            0.0
        }
    }

    #[tails_function]
    pub fn getegid() -> f64 {
        #[cfg(unix)]
        {
            nix::unistd::getegid().as_raw() as f64
        }
        #[cfg(not(unix))]
        {
            0.0
        }
    }

    #[tails_function]
    pub fn geteuid() -> f64 {
        #[cfg(unix)]
        {
            nix::unistd::geteuid().as_raw() as f64
        }
        #[cfg(not(unix))]
        {
            0.0
        }
    }

    #[tails_function]
    pub fn getgid() -> f64 {
        #[cfg(unix)]
        {
            nix::unistd::getgid().as_raw() as f64
        }
        #[cfg(not(unix))]
        {
            0.0
        }
    }

    #[tails_function]
    pub fn getuid() -> f64 {
        #[cfg(unix)]
        {
            nix::unistd::getuid().as_raw() as f64
        }
        #[cfg(not(unix))]
        {
            0.0
        }
    }
}
