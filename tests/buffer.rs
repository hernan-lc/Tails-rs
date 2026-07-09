use std::path::Path;
use tails::TailsRuntime;

#[test]
fn test_buffer_alloc() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import Buffer from "./buffer.native";
        let b = Buffer.alloc(5, 0);
        b.length;
    "#,
        Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Integer(5));
}

#[test]
fn test_buffer_from_string() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import Buffer from "./buffer.native";
        let b = Buffer.from("Hello");
        b.toString();
    "#,
        Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::string("Hello"));
}

#[test]
fn test_buffer_from_array() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import Buffer from "./buffer.native";
        let b = Buffer.from([72, 101, 108, 108, 111]);
        b.toString();
    "#,
        Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::string("Hello"));
}

#[test]
fn test_buffer_concat() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import Buffer from "./buffer.native";
        let b1 = Buffer.from("Hello");
        let b2 = Buffer.from(" World");
        let b3 = Buffer.concat([b1, b2]);
        b3.toString();
    "#,
        Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::string("Hello World"));
}

#[test]
fn test_buffer_is_buffer() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import Buffer from "./buffer.native";
        let b = Buffer.from("test");
        Buffer.isBuffer(b);
    "#,
        Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Boolean(true));
}

#[test]
fn test_buffer_is_buffer_false() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import Buffer from "./buffer.native";
        Buffer.isBuffer("not a buffer");
    "#,
        Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Boolean(false));
}

#[test]
fn test_buffer_byte_length() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import Buffer from "./buffer.native";
        Buffer.byteLength("Hello");
    "#,
        Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Integer(5));
}

#[test]
fn test_buffer_slice() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import Buffer from "./buffer.native";
        let b = Buffer.from("Hello, World!");
        let s = b.slice(0, 5);
        s.toString();
    "#,
        Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::string("Hello"));
}

#[test]
fn test_buffer_index_of() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import Buffer from "./buffer.native";
        let b = Buffer.from("Hello, World!");
        b.indexOf("World");
    "#,
        Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Integer(7));
}

#[test]
fn test_buffer_equals() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import Buffer from "./buffer.native";
        let b1 = Buffer.from("Hello");
        let b2 = Buffer.from("Hello");
        b1.equals(b2);
    "#,
        Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Boolean(true));
}

#[test]
fn test_buffer_not_equals() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import Buffer from "./buffer.native";
        let b1 = Buffer.from("Hello");
        let b2 = Buffer.from("World");
        b1.equals(b2);
    "#,
        Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Boolean(false));
}

#[test]
fn test_buffer_compare() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import Buffer from "./buffer.native";
        let b1 = Buffer.from("ABC");
        let b2 = Buffer.from("ABD");
        b1.compare(b2);
    "#,
        Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Integer(-1));
}

#[test]
fn test_buffer_alloc_fill() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import Buffer from "./buffer.native";
        let b = Buffer.alloc(3, 65);
        b.toString();
    "#,
        Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::string("AAA"));
}

#[test]
fn test_buffer_alloc_zero_fill() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import Buffer from "./buffer.native";
        let b = Buffer.alloc(3);
        b.toString();
    "#,
        Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::string("\0\0\0"));
}
