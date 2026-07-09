use std::path::Path;
use tails::TailsRuntime;

#[test]
fn test_child_process_exec_sync() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import child_process from "child_process";
        const result = child_process.execSync("echo hello");
        typeof result === "string";
    "#,
        Path::new("/tmp/test_child_process_module.ts"),
    );
    assert!(r.is_ok(), "child_process.execSync failed: {:?}", r.err());
    assert_eq!(
        r.unwrap(),
        tails::Value::Boolean(true),
        "execSync should return a string"
    );
}

#[test]
fn test_child_process_exec_sync_output() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import child_process from "child_process";
        const result = child_process.execSync("printf '%s' tails-test");
        result;
    "#,
        Path::new("/tmp/test_child_process_module.ts"),
    );
    assert!(
        r.is_ok(),
        "child_process.execSync output failed: {:?}",
        r.err()
    );
    assert_eq!(
        r.unwrap(),
        tails::Value::string("tails-test"),
        "execSync should capture stdout"
    );
}

#[test]
fn test_child_process_exec_sync_failure() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import child_process from "child_process";
        child_process.execSync("exit 1");
    "#,
        Path::new("/tmp/test_child_process_module.ts"),
    );
    assert!(r.is_err(), "execSync should return error on non-zero exit");
}

#[test]
fn test_child_process_exec() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import child_process from "child_process";
        typeof child_process.exec === "function";
    "#,
        Path::new("/tmp/test_child_process_module.ts"),
    );
    assert!(r.is_ok(), "child_process.exec check failed: {:?}", r.err());
    assert_eq!(r.unwrap(), tails::Value::Boolean(true));
}

#[test]
fn test_child_process_spawn() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import child_process from "child_process";
        typeof child_process.spawn === "function";
    "#,
        Path::new("/tmp/test_child_process_module.ts"),
    );
    assert!(r.is_ok(), "child_process.spawn check failed: {:?}", r.err());
    assert_eq!(r.unwrap(), tails::Value::Boolean(true));
}
