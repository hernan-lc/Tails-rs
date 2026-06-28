use std::path::Path;
use tails::TailsRuntime;

#[test]
fn test_path_join() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import path from "./path.native";
        path.join("/foo", "bar", "baz");
    "#,
        Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::String("/foo/bar/baz".to_string()));
}

#[test]
fn test_path_join_single() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import path from "./path.native";
        path.join("foo");
    "#,
        Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::String("foo".to_string()));
}

#[test]
fn test_path_basename() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import path from "./path.native";
        path.basename("/foo/bar/baz.txt");
    "#,
        Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::String("baz.txt".to_string()));
}

#[test]
fn test_path_basename_with_ext() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import path from "./path.native";
        path.basename("/foo/bar/baz.txt", ".txt");
    "#,
        Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::String("baz".to_string()));
}

#[test]
fn test_path_dirname() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import path from "./path.native";
        path.dirname("/foo/bar/baz.txt");
    "#,
        Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::String("/foo/bar".to_string()));
}

#[test]
fn test_path_extname() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import path from "./path.native";
        path.extname("/foo/bar/baz.txt");
    "#,
        Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::String(".txt".to_string()));
}

#[test]
fn test_path_extname_no_ext() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import path from "./path.native";
        path.extname("/foo/bar/baz");
    "#,
        Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::String("".to_string()));
}

#[test]
fn test_path_is_absolute() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import path from "./path.native";
        path.isAbsolute("/foo");
    "#,
        Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Boolean(true));
}

#[test]
fn test_path_is_not_absolute() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import path from "./path.native";
        path.isAbsolute("foo");
    "#,
        Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Boolean(false));
}

#[test]
fn test_path_normalize() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import path from "./path.native";
        path.normalize("/foo/../bar");
    "#,
        Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::String("/bar".to_string()));
}

#[test]
fn test_path_normalize_dots() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import path from "./path.native";
        path.normalize("/foo/./bar/../baz");
    "#,
        Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::String("/foo/baz".to_string()));
}

#[test]
fn test_path_sep() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import path from "./path.native";
        path.sep;
    "#,
        Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    let val = r.unwrap();
    if let tails::Value::String(s) = val {
        assert!(s == "/" || s == "\\");
    } else {
        panic!("Expected string for sep");
    }
}

#[test]
fn test_path_delimiter() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
        import path from "./path.native";
        path.delimiter;
    "#,
        Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    let val = r.unwrap();
    if let tails::Value::String(s) = val {
        assert!(s == ":" || s == ";");
    } else {
        panic!("Expected string for delimiter");
    }
}
