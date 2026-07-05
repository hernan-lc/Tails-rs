//! End-to-end tests for the `websocket` native module built as a cdylib.
//!
//! These tests exercise the new `#[tails_module]` / `#[tails_function]`
//! FFI bridge for the WebSocket module. They cover the synchronous FFI
//! surface (`create`, `url`, `close`, `destroy`) without actually opening
//! network connections, which would require a live WebSocket server.

#![cfg(feature = "websocket")]

use std::path::Path;
use tails::TailsRuntime;

fn cdylib_present() -> bool {
    let dist = std::env::current_dir()
        .ok()
        .map(|d| d.join("dist"))
        .unwrap_or_default();
    dist.join("libwebsocket.so").exists()
        || dist.join("libwebsocket.dylib").exists()
        || dist.join("websocket.dll").exists()
}

#[test]
fn test_websocket_native_create_returns_handle() {
    if !cdylib_present() {
        eprintln!("skipping: no websocket cdylib in dist/");
        return;
    }
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import ws from "./websocket.native";
        const id = ws.create("wss://example.com/socket");
        typeof id === "number" && id > 0;
    "#,
        Path::new("/tmp/test_websocket_native.ts"),
    );
    assert!(r.is_ok(), "ws.create failed: {:?}", r.err());
    assert_eq!(r.unwrap(), tails::Value::Boolean(true));
}

#[test]
fn test_websocket_native_url_returns_initial_url() {
    if !cdylib_present() {
        eprintln!("skipping: no websocket cdylib in dist/");
        return;
    }
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import ws from "./websocket.native";
        const id = ws.create("wss://example.com/path");
        ws.url(id);
    "#,
        Path::new("/tmp/test_websocket_native.ts"),
    );
    assert!(r.is_ok(), "ws.url failed: {:?}", r.err());
    assert_eq!(
        r.unwrap(),
        tails::Value::String("wss://example.com/path".to_string())
    );
}

#[test]
fn test_websocket_native_destroy_invalidates_handle() {
    if !cdylib_present() {
        eprintln!("skipping: no websocket cdylib in dist/");
        return;
    }
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import ws from "./websocket.native";
        const id = ws.create("wss://example.com/socket");
        const before = ws.destroy(id);
        const after = ws.destroy(id);
        before === true && after === false;
    "#,
        Path::new("/tmp/test_websocket_native.ts"),
    );
    assert!(r.is_ok(), "ws.destroy failed: {:?}", r.err());
    assert_eq!(r.unwrap(), tails::Value::Boolean(true));
}

#[test]
fn test_websocket_native_connect_without_server_returns_error() {
    if !cdylib_present() {
        eprintln!("skipping: no websocket cdylib in dist/");
        return;
    }
    // We can't actually open a connection in CI, but we can verify the FFI
    // returns a JSON error payload for an unreachable host.
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import ws from "./websocket.native";
        const id = ws.create("ws://127.0.0.1:1/");
        const result = ws.connect(id);
        const parsed = JSON.parse(result);
        parsed.ok === false;
    "#,
        Path::new("/tmp/test_websocket_native.ts"),
    );
    assert!(r.is_ok(), "ws.connect failed: {:?}", r.err());
    assert_eq!(r.unwrap(), tails::Value::Boolean(true));
}
