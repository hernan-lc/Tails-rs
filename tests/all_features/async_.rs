use tails::{TailsRuntime, Value};

#[test]
fn test_promise_resolve() {
    let mut rt = TailsRuntime::default();
    rt.eval(
        r#"
    var result = 0;
    var p = Promise.resolve(42);
    p.then(function(val) { result = val; });
    "#,
    )
    .unwrap();
    assert_eq!(rt.get_global("result").unwrap(), Value::Float(42.0));
}

#[test]
fn test_promise_all() {
    let mut rt = TailsRuntime::default();
    rt.eval(
        r#"
    var result = 0;
    var p1 = Promise.resolve(1);
    var p2 = Promise.resolve(2);
    var all = Promise.all([p1, p2]);
    all.then(function(val) { result = val.length; });
    "#,
    )
    .unwrap();
    assert_eq!(rt.get_global("result").unwrap(), Value::Float(2.0));
}

#[test]
fn test_await() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    var p = Promise.resolve(42);
    await p;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::Float(42.0));
}

#[test]
fn test_set_timeout() {
    let mut rt = TailsRuntime::default();
    rt.eval(
        r#"
    var result = 0;
    setTimeout(function() { result = 42; }, 0);
    "#,
    )
    .unwrap();
    assert_eq!(rt.get_global("result").unwrap(), Value::Float(42.0));
}

#[test]
fn test_bigint() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let big = 42n;
    let big2 = BigInt(10);
    typeof big + "," + (big + big2);
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::string("bigint,52"));
}

#[test]
fn test_bigint_comparison() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    10n > 5n;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::Boolean(true));
}

#[test]
fn test_date() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let now = new Date();
    typeof now.getTime() + "," + typeof Date.now() + "," + typeof now.getFullYear();
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(
        r.unwrap(),
        Value::string("number,number,number")
    );
}

#[test]
fn test_regexp() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let re = new RegExp("\\d+", "g");
    re.test("abc123");
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::Boolean(true));
}

#[test]
fn test_promise_all_settled() {
    let mut rt = TailsRuntime::default();
    rt.eval(
        r#"
    var result = [];
    var p1 = Promise.resolve(1);
    var p2 = Promise.reject("fail");
    Promise.allSettled([p1, p2]).then(function(vals) {
        result = vals.length;
    });
    "#,
    )
    .unwrap();
    assert_eq!(rt.get_global("result").unwrap(), Value::Float(2.0));
}

#[test]
fn test_promise_any() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    var result = null;
    Promise.any([Promise.resolve(1), Promise.resolve(2)]).then(function(val) {
        result = val;
    });
    result;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::Float(1.0));
}

#[test]
fn test_promise_with_resolvers() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    var result = 0;
    var { resolve, reject, promise } = Promise.withResolvers();
    resolve(42);
    promise.then(function(val) { result = val; });
    result;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::Float(42.0));
}

#[test]
fn test_url() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let url = new URL("https://example.com/path?foo=bar&baz=qux");
    url.protocol + "," + url.searchParams.get("foo");
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::string("https:,bar"));
}
