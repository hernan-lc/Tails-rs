use std::path::Path;
use tails::TailsRuntime;

#[test]
fn test_events_constructor_exists() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import events from "events";
        typeof events.EventEmitter === "function";
    "#,
        Path::new("/tmp/test_events_module.ts"),
    );
    assert!(r.is_ok(), "events import failed: {:?}", r.err());
    assert_eq!(r.unwrap(), tails::Value::Boolean(true));
}

#[test]
fn test_events_emitter_has_listeners_property() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import events from "events";
        const emitter = new events.EventEmitter();
        typeof emitter._listeners === "object";
    "#,
        Path::new("/tmp/test_events_module.ts"),
    );
    assert!(r.is_ok(), "EventEmitter instantiation failed: {:?}", r.err());
    assert_eq!(r.unwrap(), tails::Value::Boolean(true));
}

#[test]
fn test_events_emitter_creates_unique_instances() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import events from "events";
        const a = new events.EventEmitter();
        const b = new events.EventEmitter();
        a !== b;
    "#,
        Path::new("/tmp/test_events_module.ts"),
    );
    assert!(r.is_ok(), "EventEmitter instance comparison failed: {:?}", r.err());
    assert_eq!(r.unwrap(), tails::Value::Boolean(true), "Each new EventEmitter should be a unique object");
}

#[test]
fn test_events_emitter_has_on_method() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import events from "events";
        const emitter = new events.EventEmitter();
        typeof emitter.on === "function";
    "#,
        Path::new("/tmp/test_events_module.ts"),
    );
    assert!(r.is_ok(), "EventEmitter.on check failed: {:?}", r.err());
    assert_eq!(r.unwrap(), tails::Value::Boolean(true));
}

#[test]
fn test_events_emitter_has_emit_method() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import events from "events";
        const emitter = new events.EventEmitter();
        typeof emitter.emit === "function";
    "#,
        Path::new("/tmp/test_events_module.ts"),
    );
    assert!(r.is_ok(), "EventEmitter.emit check failed: {:?}", r.err());
    assert_eq!(r.unwrap(), tails::Value::Boolean(true));
}

#[test]
fn test_events_emitter_has_off_method() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import events from "events";
        const emitter = new events.EventEmitter();
        typeof emitter.off === "function";
    "#,
        Path::new("/tmp/test_events_module.ts"),
    );
    assert!(r.is_ok(), "EventEmitter.off check failed: {:?}", r.err());
    assert_eq!(r.unwrap(), tails::Value::Boolean(true));
}

#[test]
fn test_events_emitter_has_listener_count_method() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import events from "events";
        const emitter = new events.EventEmitter();
        typeof emitter.listenerCount === "function";
    "#,
        Path::new("/tmp/test_events_module.ts"),
    );
    assert!(r.is_ok(), "EventEmitter.listenerCount check failed: {:?}", r.err());
    assert_eq!(r.unwrap(), tails::Value::Boolean(true));
}

#[test]
fn test_events_on_and_emit() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import events from "events";
        const emitter = new events.EventEmitter();
        let received = "";
        emitter.on("data", (msg) => { received = msg; });
        emitter.emit("data", "hello");
        received;
    "#,
        Path::new("/tmp/test_events_module.ts"),
    );
    assert!(r.is_ok(), "events on/emit failed: {:?}", r.err());
    assert_eq!(r.unwrap(), tails::Value::String("hello".to_string()));
}

#[test]
fn test_events_multiple_listeners() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import events from "events";
        const emitter = new events.EventEmitter();
        let count = 0;
        emitter.on("data", () => { count++; });
        emitter.on("data", () => { count++; });
        emitter.on("data", () => { count++; });
        emitter.emit("data");
        count;
    "#,
        Path::new("/tmp/test_events_module.ts"),
    );
    assert!(r.is_ok(), "events multiple listeners failed: {:?}", r.err());
    match r.unwrap() {
        tails::Value::Integer(n) => assert_eq!(n, 3, "Should have called 3 listeners"),
        tails::Value::Float(n) => assert_eq!(n as i64, 3, "Should have called 3 listeners"),
        other => panic!("Expected number, got {:?}", other),
    }
}

#[test]
fn test_events_off() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import events from "events";
        const emitter = new events.EventEmitter();
        let count = 0;
        const handler = () => { count++; };
        emitter.on("data", handler);
        emitter.emit("data");
        emitter.off("data", handler);
        emitter.emit("data");
        count;
    "#,
        Path::new("/tmp/test_events_module.ts"),
    );
    assert!(r.is_ok(), "events off failed: {:?}", r.err());
    match r.unwrap() {
        tails::Value::Integer(n) => assert_eq!(n, 1, "Should have called handler only once"),
        tails::Value::Float(n) => assert_eq!(n as i64, 1, "Should have called handler only once"),
        other => panic!("Expected number, got {:?}", other),
    }
}

#[test]
fn test_events_listener_count() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import events from "events";
        const emitter = new events.EventEmitter();
        emitter.on("data", () => {});
        emitter.on("data", () => {});
        emitter.listenerCount("data");
    "#,
        Path::new("/tmp/test_events_module.ts"),
    );
    assert!(r.is_ok(), "events listenerCount failed: {:?}", r.err());
    match r.unwrap() {
        tails::Value::Integer(n) => assert_eq!(n, 2, "Should have 2 listeners"),
        tails::Value::Float(n) => assert_eq!(n as i64, 2, "Should have 2 listeners"),
        other => panic!("Expected number, got {:?}", other),
    }
}

#[test]
fn test_events_emit_with_multiple_args() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import events from "events";
        const emitter = new events.EventEmitter();
        let result = "";
        emitter.on("data", (a, b, c) => { result = a + b + c; });
        emitter.emit("data", "hello", "-", "world");
        result;
    "#,
        Path::new("/tmp/test_events_module.ts"),
    );
    assert!(r.is_ok(), "events multi-arg emit failed: {:?}", r.err());
    assert_eq!(r.unwrap(), tails::Value::String("hello-world".to_string()));
}

#[test]
fn test_events_emit_returns_false_with_no_listeners() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import events from "events";
        const emitter = new events.EventEmitter();
        emitter.emit("data");
    "#,
        Path::new("/tmp/test_events_module.ts"),
    );
    assert!(r.is_ok(), "events emit return value failed: {:?}", r.err());
    assert_eq!(r.unwrap(), tails::Value::Boolean(false), "Should return false when no listeners");
}

#[test]
fn test_events_emit_returns_true_with_listeners() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import events from "events";
        const emitter = new events.EventEmitter();
        const handler = () => {};
        emitter.on("data", handler);
        emitter.emit("data");
    "#,
        Path::new("/tmp/test_events_module.ts"),
    );
    assert!(r.is_ok(), "events emit return value failed: {:?}", r.err());
    assert_eq!(r.unwrap(), tails::Value::Boolean(true), "Should return true when listeners exist");
}
