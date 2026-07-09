use tails::{RuntimeConfig, TailsRuntime};

#[test]
fn test_basic_try_catch() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let result = 0;
        try {
            result = 1;
            throw "error";
            result = 2;
        } catch(e) {
            result = 3;
        }
        result;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(3.0));
}

#[test]
fn test_throw_new_error() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let msg = "";
        try {
            throw new Error("something went wrong");
        } catch(e) {
            msg = e.message;
        }
        msg;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::string("something went wrong"));
}

#[test]
fn test_catch_binding() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let caught = "";
        try {
            throw "my error value";
        } catch(e) {
            caught = e;
        }
        caught;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::string("my error value"));
}

#[test]
fn test_finally_always_runs() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let order = [];
        try {
            order.push(1);
        } finally {
            order.push(2);
        }
        order.length;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(2.0));
}

#[test]
fn test_try_catch_finally() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let result = 0;
        try {
            throw "err";
        } catch(e) {
            result = 1;
        } finally {
            result = 2;
        }
        result;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(2.0));
}

#[test]
fn test_nested_try_catch() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let result = "";
        try {
            try {
                throw "inner error";
            } catch(e) {
                result = "inner: " + e;
                throw "outer error";
            }
        } catch(e) {
            result = result + ", outer: " + e;
        }
        result;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(
        r.unwrap(),
        tails::Value::string("inner: inner error, outer: outer error")
    );
}

#[test]
fn test_error_prototype_message() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let e = new Error("test message");
        e.message;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::string("test message"));
}

#[test]
fn test_error_prototype_stack() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let e = new Error("test");
        typeof e.stack;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::string("string"));
}

#[test]
fn test_finally_runs_on_exception() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let ran = false;
        try {
            throw "err";
        } catch(e) {
            // caught
        } finally {
            ran = true;
        }
        ran;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Boolean(true));
}

#[test]
fn test_no_catch_propagates() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        try {
            try {
                throw "uncaught";
            } finally {
                // finally runs
            }
        } catch(e) {
            e;
        }
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::string("uncaught"));
}

#[test]
fn test_type_error_constructor() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let e = new TypeError("bad type");
        e.message;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::string("bad type"));
}

#[test]
fn test_reference_error_constructor() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let e = new ReferenceError("not defined");
        e.message;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::string("not defined"));
}

#[test]
fn test_syntax_error_constructor() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let e = new SyntaxError("unexpected token");
        e.message;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::string("unexpected token"));
}

#[test]
fn test_range_error_constructor() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let e = new RangeError("out of range");
        e.message;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::string("out of range"));
}

#[test]
fn test_throw_number() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let result = 0;
        try {
            throw 42;
        } catch(e) {
            result = e;
        }
        result;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(42.0));
}

#[test]
fn test_infinite_recursion_throws_error() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        function recurse() { recurse(); }
        recurse();
    "#,
    );
    assert!(r.is_err());
    let err = r.unwrap_err();
    assert!(err.message().contains("Maximum call stack size exceeded"));
}

#[test]
fn test_infinite_recursion_caught_by_try_catch() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        function recurse() { recurse(); }
        let caught = "";
        try {
            recurse();
        } catch(e) {
            caught = e.message;
        }
        caught;
    "#,
    );
    assert!(r.is_ok(), "eval should succeed when caught: {:?}", r);
    assert_eq!(
        r.unwrap(),
        tails::Value::string("Maximum call stack size exceeded")
    );
}

#[test]
fn test_infinite_recursion_caught_check_name() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        function recurse() { recurse(); }
        let name = "";
        try {
            recurse();
        } catch(e) {
            name = e.name;
        }
        name;
    "#,
    );
    assert!(r.is_ok(), "eval should succeed when caught: {:?}", r);
    assert_eq!(r.unwrap(), tails::Value::string("RangeError"));
}

