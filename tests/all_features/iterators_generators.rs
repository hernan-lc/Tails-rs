use tails::{TailsRuntime, Value};

#[test]
fn test_for_of() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let arr = [10, 20, 30];
    let sum = 0;
    for (let v of arr) { sum = sum + v; }
    sum;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::Float(60.0));
}

#[test]
fn test_for_of_string() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let str = "abc";
    let chars = "";
    for (let c of str) { chars = chars + c + "-"; }
    chars;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::string("a-b-c-"));
}

#[test]
fn test_generator() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    function* idGen() {
        yield 10;
        yield 20;
        yield 30;
    }
    let gen = idGen();
    gen.next().value + "," + gen.next().value;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::string("10,20"));
}

#[test]
fn test_generator_done_false() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    function* idGen() {
        yield 1;
    }
    let gen = idGen();
    gen.next().done;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::Boolean(false));
}

#[test]
fn test_generator_exhausted_returns_undefined_value() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    function* oneTwo() {
        yield 1;
        yield 2;
    }
    let gen = oneTwo();
    gen.next();
    gen.next();
    gen.next().value;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::Undefined);
}

#[test]
fn test_for_await_of_promises() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let results = [];
    let arr = [Promise.resolve(1), Promise.resolve(2), Promise.resolve(3)];
    for await (let val of arr) {
        results.push(val);
    }
    results.length;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::Float(3.0));
}

#[test]
fn test_iterator_helpers() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let arr = [1, 2, 3, 4, 5];
    arr[Symbol.iterator]().map(function(x) { return x * 2; }).toArray().join(",") + "|" +
    arr[Symbol.iterator]().filter(function(x) { return x > 2; }).toArray().join(",") + "|" +
    arr[Symbol.iterator]().take(3).toArray().join(",") + "|" +
    arr[Symbol.iterator]().drop(2).toArray().join(",");
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::string("2,4,6,8,10|3,4,5|1,2,3|3,4,5"));
}

#[test]
fn test_yield_delegation_generator() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    function* inner() { yield 1; yield 2; yield 3; }
    function* outer() { yield * inner(); yield 4; }
    let result = "";
    for (let v of outer()) { result = result + v; }
    result;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::string("1234"));
}

#[test]
fn test_yield_delegation_array() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    function* delegateArr() { yield * [10, 20, 30]; yield 40; }
    let result = "";
    for (let v of delegateArr()) { result = result + v + ","; }
    result;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::string("10,20,30,40,"));
}
