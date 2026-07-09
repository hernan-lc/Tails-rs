//! Regression: undeclared free variables must be catchable with try/catch
//! (Node/browser libraries like `debug` rely on this for `localStorage`).

use tails::{TailsRuntime, Value};

#[test]
fn try_catch_swallows_undefined_global() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(
            r#"
        let result = "none";
        try {
            result = localStorage;
        } catch (e) {
            result = "caught";
        }
        result;
    "#,
        )
        .expect("eval");
    assert_eq!(r, Value::string("caught"));
}

#[test]
fn try_catch_reference_error_name() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(
            r#"
        let name = "";
        try {
            notDefinedAtAll;
        } catch (e) {
            name = e.name;
        }
        name;
    "#,
        )
        .expect("eval");
    assert_eq!(r, Value::string("ReferenceError"));
}

#[test]
fn try_catch_require_missing_module() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(
            r#"
        let result = "none";
        try {
            require("this-module-definitely-does-not-exist-xyz");
        } catch (e) {
            result = "caught";
        }
        result;
    "#,
        )
        .expect("eval");
    assert_eq!(r, Value::string("caught"));
}
