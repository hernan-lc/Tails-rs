use tails::{TailsRuntime, Value};

#[test]
fn test_declarations() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let x = 10;
    const y = 20;
    var z = 30;
    x + y + z;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::Float(60.0));
}

#[test]
fn test_primitives() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let s = "hello";
    let b = true;
    let n = null;
    let u = undefined;
    typeof s + "," + typeof b + "," + typeof n + "," + typeof u;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(
        r.unwrap(),
        Value::string("string,boolean,object,undefined")
    );
}

#[test]
fn test_arithmetic_operators() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    10 + 3 + "," + (10 - 3) + "," + (10 * 3) + "," + (10 / 3) + "," + (10 % 3) + "," + (2 ** 10);
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(
        r.unwrap(),
        Value::string("13,7,30,3.3333333333333335,1,1024")
    );
}

#[test]
fn test_compound_assignment() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let x = 10;
    x += 5;
    x;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::Float(15.0));
}

#[test]
fn test_comparison_operators() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    (5 == 5) + "," + (5 === 5) + "," + (5 != 3) + "," + (5 !== "5") + "," + (5 < 10) + "," + (5 > 3);
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(
        r.unwrap(),
        Value::string("true,true,true,true,true,true")
    );
}

#[test]
fn test_logical_operators() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    (true && false) + "," + (true || false) + "," + (!true);
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::string("false,true,false"));
}

#[test]
fn test_typeof_and_void() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    typeof 42 + "," + typeof "hi" + "," + typeof true + "," + typeof undefined + "," + typeof (void 0);
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(
        r.unwrap(),
        Value::string("number,string,boolean,undefined,undefined")
    );
}

#[test]
fn test_increment_decrement() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let counter = 0;
    counter++;
    counter++;
    counter--;
    counter;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::Float(1.0));
}
