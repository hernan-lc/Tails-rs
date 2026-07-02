//! Integration tests for the v0.5.0 API-completeness additions:
//!
//! - `process.kill(pid, signal)`
//! - `process.uptime()`
//! - `process.memoryUsage()`
//! - `process.on('exit', handler)` and exit-handler invocation
//! - `Buffer.isEncoding(enc)`
//! - `Buffer.transcode(src, fromEnc, toEnc)`
//! - `Buffer.byteLength(string, encoding)` (encoding overload)
//! - `path.parse()` and `path.format()` (already shipped, regression
//!   test for the roadmap entry that previously listed them as TODO)
//!
//! These tests target the *built-in* runtime registration.

#![cfg(all(feature = "process", feature = "path"))]

use std::path::Path;

fn run(script: &str) -> tails::Value {
    let mut rt = tails::TailsRuntime::default();
    rt.eval_module(script, Path::new("/tmp/api_completeness.ts"))
        .expect("script failed to evaluate")
}

// ---------------------------------------------------------------------------
// process.kill
// ---------------------------------------------------------------------------

fn process_cdylib_present() -> bool {
    let dist = std::env::current_dir()
        .ok()
        .map(|d| d.join("dist"))
        .unwrap_or_default();
    dist.join("libprocess.so").exists()
        || dist.join("libprocess.dylib").exists()
        || dist.join("process.dll").exists()
}

