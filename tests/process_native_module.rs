//! End-to-end tests for the `process` native module built as a cdylib.
//!
//! These tests exercise the new `#[tails_module]` / `#[tails_function]`
//! FFI bridge and the module-scoped symbol naming. They exercise the
//! `process.platform` / `process.arch` / `process.cwd()` / etc. exports
//! that ship with the v0.3.0 module-fix work.

#![cfg(feature = "process")]

use std::path::Path;
use tails::TailsRuntime;

#[test]
fn test_process_native_platform() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import process from "./process.native";
        process.platform;
    "#,
        Path::new("/tmp/test_process_native.ts"),
    );
    assert!(r.is_ok(), "process.platform failed: {:?}", r.err());
    let val = r.unwrap();
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
fn test_process_native_arch() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import process from "./process.native";
        process.arch;
    "#,
        Path::new("/tmp/test_process_native.ts"),
    );
    assert!(r.is_ok(), "process.arch failed: {:?}", r.err());
    let val = r.unwrap();
    if let tails::Value::String(s) = val {
        assert!(
            s == "x64" || s == "arm64" || s == "x86" || s == "unknown",
            "Unexpected arch: {}",
            s
        );
    } else {
        panic!("Expected string for arch, got {:?}", val);
    }
}

#[test]
fn test_process_native_pid_is_number() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import process from "./process.native";
        process.pid;
    "#,
        Path::new("/tmp/test_process_native.ts"),
    );
    assert!(r.is_ok(), "process.pid failed: {:?}", r.err());
    match r.unwrap() {
        tails::Value::Integer(n) => assert!(n > 0, "pid should be positive: {}", n),
        tails::Value::Float(n) => assert!(n > 0.0, "pid should be positive: {}", n),
        other => panic!("Expected number for pid, got {:?}", other),
    }
}

#[test]
fn test_process_native_cwd_is_string() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import process from "./process.native";
        process.cwd();
    "#,
        Path::new("/tmp/test_process_native.ts"),
    );
    assert!(r.is_ok(), "process.cwd failed: {:?}", r.err());
    let val = r.unwrap();
    if let tails::Value::String(s) = val {
        assert!(!s.is_empty(), "cwd should not be empty");
    } else {
        panic!("Expected string for cwd, got {:?}", val);
    }
}

#[test]
fn test_process_native_env_vars_returns_json_array_string() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import process from "./process.native";
        process.env;
    "#,
        Path::new("/tmp/test_process_native.ts"),
    );
    assert!(r.is_ok(), "process.env failed: {:?}", r.err());
    let val = r.unwrap();
    match val {
        tails::Value::Object(_) => {}
        other => panic!("Expected object for env, got {:?}", other),
    }
}
