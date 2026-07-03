//! `fs/promises` — Promise-based file system API for Tails-rs.
//!
//! Mirrors the Node.js `fs/promises` module surface. The functions
//! here return a *resolved* string, JSON string, or empty result on
//! success and a *rejected* string on error — the same envelope that
//! the runtime's built-in `native_fs_read_file` / `native_fs_write_file`
//! / etc. promises produce. That keeps `import fs from "fs/promises"`
//! and the legacy `import fs from "./fs.native"` consistent.
//!
//! The cdylib is intentionally synchronous: each function is a thin
//! wrapper around the underlying `tails-fs` call wrapped in a JSON
//! success/error envelope, exactly as `native_fs_*` in
//! `src/runtime_env/native_fns/fs_fns.rs` does. That way `await`
//! always works without the user having to coordinate threading.
//!
//! All public functions are FFI-safe (no `async`, no threads) so the
//! module can link into the host binary without a tokio runtime — see
//! `modules/websocket/src/lib.rs` for the async alternative pattern.

use tails_native_macros::tails_module;

// ============================================================================
// Native module (cdylib FFI exports)
// ============================================================================

#[tails_module(name = "tails-fs-promises")]
mod fs_promises_native {
    use serde_json::Value as JsonValue;
    use tails_native_macros::tails_function;

    /// Build a `{ok: true, value: <serialized>}` JSON string.
    /// The `__tails_` prefix prevents the module macro from
    /// exporting it as a JS function.
    fn __tails_ok(value: JsonValue) -> String {
        serde_json::json!({ "ok": true, "value": value }).to_string()
    }

    /// Build a `{ok: false, error: "..."}` JSON string.
    /// The `__tails_` prefix prevents the module macro from
    /// exporting it as a JS function.
    fn __tails_err(message: impl Into<String>) -> String {
        serde_json::json!({ "ok": false, "error": message.into() }).to_string()
    }

    #[tails_function]
    pub fn read_file(path: String) -> String {
        match tails_fs::read_file(&path) {
            Ok(content) => __tails_ok(JsonValue::String(content)),
            Err(e) => __tails_err(format!(
                "ENOENT: no such file or directory, open '{}': {}",
                path, e
            )),
        }
    }

    #[tails_function]
    pub fn write_file(path: String, content: String) -> String {
        match tails_fs::write_file(&path, &content) {
            Ok(()) => __tails_ok(JsonValue::Null),
            Err(e) => __tails_err(format!(
                "EACCES: permission denied, open '{}': {}",
                path, e
            )),
        }
    }

    #[tails_function]
    pub fn readdir(path: String) -> String {
        match tails_fs::readdir(&path) {
            Ok(names) => {
                let arr: Vec<JsonValue> = names.into_iter().map(JsonValue::String).collect();
                __tails_ok(JsonValue::Array(arr))
            }
            Err(e) => __tails_err(format!(
                "ENOENT: no such file or directory, scandir '{}': {}",
                path, e
            )),
        }
    }

    #[tails_function]
    pub fn stat(path: String) -> String {
        match tails_fs::stat(&path) {
            Ok(s) => __tails_ok(serde_json::json!({
                "size": s.size,
                "isFile": s.is_file,
                "isDirectory": s.is_directory,
                "isSymbolicLink": s.is_symbolic_link,
                "mode": s.mode,
                "mtimeMs": s.mtime_ms,
                "birthtimeMs": s.birthtime_ms,
            })),
            Err(e) => __tails_err(format!(
                "ENOENT: no such file or directory, stat '{}': {}",
                path, e
            )),
        }
    }

    #[tails_function]
    pub fn mkdir(path: String, recursive: bool) -> String {
        match tails_fs::mkdir(&path, recursive) {
            Ok(()) => __tails_ok(JsonValue::Null),
            Err(e) => __tails_err(format!(
                "EACCES: permission denied, mkdir '{}': {}",
                path, e
            )),
        }
    }

    #[tails_function]
    pub fn unlink(path: String) -> String {
        match tails_fs::unlink(&path) {
            Ok(()) => __tails_ok(JsonValue::Null),
            Err(e) => __tails_err(format!(
                "ENOENT: no such file or directory, unlink '{}': {}",
                path, e
            )),
        }
    }

    #[tails_function]
    pub fn copy_file(src: String, dest: String) -> String {
        match tails_fs::copy_file(&src, &dest) {
            Ok(bytes) => __tails_ok(serde_json::json!({ "bytesCopied": bytes })),
            Err(e) => __tails_err(format!(
                "EACCES: permission denied, copy '{}' to '{}': {}",
                src, dest, e
            )),
        }
    }

    #[tails_function]
    pub fn rename(old_path: String, new_path: String) -> String {
        match tails_fs::rename(&old_path, &new_path) {
            Ok(()) => __tails_ok(JsonValue::Null),
            Err(e) => __tails_err(format!(
                "EACCES: permission denied, rename '{}' to '{}': {}",
                old_path, new_path, e
            )),
        }
    }

    /// `appendFile(path, data)` — promise-style append.
    #[tails_function]
    pub fn append_file(path: String, content: String) -> String {
        match tails_fs::append_file(&path, &content) {
            Ok(()) => __tails_ok(JsonValue::Null),
            Err(e) => __tails_err(format!(
                "EACCES: permission denied, open '{}': {}",
                path, e
            )),
        }
    }

    /// `exists(path)` — promise-style existence check.
    #[tails_function]
    pub fn exists(path: String) -> String {
        __tails_ok(JsonValue::Bool(tails_fs::exists(&path)))
    }
}