#[test]
fn test_deep_recursion_passes_with_default_limit() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        function factorial(n, acc) {
            if (acc === undefined) { acc = 1; }
            if (n <= 1) { return acc; }
            return factorial(n - 1, n * acc);
        }
        factorial(5000);
    "#,
    );
    assert!(r.is_ok(), "deep recursion should pass: {:?}", r);
}

#[test]
fn test_mutual_recursion_hits_limit() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let depth = 0;
        function a() { depth++; b(); }
        function b() { depth++; a(); }
        try { a(); } catch(e) {}
        depth;
    "#,
    );
    assert!(r.is_ok());
    let val = r.unwrap();
    if let tails::Value::Float(n) = val {
        assert!(n > 100.0, "should recurse deeply before limit: got {}", n);
    } else {
        panic!("expected Float, got {:?}", val);
    }
}

#[test]
fn test_custom_shorter_recursion_limit() {
    let config = RuntimeConfig {
        max_call_stack_depth: 50,
        ..RuntimeConfig::default()
    };
    let mut rt = TailsRuntime::new(config).unwrap();
    let r = rt.eval(
        r#"
        function recurse() { recurse(); }
        let msg = "";
        try {
            recurse();
        } catch(e) {
            msg = e.message;
        }
        msg;
    "#,
    );
    assert!(r.is_ok(), "custom limit 50: {:?}", r);
    assert_eq!(
        r.unwrap(),
        tails::Value::string("Maximum call stack size exceeded")
    );
    drop(rt);

    let config = RuntimeConfig {
        max_call_stack_depth: 200,
        ..RuntimeConfig::default()
    };
    let mut rt = TailsRuntime::new(config).unwrap();
    let r = rt.eval(
        r#"
        function recurse() { recurse(); }
        let msg = "";
        try {
            recurse();
        } catch(e) {
            msg = e.message;
        }
        msg;
    "#,
    );
    assert!(r.is_ok(), "custom limit 200: {:?}", r);
    assert_eq!(
        r.unwrap(),
        tails::Value::string("Maximum call stack size exceeded")
    );
}

#[test]
fn test_custom_shorter_limit_caught_name() {
    let config = RuntimeConfig {
        max_call_stack_depth: 50,
        ..RuntimeConfig::default()
    };
    let mut rt = TailsRuntime::new(config).unwrap();
    let r = rt.eval(
        r#"
        function recurse() { recurse(); }
        let name = "";
        try {
            recurse();
        } catch(e) {
            name = e.name;
        }
        name;
    "#,
    );
    assert!(r.is_ok(), "caught name: {:?}", r);
    assert_eq!(r.unwrap(), tails::Value::string("RangeError"));
}

#[test]
fn test_custom_shorter_limit_caught_stack() {
    let config = RuntimeConfig {
        max_call_stack_depth: 10,
        ..RuntimeConfig::default()
    };
    let mut rt = TailsRuntime::new(config).unwrap();
    let r = rt.eval(
        r#"
        function recurse() { recurse(); }
        let stack = "";
        try {
            recurse();
        } catch(e) {
            stack = e.stack;
        }
        stack;
    "#,
    );
    assert!(r.is_ok(), "caught stack: {:?}", r);
    let val = r.unwrap();
    if let tails::Value::String(s) = val {
        assert!(s.contains("RangeError"), "stack should contain RangeError");
        assert!(
            s.contains("Maximum call stack size exceeded"),
            "stack should contain message"
        );
        assert!(s.contains("recurse"), "stack should contain function name");
    } else {
        panic!("expected String, got {:?}", val);
    }
}

#[test]
fn test_recursion_limit_zero_disables_check() {
    let config = RuntimeConfig {
        max_call_stack_depth: 0,
        ..RuntimeConfig::default()
    };
    let mut rt = TailsRuntime::new(config).unwrap();
    let r = rt.eval(
        r#"
        function factorial(n, acc) {
            if (acc === undefined) { acc = 1; }
            if (n <= 1) { return acc; }
            return factorial(n - 1, n * acc);
        }
        factorial(5);
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(120.0));
}
