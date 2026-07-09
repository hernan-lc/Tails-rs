use tails::{TailsRuntime, Value};

#[test]
fn test_try_catch() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let msg = "";
    try {
        throw new Error("test error");
    } catch (e) {
        msg = e.message;
    }
    msg;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::string("test error"));
}

#[test]
fn test_error_types_and_stack() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let name = "";
    let hasStack = false;
    try {
        throw new TypeError("bad type");
    } catch (e) {
        name = e.name;
        hasStack = typeof e.stack === "string";
    }
    name + "," + hasStack;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::string("TypeError,true"));
}

#[test]
fn test_finally() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let order = [];
    try {
        order.push("try");
        throw "err";
    } catch (e) {
        order.push("catch");
    } finally {
        order.push("finally");
    }
    order.join(",");
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::string("try,catch,finally"));
}

#[test]
fn test_parse_int_float() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    parseInt("42") + "," + parseInt("0xFF") + "," + parseFloat("3.14");
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::string("42,255,3.14"));
}

#[test]
fn test_is_nan_is_finite() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    isNaN(NaN) + "," + isNaN(42) + "," + isFinite(42) + "," + isFinite(Infinity);
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(
        r.unwrap(),
        Value::string("true,false,true,false")
    );
}

#[test]
fn test_btoa_atob() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let encoded = btoa("Hello, World!");
    atob(encoded);
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::string("Hello, World!"));
}
