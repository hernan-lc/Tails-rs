use tails::TailsRuntime;

#[test]
fn test_basic_try_catch() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"
        let result = 0;
        try {
            result = 1;
            throw "error";
            result = 2;
        } catch(e) {
            result = 3;
        }
        result;
    "#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(3.0));
}

#[test]
fn test_throw_new_error() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"
        let msg = "";
        try {
            throw new Error("something went wrong");
        } catch(e) {
            msg = e.message;
        }
        msg;
    "#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::String("something went wrong".to_string()));
}

#[test]
fn test_catch_binding() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"
        let caught = "";
        try {
            throw "my error value";
        } catch(e) {
            caught = e;
        }
        caught;
    "#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::String("my error value".to_string()));
}

#[test]
fn test_finally_always_runs() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"
        let order = [];
        try {
            order.push(1);
        } finally {
            order.push(2);
        }
        order.length;
    "#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(2.0));
}

#[test]
fn test_try_catch_finally() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"
        let result = 0;
        try {
            throw "err";
        } catch(e) {
            result = 1;
        } finally {
            result = 2;
        }
        result;
    "#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(2.0));
}

#[test]
fn test_nested_try_catch() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"
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
    "#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::String("inner: inner error, outer: outer error".to_string()));
}

#[test]
fn test_error_prototype_message() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"
        let e = new Error("test message");
        e.message;
    "#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::String("test message".to_string()));
}

#[test]
fn test_error_prototype_stack() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"
        let e = new Error("test");
        typeof e.stack;
    "#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::String("string".to_string()));
}

#[test]
fn test_finally_runs_on_exception() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"
        let ran = false;
        try {
            throw "err";
        } catch(e) {
            // caught
        } finally {
            ran = true;
        }
        ran;
    "#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Boolean(true));
}

#[test]
fn test_no_catch_propagates() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"
        try {
            try {
                throw "uncaught";
            } finally {
                // finally runs
            }
        } catch(e) {
            e;
        }
    "#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::String("uncaught".to_string()));
}

#[test]
fn test_type_error_constructor() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"
        let e = new TypeError("bad type");
        e.message;
    "#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::String("bad type".to_string()));
}

#[test]
fn test_reference_error_constructor() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"
        let e = new ReferenceError("not defined");
        e.message;
    "#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::String("not defined".to_string()));
}

#[test]
fn test_syntax_error_constructor() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"
        let e = new SyntaxError("unexpected token");
        e.message;
    "#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::String("unexpected token".to_string()));
}

#[test]
fn test_range_error_constructor() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"
        let e = new RangeError("out of range");
        e.message;
    "#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::String("out of range".to_string()));
}

#[test]
fn test_throw_number() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"
        let result = 0;
        try {
            throw 42;
        } catch(e) {
            result = e;
        }
        result;
    "#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(42.0));
}
