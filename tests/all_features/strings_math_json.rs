use tails::{TailsRuntime, Value};

#[test]
fn test_string_methods() {
    let child = std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            let mut rt = TailsRuntime::default();
            let r = rt.eval(
                r#"
    let str = "Hello, World!";
    str.charAt(0) + "," + str.charCodeAt(0) + "," + str.slice(0, 5) + "," + str.substring(7, 12) + "," +
    str.indexOf("World") + "," + str.includes("World") + "," + str.replace("World", "Tails") + "," +
    str.split(", ").join("-") + "," + "  hi  ".trim() + "," + str.toLowerCase() + "," +
    str.toUpperCase() + "," + str.startsWith("Hello") + "," + str.endsWith("World!") + "," +
    "5".padStart(3, "0") + "," + "ab".repeat(3);
    "#,
            );
            assert!(r.is_ok(), "eval failed");
            assert_eq!(
                r.unwrap().flatten(),
                "H,72,Hello,World,7,true,Hello, Tails!,Hello-World!,hi,hello, world!,HELLO, WORLD!,true,true,005,ababab"
            );
        })
        .unwrap();
    child.join().unwrap();
}

#[test]
fn test_math() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    Math.abs(-5) + "," + Math.floor(3.7) + "," + Math.ceil(3.2) + "," + Math.round(3.5) + "," +
    Math.min(1, 2, 3) + "," + Math.max(1, 2, 3) + "," + Math.pow(2, 10) + "," + Math.sqrt(16) + "," +
    Math.sin(0) + "," + (Math.random() >= 0);
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::string("5,3,4,4,1,3,1024,4,0,true"));
}

#[test]
fn test_math_constants() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    typeof Math.PI + "," + typeof Math.E;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::string("number,number"));
}

#[test]
fn test_json() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let json = JSON.stringify({ a: 1, b: [2, 3] });
    let parsed = JSON.parse(json);
    parsed.a + "," + parsed.b.length;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::string("1,2"));
}
