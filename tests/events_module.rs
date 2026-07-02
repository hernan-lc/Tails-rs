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
