#![cfg(feature = "process")]

//! Tests for the `process` module exposed by the runtime. With the v0.3.0
//! cdylib module work, the same surface is reachable two ways:
//!   1. As a cdylib via `import process from "./process.native"` — see
//!      `process_native_module.rs` for the FFI-bridge test suite.
//!   2. Through the runtime's built-in registration when no cdylib is
//!      available. This file exercises that fallback path. The tests
//!      skip automatically if a cdylib is detected in `dist/` to avoid
//!      false failures from the two divergent APIs.

use std::path::Path;
use tails::TailsRuntime;

/// Detect whether the `process` cdylib is present in the project's
/// `dist/` directory. When it is, this module is bypassed in favour of
/// `process_native_module.rs` so the two suites don't double-cover the
/// same code paths.
fn cdylib_present() -> bool {
    let dist = std::env::current_dir()
        .ok()
        .map(|d| d.join("dist"))
        .unwrap_or_default();
    dist.join("libprocess.so").exists()
        || dist.join("libprocess.dylib").exists()
        || dist.join("process.dll").exists()
}

fn run(script: &str) -> tails::Value {
    let mut rt = TailsRuntime::default();
    rt.eval_module(script, Path::new("/tmp/test_module.ts"))
        .expect("script failed to evaluate")
}

macro_rules! skip_if_cdylib_present {
    () => {
        if cdylib_present() {
            eprintln!(
                "skipping: cdylib present in dist/ — covered by \
                 process_native_module.rs"
            );
            return;
        }
    };
}

#[test]
fn test_process_platform() {
    skip_if_cdylib_present!();
    let val = run("process.platform;");
    if let tails::Value::String(s) = val {
        assert!(
            s == "linux" || s == "darwin" || s == "win32",
            "Unexpected platform: {}",
            s
        );
    } else {
        panic!("Expected string for platform, got {:?}", val);
    }
}

#[test]
fn test_process_arch() {
    skip_if_cdylib_present!();
    let val = run("process.arch;");
    if let tails::Value::String(s) = val {
        assert!(
            s == "x64" || s == "arm64" || s == "unknown",
            "Unexpected arch: {}",
            s
        );
    } else {
        panic!("Expected string for arch, got {:?}", val);
    }
}

#[test]
fn test_process_pid() {
    skip_if_cdylib_present!();
    let val = run("process.pid;");
    if let tails::Value::Integer(n) = val {
        assert!(n > 0, "PID should be positive");
    } else {
        panic!("Expected integer for pid, got {:?}", val);
    }
}

#[test]
fn test_process_cwd() {
    skip_if_cdylib_present!();
    let val = run("process.cwd();");
    if let tails::Value::String(s) = val {
        assert!(!s.is_empty(), "cwd should not be empty");
    } else {
        panic!("Expected string for cwd, got {:?}", val);
    }
}

#[test]
fn test_process_argv() {
    skip_if_cdylib_present!();
    let val = run("process.argv.length;");
    match val {
        tails::Value::Float(n) => assert!(n >= 1.0, "argv should have at least one element"),
        tails::Value::Integer(n) => assert!(n >= 1, "argv should have at least one element"),
        _ => panic!("Expected number for argv.length, got {:?}", val),
    }
}

#[test]
fn test_process_env() {
    skip_if_cdylib_present!();
    let val = run("typeof process.env;");
    assert_eq!(val, tails::Value::String("object".to_string()));
}

#[test]
fn test_process_stdout_write() {
    skip_if_cdylib_present!();
    let val = run("process.stdout.write(\"test\");");
    assert_eq!(val, tails::Value::Boolean(true));
}
