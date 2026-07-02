#![cfg(feature = "fs")]

//! Tests for the `fs` native module exposed by the runtime. With the
//! v0.3.0 cdylib work, the same surface is reachable two ways:
//!   1. As a cdylib via `import fs from "./fs.native"` — exercised here.
//!   2. Through the runtime's built-in registration when no cdylib is
//!      present. (The legacy `fs.writeFileSync` / `fs.readFileSync` etc.
//!      API lived there and is exercised by the previous test suite
//!      in the git history.)

use std::path::Path;
use tails::TailsRuntime;

/// Skip these tests when no `fs` cdylib is present. The legacy built-in
/// registration uses a different API (`fs.writeFileSync` etc.) and is
/// covered by the older `fs` integration tests in the git history.
fn cdylib_present() -> bool {
    let dist = std::env::current_dir()
        .ok()
        .map(|d| d.join("dist"))
        .unwrap_or_default();
    dist.join("libfs.so").exists()
        || dist.join("libfs.dylib").exists()
        || dist.join("fs.dll").exists()
}

fn run(script: &str) -> tails::Value {
    let mut rt = TailsRuntime::default();
    rt.eval_module(script, Path::new("/tmp/test_module.ts"))
        .expect("script failed to evaluate")
}

#[test]
fn test_fs_write_and_read() {
    if !cdylib_present() {
        eprintln!("skipping: no fs cdylib in dist/");
        return;
    }
    let val = run(r#"
        import fs from "./fs.native";
        fs.write_file("/tmp/test_fs_write.txt", "Hello");
        fs.read_file("/tmp/test_fs_write.txt");
        "#);
    assert_eq!(val, tails::Value::String("Hello".to_string()));
    std::fs::remove_file("/tmp/test_fs_write.txt").ok();
}

#[test]
fn test_fs_exists_sync() {
    if !cdylib_present() {
        eprintln!("skipping: no fs cdylib in dist/");
        return;
    }
    let val = run(r#"
        import fs from "./fs.native";
        fs.exists("/tmp/nonexistent_file_12345.txt");
        "#);
    assert_eq!(val, tails::Value::Boolean(false));
}

#[test]
fn test_fs_mkdir_and_readdir() {
    if !cdylib_present() {
        eprintln!("skipping: no fs cdylib in dist/");
        return;
    }
    let val = run(r#"
        import fs from "./fs.native";
        fs.mkdir("/tmp/test_fs_mkdir", true);
        fs.write_file("/tmp/test_fs_mkdir/a.txt", "a");
        fs.write_file("/tmp/test_fs_mkdir/b.txt", "b");
        let files = JSON.parse(fs.readdir("/tmp/test_fs_mkdir"));
        fs.rm("/tmp/test_fs_mkdir", true);
        files.length;
        "#);
    assert_eq!(val, tails::Value::Float(2.0));
}

#[test]
fn test_fs_stat_sync() {
    if !cdylib_present() {
        eprintln!("skipping: no fs cdylib in dist/");
        return;
    }
    let val = run(r#"
        import fs from "./fs.native";
        fs.write_file("/tmp/test_fs_stat.txt", "test content");
        let stat = JSON.parse(fs.stat("/tmp/test_fs_stat.txt"));
        fs.unlink("/tmp/test_fs_stat.txt");
        stat.size;
        "#);
    assert_eq!(val, tails::Value::Integer(12));
}

#[test]
fn test_fs_stat_is_file() {
    if !cdylib_present() {
        eprintln!("skipping: no fs cdylib in dist/");
        return;
    }
    let val = run(r#"
        import fs from "./fs.native";
        fs.write_file("/tmp/test_fs_stat2.txt", "test");
        let stat = JSON.parse(fs.stat("/tmp/test_fs_stat2.txt"));
        fs.unlink("/tmp/test_fs_stat2.txt");
        stat.isFile;
        "#);
    assert_eq!(val, tails::Value::Boolean(true));
}

#[test]
fn test_fs_append_file() {
    if !cdylib_present() {
        eprintln!("skipping: no fs cdylib in dist/");
        return;
    }
    let val = run(r#"
        import fs from "./fs.native";
        fs.write_file("/tmp/test_fs_append.txt", "Hello");
        fs.append_file("/tmp/test_fs_append.txt", " World");
        let result = fs.read_file("/tmp/test_fs_append.txt");
        fs.unlink("/tmp/test_fs_append.txt");
        result;
        "#);
    assert_eq!(val, tails::Value::String("Hello World".to_string()));
}

#[test]
fn test_fs_copy_file() {
    if !cdylib_present() {
        eprintln!("skipping: no fs cdylib in dist/");
        return;
    }
    let val = run(r#"
        import fs from "./fs.native";
        fs.write_file("/tmp/test_fs_copy_src.txt", "copy me");
        fs.copy_file("/tmp/test_fs_copy_src.txt", "/tmp/test_fs_copy_dst.txt");
        let result = fs.read_file("/tmp/test_fs_copy_dst.txt");
        fs.unlink("/tmp/test_fs_copy_src.txt");
        fs.unlink("/tmp/test_fs_copy_dst.txt");
        result;
        "#);
    assert_eq!(val, tails::Value::String("copy me".to_string()));
}

#[test]
fn test_fs_rename_file() {
    if !cdylib_present() {
        eprintln!("skipping: no fs cdylib in dist/");
        return;
    }
    let val = run(r#"
        import fs from "./fs.native";
        fs.write_file("/tmp/test_fs_rename_old.txt", "rename me");
        fs.rename("/tmp/test_fs_rename_old.txt", "/tmp/test_fs_rename_new.txt");
        let result = fs.read_file("/tmp/test_fs_rename_new.txt");
        fs.unlink("/tmp/test_fs_rename_new.txt");
        result;
        "#);
    assert_eq!(val, tails::Value::String("rename me".to_string()));
}

#[test]
fn test_fs_unlink_file() {
    if !cdylib_present() {
        eprintln!("skipping: no fs cdylib in dist/");
        return;
    }
    let val = run(r#"
        import fs from "./fs.native";
        fs.write_file("/tmp/test_fs_unlink.txt", "delete me");
        fs.unlink("/tmp/test_fs_unlink.txt");
        fs.exists("/tmp/test_fs_unlink.txt");
        "#);
    assert_eq!(val, tails::Value::Boolean(false));
}

#[test]
fn test_fs_rm_recursive() {
    if !cdylib_present() {
        eprintln!("skipping: no fs cdylib in dist/");
        return;
    }
    let val = run(r#"
        import fs from "./fs.native";
        fs.mkdir("/tmp/test_fs_rm_dir", false);
        fs.write_file("/tmp/test_fs_rm_dir/file.txt", "data");
        fs.rm("/tmp/test_fs_rm_dir", true);
        fs.exists("/tmp/test_fs_rm_dir");
        "#);
    assert_eq!(val, tails::Value::Boolean(false));
}

// ---------------------------------------------------------------------------
// v0.5.0 additions: createReadStream + watch
// ---------------------------------------------------------------------------

#[test]
fn test_fs_create_read_stream_reads_full_file() {
    if !cdylib_present() {
        eprintln!("skipping: no fs cdylib in dist/");
        return;
    }
    // Open a 12-byte file as a stream and drain it in two chunks of
    // 5 bytes each. The first chunk returns `done: false`, the
    // second returns `done: true` after EOF, and a final dummy call
    // also returns `done: true` so we know the table cleanup is
    // idempotent.
    let val = run(r#"
        import fs from "./fs.native";
        fs.write_file("/tmp/test_fs_crs.txt", "Hello, Tails!");
        const open = JSON.parse(fs.create_read_stream("/tmp/test_fs_crs.txt"));
        const id = open.id;
        const c1 = JSON.parse(fs.stream_read(id, 5));
        const c2 = JSON.parse(fs.stream_read(id, 5));
        const c3 = JSON.parse(fs.stream_read(id, 5));
        const dec1 = Buffer.from(c1.data, "base64").toString("utf8");
        const dec2 = Buffer.from(c2.data, "base64").toString("utf8");
        const closed = fs.stream_close(id);
        fs.unlink("/tmp/test_fs_crs.txt");
        open.ok === true &&
        c1.ok === true && c1.done === false && c2.ok === true && c2.done === false &&
        c3.ok === true && c3.done === true &&
        dec1 === "Hello" && dec2 === ", Tai" && closed === true;
        "#);
    assert_eq!(val, tails::Value::Boolean(true));
}

#[test]
fn test_fs_create_read_stream_missing_file_errors() {
    if !cdylib_present() {
        eprintln!("skipping: no fs cdylib in dist/");
        return;
    }
    let val = run(r#"
        import fs from "./fs.native";
        const r = JSON.parse(fs.create_read_stream("/tmp/__definitely_missing__.bin"));
        r.ok === false && typeof r.error === "string" && r.error.indexOf("ENOENT") !== -1;
        "#);
    assert_eq!(val, tails::Value::Boolean(true));
}

#[test]
fn test_fs_stream_close_invalid_id_is_false() {
    if !cdylib_present() {
        eprintln!("skipping: no fs cdylib in dist/");
        return;
    }
    // Closing a never-allocated id should return false, not panic.
    let val = run(r#"
        import fs from "./fs.native";
        fs.stream_close(999999) === false;
        "#);
    assert_eq!(val, tails::Value::Boolean(true));
}

#[test]
fn test_fs_watch_detects_create() {
    if !cdylib_present() {
        eprintln!("skipping: no fs cdylib in dist/");
        return;
    }
    // Watch an empty directory, write a new file into it, and then
    // poll the watcher. The polling interval is clamped to >= 10ms
    // so we sleep for ~150ms before polling to guarantee the
    // background snapshot diff has run.
    let val = run(r#"
        import fs from "./fs.native";
        const dir = "/tmp/test_fs_watch_create";
        fs.mkdir(dir, true);
        const w = JSON.parse(fs.watch(dir, 50));
        if (!w.ok) { fs.rm(dir, true); false; }
        fs.write_file(dir + "/new.txt", "hello");
        // Poll the watcher in a loop, breaking early as soon as we
        // see the create event. A single-threaded runtime never
        // yields to a background thread between awaits, but the
        // watcher's `poll()` re-snapshots on every call when the
        // interval has elapsed — so the very first poll after the
        // interval will see the new file.
        let events = [];
        for (let i = 0; i < 30 && events.length === 0; i++) {
            const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
            await sleep(20);
            events = JSON.parse(fs.watch_poll(w.id));
        }
        const closed = fs.watch_close(w.id);
        fs.rm(dir, true);
        w.ok === true && closed === true && events.length > 0 &&
        events[0].type === "create" &&
        events[0].path.indexOf("/tmp/test_fs_watch_create/new.txt") !== -1;
        "#);
    assert_eq!(val, tails::Value::Boolean(true));
}

#[test]
fn test_fs_watch_missing_path_errors() {
    if !cdylib_present() {
        eprintln!("skipping: no fs cdylib in dist/");
        return;
    }
    let val = run(r#"
        import fs from "./fs.native";
        const r = JSON.parse(fs.watch("/tmp/__definitely_missing_watch__.dir", 100));
        r.ok === false && typeof r.error === "string" && r.error.length > 0;
        "#);
    assert_eq!(val, tails::Value::Boolean(true));
}
