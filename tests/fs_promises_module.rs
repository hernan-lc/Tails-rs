#![cfg(feature = "fs-promises")]

//! Tests for the `fs/promises` cdylib — the Promise-based file
//! system API for Tails-rs.
//!
//! Each `#[tails_function]` in `modules/fs-promises/src/lib.rs`
//! returns a JSON string with the shape `{ok: true, value: <x>}`
//! for success and `{ok: false, error: "..."}` for failure. The
//! tests parse that envelope and assert on the `value` / `error`
//! fields so we exercise the full FFI path.

use std::path::Path;
use tails::TailsRuntime;

fn cdylib_present() -> bool {
    let dist = std::env::current_dir()
        .ok()
        .map(|d| d.join("dist"))
        .unwrap_or_default();
    dist.join("libfs-promises.so").exists()
        || dist.join("libfs-promises.dylib").exists()
        || dist.join("fs-promises.dll").exists()
}

fn run(script: &str) -> tails::Value {
    let mut rt = TailsRuntime::default();
    rt.eval_module(script, Path::new("/tmp/test_fs_promises_module.ts"))
        .expect("script failed to evaluate")
}

#[test]
fn test_fs_promises_write_and_read_roundtrip() {
    if !cdylib_present() {
        eprintln!("skipping: no fs-promises cdylib in dist/");
        return;
    }
    let val = run(r#"
        import fs from "fs/promises";
        const w = JSON.parse(await fs.write_file("/tmp/test_fs_promises_round.txt", "abc"));
        const r = JSON.parse(await fs.read_file("/tmp/test_fs_promises_round.txt"));
        w.ok === true && r.ok === true && r.value === "abc";
    "#);
    assert_eq!(val, tails::Value::Boolean(true));
    let _ = std::fs::remove_file("/tmp/test_fs_promises_round.txt");
}

#[test]
fn test_fs_promises_read_file_missing() {
    if !cdylib_present() {
        eprintln!("skipping: no fs-promises cdylib in dist/");
        return;
    }
    let val = run(r#"
        import fs from "fs/promises";
        const res = JSON.parse(await fs.read_file("/tmp/__nope_does_not_exist__12345.txt"));
        res.ok === false && typeof res.error === "string" && res.error.indexOf("ENOENT") !== -1;
    "#);
    assert_eq!(val, tails::Value::Boolean(true));
}

#[test]
fn test_fs_promises_mkdir_and_readdir() {
    if !cdylib_present() {
        eprintln!("skipping: no fs-promises cdylib in dist/");
        return;
    }
    let val = run(r#"
        import fs from "fs/promises";
        const dir = "/tmp/test_fs_promises_mkdir";
        const m = JSON.parse(await fs.mkdir(dir, true));
        await fs.write_file(dir + "/a.txt", "a");
        await fs.write_file(dir + "/b.txt", "b");
        const r = JSON.parse(await fs.readdir(dir));
        await fs.unlink(dir + "/a.txt");
        await fs.unlink(dir + "/b.txt");
        await fs.unlink(dir);
        m.ok === true && r.ok === true && Array.isArray(r.value) && r.value.length === 2;
    "#);
    assert_eq!(val, tails::Value::Boolean(true));
    let _ = std::fs::remove_dir_all("/tmp/test_fs_promises_mkdir");
}

#[test]
fn test_fs_promises_stat_is_file() {
    if !cdylib_present() {
        eprintln!("skipping: no fs-promises cdylib in dist/");
        return;
    }
    let val = run(r#"
        import fs from "fs/promises";
        const f = "/tmp/test_fs_promises_stat.txt";
        await fs.write_file(f, "hello");
        const s = JSON.parse(await fs.stat(f));
        await fs.unlink(f);
        s.ok === true && s.value.isFile === true && s.value.size === 5;
    "#);
    assert_eq!(val, tails::Value::Boolean(true));
}

#[test]
fn test_fs_promises_unlink() {
    if !cdylib_present() {
        eprintln!("skipping: no fs-promises cdylib in dist/");
        return;
    }
    let val = run(r#"
        import fs from "fs/promises";
        const f = "/tmp/test_fs_promises_unlink.txt";
        await fs.write_file(f, "bye");
        const u = JSON.parse(await fs.unlink(f));
        u.ok === true;
    "#);
    assert_eq!(val, tails::Value::Boolean(true));
    let _ = std::fs::remove_file("/tmp/test_fs_promises_unlink.txt");
}

#[test]
fn test_fs_promises_copy_and_rename() {
    if !cdylib_present() {
        eprintln!("skipping: no fs-promises cdylib in dist/");
        return;
    }
    let val = run(r#"
        import fs from "fs/promises";
        const src = "/tmp/test_fs_promises_src.txt";
        const dst = "/tmp/test_fs_promises_dst.txt";
        await fs.write_file(src, "copy me");
        const c = JSON.parse(await fs.copy_file(src, dst));
        const r1 = JSON.parse(await fs.read_file(dst));
        const mv = JSON.parse(await fs.rename(src, "/tmp/test_fs_promises_src2.txt"));
        const r2 = JSON.parse(await fs.read_file("/tmp/test_fs_promises_src2.txt"));
        await fs.unlink("/tmp/test_fs_promises_src2.txt");
        await fs.unlink(dst);
        c.ok === true && r1.value === "copy me" &&
        mv.ok === true && r2.value === "copy me";
    "#);
    assert_eq!(val, tails::Value::Boolean(true));
    let _ = std::fs::remove_file("/tmp/test_fs_promises_src.txt");
    let _ = std::fs::remove_file("/tmp/test_fs_promises_src2.txt");
    let _ = std::fs::remove_file("/tmp/test_fs_promises_dst.txt");
}

#[test]
fn test_fs_promises_append_file() {
    if !cdylib_present() {
        eprintln!("skipping: no fs-promises cdylib in dist/");
        return;
    }
    let val = run(r#"
        import fs from "fs/promises";
        const f = "/tmp/test_fs_promises_append.txt";
        await fs.write_file(f, "Hello");
        await fs.append_file(f, ", World!");
        const r = JSON.parse(await fs.read_file(f));
        await fs.unlink(f);
        r.ok === true && r.value === "Hello, World!";
    "#);
    assert_eq!(val, tails::Value::Boolean(true));
    let _ = std::fs::remove_file("/tmp/test_fs_promises_append.txt");
}

#[test]
fn test_fs_promises_exists() {
    if !cdylib_present() {
        eprintln!("skipping: no fs-promises cdylib in dist/");
        return;
    }
    let val = run(r#"
        import fs from "fs/promises";
        const f = "/tmp/test_fs_promises_exists.txt";
        await fs.write_file(f, "x");
        const a = JSON.parse(await fs.exists(f));
        await fs.unlink(f);
        const b = JSON.parse(await fs.exists(f));
        a.ok === true && a.value === true && b.ok === true && b.value === false;
    "#);
    assert_eq!(val, tails::Value::Boolean(true));
    let _ = std::fs::remove_file("/tmp/test_fs_promises_exists.txt");
}

#[test]
fn test_fs_promises_error_envelope_shape() {
    if !cdylib_present() {
        eprintln!("skipping: no fs-promises cdylib in dist/");
        return;
    }
    // Verify that the error envelope has both an `ok: false` flag
    // and a non-empty `error` string.
    let val = run(r#"
        import fs from "fs/promises";
        const r = JSON.parse(await fs.read_file("/tmp/__definitely_missing_xyzzy__.txt"));
        r.ok === false && r.value === undefined && typeof r.error === "string" && r.error.length > 0;
    "#);
    assert_eq!(val, tails::Value::Boolean(true));
}
