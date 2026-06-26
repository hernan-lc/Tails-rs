use tails::TailsRuntime;
use tails::Value;

#[test]
fn test_promise_resolve_static() {
    let mut runtime = TailsRuntime::default();
    runtime.eval(r#"
        var result = 0;
        var p = Promise.resolve(99);
        p.then(function(val) {
            result = val;
        });
    "#).unwrap();
    let result = runtime.get_global("result").unwrap();
    assert_eq!(result, Value::Float(99.0));
}

#[test]
fn test_promise_reject_static() {
    let mut runtime = TailsRuntime::default();
    runtime.eval(r#"
        var result = 0;
        var p = Promise.reject("err");
        p.catch(function(val) {
            result = val;
        });
    "#).unwrap();
    let result = runtime.get_global("result").unwrap();
    assert_eq!(result, Value::String("err".into()));
}

#[test]
fn test_promise_all_resolved() {
    let mut runtime = TailsRuntime::default();
    runtime.eval(r#"
        var result = 0;
        var p1 = Promise.resolve(1);
        var p2 = Promise.resolve(2);
        var p3 = Promise.resolve(3);
        var all = Promise.all([p1, p2, p3]);
        all.then(function(val) {
            result = val.length;
        });
    "#).unwrap();
    let result = runtime.get_global("result").unwrap();
    assert_eq!(result, Value::Float(3.0));
}

#[test]
fn test_promise_all_one_rejected() {
    let mut runtime = TailsRuntime::default();
    runtime.eval(r#"
        var result = 0;
        var p1 = Promise.resolve(1);
        var p2 = Promise.reject("fail");
        var p3 = Promise.resolve(3);
        var all = Promise.all([p1, p2, p3]);
        all.catch(function(val) {
            result = val;
        });
    "#).unwrap();
    let result = runtime.get_global("result").unwrap();
    assert_eq!(result, Value::String("fail".into()));
}

#[test]
fn test_promise_constructor_resolve() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(r#"
        var p = new Promise(function(resolve, reject) {
            resolve(42);
        });
        p;
    "#);
    eprintln!("new Promise result: {:?}", result);
    result.unwrap();
}

#[test]
fn test_set_timeout_schedules_callback() {
    let mut runtime = TailsRuntime::default();
    runtime.eval(r#"
        var result = 0;
        setTimeout(function() {
            result = 42;
        }, 0);
    "#).unwrap();
    let result = runtime.get_global("result").unwrap();
    assert_eq!(result, Value::Float(42.0));
}

#[test]
fn test_promise_chaining_then() {
    let mut runtime = TailsRuntime::default();
    runtime.eval(r#"
        var result = 0;
        var p = Promise.resolve(10);
        p.then(function(val) {
            result = val + 5;
        });
    "#).unwrap();
    let result = runtime.get_global("result").unwrap();
    assert_eq!(result, Value::Float(15.0));
}

#[test]
fn test_promise_finally() {
    let mut runtime = TailsRuntime::default();
    runtime.eval(r#"
        var result = 0;
        var p = Promise.resolve(1);
        p.then(function(val) {
            result = result + val;
        }).finally(function() {
            result = result + 10;
        });
    "#).unwrap();
    let result = runtime.get_global("result").unwrap();
    assert_eq!(result, Value::Float(11.0));
}

#[test]
fn test_promise_chaining_multiple_thens() {
    let mut runtime = TailsRuntime::default();
    runtime.eval(r#"
        var result = 0;
        var p = Promise.resolve(1);
        p.then(function(val) {
            result = result + val;
        }).then(function(val) {
            result = result + 10;
        });
    "#).unwrap();
    let result = runtime.get_global("result").unwrap();
    assert_eq!(result, Value::Float(11.0));
}

#[test]
fn test_await_resolved_promise() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(r#"
        var p = Promise.resolve(42);
        await p;
    "#).unwrap();
    assert_eq!(result, Value::Float(42.0));
}

#[test]
fn test_await_non_promise_value() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(r#"
        await 42;
    "#).unwrap();
    assert_eq!(result, Value::Float(42.0));
}

#[test]
fn test_promise_basic_resolve() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(r#"
        var p = Promise.resolve(42);
        p;
    "#).unwrap();
    match &result {
        Value::Promise(_) => {},
        _other => panic!("Expected Promise, got {:?}", result),
    }
}

#[test]
fn test_promise_reject_with_catch() {
    let mut runtime = TailsRuntime::default();
    runtime.eval(r#"
        var result = 0;
        var p = Promise.reject(10);
        p.catch(function(val) {
            result = val + 5;
        });
    "#).unwrap();
    let result = runtime.get_global("result").unwrap();
    assert_eq!(result, Value::Float(15.0));
}
