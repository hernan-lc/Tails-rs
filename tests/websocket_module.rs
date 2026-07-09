use std::path::Path;
use tails::TailsRuntime;

#[test]
fn test_websocket_constructor_exists() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        typeof WebSocket === "function";
    "#,
        Path::new("/tmp/test_websocket_module.ts"),
    );
    assert!(
        r.is_ok(),
        "WebSocket constructor check failed: {:?}",
        r.err()
    );
    assert_eq!(r.unwrap(), tails::Value::Boolean(true));
}

#[test]
fn test_websocket_instance_properties() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        const ws = new WebSocket("wss://example.com/ws");
        ws.url;
    "#,
        Path::new("/tmp/test_websocket_module.ts"),
    );
    assert!(r.is_ok(), "WebSocket instance failed: {:?}", r.err());
    assert_eq!(
        r.unwrap(),
        tails::Value::string("wss://example.com/ws")
    );
}

#[test]
fn test_websocket_ready_state() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        const ws = new WebSocket("wss://example.com/ws");
        ws.readyState;
    "#,
        Path::new("/tmp/test_websocket_module.ts"),
    );
    assert!(r.is_ok(), "WebSocket readyState failed: {:?}", r.err());
    match r.unwrap() {
        tails::Value::Integer(n) => assert_eq!(n, 0, "Initial readyState should be CONNECTING (0)"),
        tails::Value::Float(n) => {
            assert_eq!(n as i64, 0, "Initial readyState should be CONNECTING (0)")
        }
        other => panic!("Expected number for readyState, got {:?}", other),
    }
}

#[test]
fn test_websocket_methods_exist() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        const ws = new WebSocket("wss://example.com/ws");
        typeof ws.send === "function" &&
        typeof ws.close === "function" &&
        typeof ws.addEventListener === "function" &&
        typeof ws.removeEventListener === "function";
    "#,
        Path::new("/tmp/test_websocket_module.ts"),
    );
    assert!(r.is_ok(), "WebSocket methods check failed: {:?}", r.err());
    assert_eq!(r.unwrap(), tails::Value::Boolean(true));
}

#[test]
fn test_websocket_event_listeners() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        const ws = new WebSocket("wss://example.com/ws");
        let called = false;
        const handler = () => { called = true; };
        ws.addEventListener("open", handler);
        ws.removeEventListener("open", handler);
        // After removing, emit should not trigger
        called;
    "#,
        Path::new("/tmp/test_websocket_module.ts"),
    );
    assert!(
        r.is_ok(),
        "WebSocket event listener test failed: {:?}",
        r.err()
    );
    assert_eq!(
        r.unwrap(),
        tails::Value::Boolean(false),
        "Handler should not have been called"
    );
}

#[test]
fn test_websocket_buffered_amount() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        const ws = new WebSocket("wss://example.com/ws");
        ws.bufferedAmount;
    "#,
        Path::new("/tmp/test_websocket_module.ts"),
    );
    assert!(r.is_ok(), "WebSocket bufferedAmount failed: {:?}", r.err());
    match r.unwrap() {
        tails::Value::Integer(n) => assert_eq!(n, 0, "Initial bufferedAmount should be 0"),
        tails::Value::Float(n) => assert_eq!(n as i64, 0, "Initial bufferedAmount should be 0"),
        other => panic!("Expected number for bufferedAmount, got {:?}", other),
    }
}
