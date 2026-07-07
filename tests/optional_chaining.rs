mod common;
use common::TailsTestHarness;

#[test]
fn test_optional_chaining_basic() {
    let mut h = TailsTestHarness::new();
    let result = h.eval(
            r#"
        const obj = { a: { b: 42 } };
        obj?.a?.b;
    "#,
        );
    assert_eq!(result.to_string(), "42");
}

#[test]
fn test_optional_chaining_null() {
    let mut h = TailsTestHarness::new();
    let result = h.eval(
            r#"
        const obj = null;
        obj?.a;
    "#,
        );
    assert_eq!(result.to_string(), "undefined");
}

#[test]
fn test_optional_chaining_undefined() {
    let mut h = TailsTestHarness::new();
    let result = h.eval(
            r#"
        const obj = {};
        obj?.a;
    "#,
        );
    assert_eq!(result.to_string(), "undefined");
}

#[test]
fn test_optional_chaining_method() {
    let mut h = TailsTestHarness::new();
    let result = h.eval(
            r#"
        const obj = { greet: function() { return 42; } };
        obj?.greet();
    "#,
        );
    assert_eq!(result.to_string(), "42");
}

#[test]
fn test_optional_chaining_null_method() {
    let mut h = TailsTestHarness::new();
    let result = h.eval(
            r#"
        const obj = null;
        obj?.greet();
    "#,
        );
    assert_eq!(result.to_string(), "undefined");
}

#[test]
fn test_optional_chaining_computed() {
    let mut h = TailsTestHarness::new();
    let result = h.eval(
            r#"
        const obj = { a: 42 };
        obj?.['a'];
    "#,
        );
    assert_eq!(result.to_string(), "42");
}

#[test]
fn test_nullish_coalescing_basic() {
    let mut h = TailsTestHarness::new();
    let result = h.eval(
            r#"
        const x = null ?? 42;
        x;
    "#,
        );
    assert_eq!(result.to_string(), "42");
}

#[test]
fn test_nullish_coalescing_undefined() {
    let mut h = TailsTestHarness::new();
    let result = h.eval(
            r#"
        const x = undefined ?? 42;
        x;
    "#,
        );
    assert_eq!(result.to_string(), "42");
}

#[test]
fn test_nullish_coalescing_falsy() {
    let mut h = TailsTestHarness::new();
    let result = h.eval(
            r#"
        const x = 0 ?? 42;
        x;
    "#,
        );
    assert_eq!(result.to_string(), "0");
}

#[test]
fn test_nullish_coalescing_false() {
    let mut h = TailsTestHarness::new();
    let result = h.eval(
            r#"
        const x = false ?? 42;
        x;
    "#,
        );
    assert_eq!(result.to_string(), "false");
}

#[test]
fn test_nullish_coalescing_empty_string() {
    let mut h = TailsTestHarness::new();
    let result = h.eval(
            r#"
        const x = "" ?? 42;
        x;
    "#,
        );
    assert_eq!(result.to_string(), "");
}

#[test]
fn test_optional_chaining_with_nullish() {
    let mut h = TailsTestHarness::new();
    let result = h.eval(
            r#"
        const obj = { a: { b: null } };
        const result = obj?.a?.b ?? 42;
        result;
    "#,
        );
    assert_eq!(result.to_string(), "42");
}
