#![cfg(feature = "os")]

use std::path::Path;
use tails::TailsRuntime;

#[test]
fn test_os_platform() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import os from "os";
        os.platform();
    "#,
        Path::new("/tmp/test_os_module.ts"),
    );
    assert!(r.is_ok(), "os.platform() failed: {:?}", r.err());
    if let tails::Value::String(s) = r.unwrap() {
        assert!(
            s.as_ref() == "linux" || s.as_ref() == "darwin" || s.as_ref() == "win32",
            "Unexpected platform: {}",
            s
        );
    } else {
        panic!("Expected string for platform");
    }
}

#[test]
fn test_os_arch() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import os from "os";
        os.arch();
    "#,
        Path::new("/tmp/test_os_module.ts"),
    );
    assert!(r.is_ok(), "os.arch() failed: {:?}", r.err());
    if let tails::Value::String(s) = r.unwrap() {
        assert!(
            s.as_ref() == "x64" || s.as_ref() == "arm64" || s.as_ref() == "unknown",
            "Unexpected arch: {}",
            s
        );
    } else {
        panic!("Expected string for arch");
    }
}

#[test]
fn test_os_cpus() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import os from "os";
        os.cpus().length;
    "#,
        Path::new("/tmp/test_os_module.ts"),
    );
    assert!(r.is_ok(), "os.cpus() failed: {:?}", r.err());
    match r.unwrap() {
        tails::Value::Integer(n) => assert!(n > 0, "CPU count should be > 0"),
        tails::Value::Float(n) => assert!(n > 0.0, "CPU count should be > 0"),
        other => panic!("Expected number for cpus().length, got {:?}", other),
    }
}

#[test]
fn test_os_totalmem() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import os from "os";
        os.totalmem();
    "#,
        Path::new("/tmp/test_os_module.ts"),
    );
    assert!(r.is_ok(), "os.totalmem() failed: {:?}", r.err());
    match r.unwrap() {
        tails::Value::Integer(n) => assert!(n > 0, "totalmem should be > 0"),
        tails::Value::Float(n) => assert!(n > 0.0, "totalmem should be > 0"),
        other => panic!("Expected number for totalmem, got {:?}", other),
    }
}

#[test]
fn test_os_freemem() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import os from "os";
        os.freemem();
    "#,
        Path::new("/tmp/test_os_module.ts"),
    );
    assert!(r.is_ok(), "os.freemem() failed: {:?}", r.err());
    match r.unwrap() {
        tails::Value::Integer(n) => assert!(n >= 0, "freemem should be >= 0"),
        tails::Value::Float(n) => assert!(n >= 0.0, "freemem should be >= 0"),
        other => panic!("Expected number for freemem, got {:?}", other),
    }
}

#[test]
fn test_os_uptime() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import os from "os";
        os.uptime();
    "#,
        Path::new("/tmp/test_os_module.ts"),
    );
    assert!(r.is_ok(), "os.uptime() failed: {:?}", r.err());
    match r.unwrap() {
        tails::Value::Integer(n) => assert!(n >= 0, "uptime should be >= 0"),
        tails::Value::Float(n) => assert!(n >= 0.0, "uptime should be >= 0"),
        other => panic!("Expected number for uptime, got {:?}", other),
    }
}

#[test]
fn test_os_hostname() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import os from "os";
        os.hostname();
    "#,
        Path::new("/tmp/test_os_module.ts"),
    );
    assert!(r.is_ok(), "os.hostname() failed: {:?}", r.err());
    if let tails::Value::String(s) = r.unwrap() {
        assert!(!s.is_empty(), "hostname should not be empty");
    } else {
        panic!("Expected string for hostname");
    }
}

#[test]
fn test_os_type() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import os from "os";
        os.type();
    "#,
        Path::new("/tmp/test_os_module.ts"),
    );
    assert!(r.is_ok(), "os.os_type() failed: {:?}", r.err());
    if let tails::Value::String(s) = r.unwrap() {
        assert!(!s.is_empty(), "type should not be empty");
    } else {
        panic!("Expected string for type");
    }
}

#[test]
fn test_os_release() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import os from "os";
        os.release();
    "#,
        Path::new("/tmp/test_os_module.ts"),
    );
    assert!(r.is_ok(), "os.release() failed: {:?}", r.err());
    if let tails::Value::String(s) = r.unwrap() {
        assert!(!s.is_empty(), "release should not be empty");
    } else {
        panic!("Expected string for release");
    }
}

#[test]
fn test_os_homedir() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import os from "os";
        os.homedir();
    "#,
        Path::new("/tmp/test_os_module.ts"),
    );
    assert!(r.is_ok(), "os.homedir() failed: {:?}", r.err());
    if let tails::Value::String(s) = r.unwrap() {
        assert!(!s.is_empty(), "homedir should not be empty");
        assert!(
            Path::new(s.as_ref()).is_absolute(),
            "homedir should be absolute path"
        );
    } else {
        panic!("Expected string for homedir");
    }
}

#[test]
fn test_os_tmpdir() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import os from "os";
        os.tmpdir();
    "#,
        Path::new("/tmp/test_os_module.ts"),
    );
    assert!(r.is_ok(), "os.tmpdir() failed: {:?}", r.err());
    if let tails::Value::String(s) = r.unwrap() {
        assert!(!s.is_empty(), "tmpdir should not be empty");
    } else {
        panic!("Expected string for tmpdir");
    }
}
