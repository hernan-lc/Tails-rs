use tails::{TailsRuntime, Value};

#[test]
fn test_if_else() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let a = 5;
    if (a > 10) { "big"; } else if (a > 3) { "medium"; } else { "small"; }
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::string("medium"));
}

#[test]
fn test_ternary() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let a = 5;
    let ternary = a > 3 ? "big" : "small";
    ternary;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::string("big"));
}

#[test]
fn test_for_loop() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let sum = 0;
    for (let i = 1; i <= 5; i++) { sum = sum + i; }
    sum;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::Float(15.0));
}

#[test]
fn test_while_loop() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let w = 5;
    while (w > 0) { w = w - 1; }
    w;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::Float(0.0));
}

#[test]
fn test_do_while_loop() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let d = 0;
    do { d = d + 1; } while (d < 3);
    d;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::Float(3.0));
}

#[test]
fn test_for_in() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let obj = { a: 1, b: 2 };
    let keys = "";
    for (let k in obj) { keys = keys + k; }
    keys;
    "#,
    );
    assert!(r.is_ok());
    let val = r.unwrap();
    assert!(val == Value::string("ab") || val == Value::string("ba"));
}

#[test]
fn test_switch() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let day = 2;
    let dayName = "";
    switch (day) {
        case 1: dayName = "Mon"; break;
        case 2: dayName = "Tue"; break;
        default: dayName = "Other";
    }
    dayName;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::string("Tue"));
}

#[test]
fn test_break_continue() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let sum = 0;
    for (let i = 0; i < 10; i++) {
        if (i === 3) continue;
        if (i === 7) break;
        sum = sum + i;
    }
    sum;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::Float(18.0));
}