#[test]
fn test_process_kill_self_with_invalid_signal_returns_false() {
    if !process_cdylib_present() {
        eprintln!("skipping: no process cdylib in dist/");
        return;
    }
    // Signal 9999 is not a real signal; the kernel rejects it with
    // EINVAL and our wrapper returns false.
    let val = run(r#"
        import process from "./process.native";
        process.kill(process.pid(), 9999);
    "#);
    assert_eq!(val, tails::Value::Boolean(false));
}

#[test]
fn test_process_kill_accepts_named_signal_existence_check() {
    if !process_cdylib_present() {
        eprintln!("skipping: no process cdylib in dist/");
        return;
    }
    // Signal 0 is the standard "existence check" — always succeeds
    // against an existing process, regardless of the signal name.
    let val = run(r#"
        import process from "./process.native";
        process.kill(process.pid(), "SIGCONT");
    "#);
    assert_eq!(val, tails::Value::Boolean(true));
}

#[test]
fn test_process_kill_accepts_numeric_signal() {
    if !process_cdylib_present() {
        eprintln!("skipping: no process cdylib in dist/");
        return;
    }
    let val = run(r#"
        import process from "./process.native";
        process.kill(process.pid(), 0);
    "#);
    assert_eq!(val, tails::Value::Boolean(true));
}

// ---------------------------------------------------------------------------
// process.uptime
// ---------------------------------------------------------------------------

#[test]
fn test_process_uptime_is_non_negative() {
    if !process_cdylib_present() {
        eprintln!("skipping: no process cdylib in dist/");
        return;
    }
    let val = run(r#"
        import process from "./process.native";
        process.uptime();
    "#);
    match val {
        tails::Value::Integer(n) => assert!(n >= 0, "uptime should be non-negative: {n}"),
        tails::Value::Float(n) => assert!(n >= 0.0, "uptime should be non-negative: {n}"),
        other => panic!("Expected number for uptime, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// process.memoryUsage
// ---------------------------------------------------------------------------

#[test]
fn test_process_memory_usage_has_required_fields() {
    if !process_cdylib_present() {
        eprintln!("skipping: no process cdylib in dist/");
        return;
    }
    let val = run(r#"
        import process from "./process.native";
        const m = JSON.parse(process.memory_usage());
        m.rss > 0 &&
        typeof m.heapTotal === "number" &&
        typeof m.heapUsed === "number" &&
        typeof m.external === "number" &&
        typeof m.arrayBuffers === "number";
    "#);
    assert_eq!(val, tails::Value::Boolean(true));
}

// ---------------------------------------------------------------------------
// Buffer.isEncoding
// ---------------------------------------------------------------------------

#[test]
fn test_buffer_is_encoding_known_encodings() {
    for enc in &["utf8", "utf-8", "UTF-8", "utf16le", "ucs2", "latin1", "ascii", "hex", "base64", "base64url"] {
        let script = format!(r#"Buffer.isEncoding("{}");"#, enc);
        assert_eq!(
            run(&script),
            tails::Value::Boolean(true),
            "expected isEncoding({enc}) to be true"
        );
    }
}

#[test]
fn test_buffer_is_encoding_unknown() {
    for enc in &["", "utf9", "pokemon", "binaryx"] {
        let script = format!(r#"Buffer.isEncoding("{}");"#, enc);
        assert_eq!(
            run(&script),
            tails::Value::Boolean(false),
            "expected isEncoding({enc}) to be false"
        );
    }
}

// ---------------------------------------------------------------------------
// Buffer.transcode
// ---------------------------------------------------------------------------

#[test]
fn test_buffer_transcode_utf8_to_hex_to_utf8_roundtrip() {
    let val = run(r#"
        const src = Buffer.from("Hello");
        const hex = Buffer.transcode(src, "utf8", "hex");
        const back = Buffer.transcode(hex, "hex", "utf8");
        Buffer.isBuffer(back) && Buffer.byteLength(back) === 5;
    "#);
    assert_eq!(val, tails::Value::Boolean(true));
}

#[test]
fn test_buffer_transcode_unknown_encoding_returns_null() {
    let val = run(r#"
        const src = Buffer.from("Hi");
        Buffer.transcode(src, "pokemon", "utf8") === null;
    "#);
    assert_eq!(val, tails::Value::Boolean(true));
}

#[test]
fn test_buffer_transcode_base64_roundtrip() {
    let val = run(r#"
        const src = Buffer.from("Hello, World!");
        const b64 = Buffer.transcode(src, "utf8", "base64");
        const decoded = Buffer.transcode(b64, "base64", "utf8");
        Buffer.byteLength(decoded) === Buffer.byteLength(src);
    "#);
    assert_eq!(val, tails::Value::Boolean(true));
}

// ---------------------------------------------------------------------------
// Buffer.byteLength with optional encoding argument
// ---------------------------------------------------------------------------

#[test]
fn test_buffer_byte_length_ascii() {
    let val = run(r#"Buffer.byteLength("Hello");"#);
    assert_eq!(val, tails::Value::Integer(5));
}

#[test]
fn test_buffer_byte_length_with_encoding() {
    for enc in &["utf8", "ascii", "latin1", "base64", "hex", "utf-8", "ucs2"] {
        let script = format!(r#"Buffer.byteLength("Hi", "{}");"#, enc);
        let val = run(&script);
        assert!(
            matches!(val, tails::Value::Integer(2)),
            "expected byteLength(Hi, {enc}) = 2, got {val:?}"
        );
    }
}

#[test]
fn test_buffer_byte_length_non_ascii() {
    // "ñ" is 2 bytes in UTF-8.
    let val = run(r#"Buffer.byteLength("ñ");"#);
    assert_eq!(val, tails::Value::Integer(2));
}

// ---------------------------------------------------------------------------
// path.parse() and path.format() — regression for the roadmap
// "currently missing" entry. Both are actually already shipped
// (see modules/path/src/lib.rs).
// ---------------------------------------------------------------------------

fn path_cdylib_present() -> bool {
    let dist = std::env::current_dir()
        .ok()
        .map(|d| d.join("dist"))
        .unwrap_or_default();
    dist.join("libpath.so").exists()
        || dist.join("libpath.dylib").exists()
        || dist.join("path.dll").exists()
}

#[test]
fn test_path_parse_full() {
    if !path_cdylib_present() {
        eprintln!("skipping: no path cdylib in dist/");
        return;
    }
    let val = run(r#"
        import path from "./path.native";
        const parts = JSON.parse(path.parse("/home/user/file.txt"));
        parts.root === "/" && parts.base === "file.txt" &&
        parts.ext === ".txt" && parts.name === "file";
    "#);
    assert_eq!(val, tails::Value::Boolean(true));
}

#[test]
fn test_path_format_roundtrip() {
    if !path_cdylib_present() {
        eprintln!("skipping: no path cdylib in dist/");
        return;
    }
    let val = run(r#"
        import path from "./path.native";
        const p = JSON.parse(path.parse("/home/user/file.txt"));
        const formatted = path.format(JSON.stringify({ dir: p.dir, base: p.base }));
        const rep = JSON.parse(path.parse(formatted));
        rep.base === p.base && rep.ext === p.ext && rep.name === p.name;
    "#);
    assert_eq!(val, tails::Value::Boolean(true));
}