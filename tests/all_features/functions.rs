use tails::{TailsRuntime, Value};

#[test]
fn test_function_declaration() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    function add(a, b) { return a + b; }
    add(3, 4);
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::Float(7.0));
}

#[test]
fn test_arrow_function() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let mul = (a, b) => a * b;
    mul(3, 4);
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::Float(12.0));
}

#[test]
fn test_closure() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    function makeCounter() {
        let count = 0;
        return function() { count = count + 1; return count; };
    }
    let counterFn = makeCounter();
    counterFn() + "," + counterFn() + "," + counterFn();
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::String("1,2,3".to_string()));
}

#[test]
fn test_higher_order_function() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    function add(a, b) { return a + b; }
    let applyFn = (fn, x, y) => fn(x, y);
    applyFn(add, 10, 20);
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::Float(30.0));
}

#[test]
fn test_function_call() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    function greet(greeting, name) {
        return greeting + ", " + name + "!";
    }
    greet.call(null, "Hi", "World");
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::String("Hi, World!".to_string()));
}

#[test]
fn test_function_apply() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    function add(a, b) { return a + b; }
    add.apply(null, [3, 4]);
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::Float(7.0));
}

#[test]
fn test_function_bind() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    function multiply(a, b) { return a * b; }
    let double = multiply.bind(null, 2);
    double(5);
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::Float(10.0));
}
