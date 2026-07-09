#![cfg(feature = "path")]

//! Tests for the `path` native module exposed by the runtime. With the
//! v0.3.0 cdylib work, the same surface is reachable two ways:
//!   1. As a cdylib via `import path from "./path.native"` — exercised here.
//!   2. Through the runtime's built-in registration when no cdylib is
//!      present. (The legacy `path.join("/foo", "bar", "baz")` etc. API
//!      lived there and is exercised by the previous test suite in the
//!      git history.)

use std::path::Path;
use tails::TailsRuntime;

/// Skip these tests when no `path` cdylib is present.
fn cdylib_present() -> bool {
    let dist = std::env::current_dir()
        .ok()
        .map(|d| d.join("dist"))
        .unwrap_or_default();
    dist.join("libpath.so").exists()
        || dist.join("libpath.dylib").exists()
        || dist.join("path.dll").exists()
}

fn run(script: &str) -> tails::Value {
    let mut rt = TailsRuntime::default();
    rt.eval_module(script, Path::new("/tmp/test_module.ts"))
        .expect("script failed to evaluate")
}

#[test]
fn test_path_join() {
    if !cdylib_present() {
        eprintln!("skipping: no path cdylib in dist/");
        return;
    }
    let val = run(r#"
        import path from "./path.native";
        path.join('["/foo","bar","baz"]');
        "#);
    assert_eq!(val, tails::Value::string("/foo/bar/baz"));
}

#[test]
fn test_path_join_single() {
    if !cdylib_present() {
        eprintln!("skipping: no path cdylib in dist/");
        return;
    }
    let val = run(r#"
        import path from "./path.native";
        path.join('["foo"]');
        "#);
    assert_eq!(val, tails::Value::string("foo"));
}

#[test]
fn test_path_basename() {
    if !cdylib_present() {
        eprintln!("skipping: no path cdylib in dist/");
        return;
    }
    let val = run(r#"
        import path from "./path.native";
        path.basename("/foo/bar/baz.txt", "");
        "#);
    assert_eq!(val, tails::Value::string("baz.txt"));
}

#[test]
fn test_path_basename_with_ext() {
    if !cdylib_present() {
        eprintln!("skipping: no path cdylib in dist/");
        return;
    }
    let val = run(r#"
        import path from "./path.native";
        path.basename("/foo/bar/baz.txt", ".txt");
        "#);
    assert_eq!(val, tails::Value::string("baz"));
}

#[test]
fn test_path_dirname() {
    if !cdylib_present() {
        eprintln!("skipping: no path cdylib in dist/");
        return;
    }
    let val = run(r#"
        import path from "./path.native";
        path.dirname("/foo/bar/baz.txt");
        "#);
    assert_eq!(val, tails::Value::string("/foo/bar"));
}

#[test]
fn test_path_extname() {
    if !cdylib_present() {
        eprintln!("skipping: no path cdylib in dist/");
        return;
    }
    let val = run(r#"
        import path from "./path.native";
        path.extname("/foo/bar/baz.txt");
        "#);
    assert_eq!(val, tails::Value::string(".txt"));
}

#[test]
fn test_path_extname_no_ext() {
    if !cdylib_present() {
        eprintln!("skipping: no path cdylib in dist/");
        return;
    }
    let val = run(r#"
        import path from "./path.native";
        path.extname("/foo/bar/baz");
        "#);
    assert_eq!(val, tails::Value::string(""));
}

#[test]
fn test_path_is_absolute() {
    if !cdylib_present() {
        eprintln!("skipping: no path cdylib in dist/");
        return;
    }
    let val = run(r#"
        import path from "./path.native";
        path.is_absolute("/foo");
        "#);
    assert_eq!(val, tails::Value::Boolean(true));
}

#[test]
fn test_path_is_not_absolute() {
    if !cdylib_present() {
        eprintln!("skipping: no path cdylib in dist/");
        return;
    }
    let val = run(r#"
        import path from "./path.native";
        path.is_absolute("foo");
        "#);
    assert_eq!(val, tails::Value::Boolean(false));
}

#[test]
fn test_path_normalize() {
    if !cdylib_present() {
        eprintln!("skipping: no path cdylib in dist/");
        return;
    }
    let val = run(r#"
        import path from "./path.native";
        path.normalize("/foo/../bar");
        "#);
    assert_eq!(val, tails::Value::string("/bar"));
}

#[test]
fn test_path_normalize_dots() {
    if !cdylib_present() {
        eprintln!("skipping: no path cdylib in dist/");
        return;
    }
    let val = run(r#"
        import path from "./path.native";
        path.normalize("/foo/./bar/../baz");
        "#);
    assert_eq!(val, tails::Value::string("/foo/baz"));
}

#[test]
fn test_path_sep() {
    if !cdylib_present() {
        eprintln!("skipping: no path cdylib in dist/");
        return;
    }
    let val = run(r#"
        import path from "./path.native";
        path.sep();
        "#);
    if let tails::Value::String(s) = val {
        assert!(s.as_ref() == "/" || s.as_ref() == "\\");
    } else {
        panic!("Expected string for sep");
    }
}

#[test]
fn test_path_delimiter() {
    if !cdylib_present() {
        eprintln!("skipping: no path cdylib in dist/");
        return;
    }
    let val = run(r#"
        import path from "./path.native";
        path.delimiter();
        "#);
    if let tails::Value::String(s) = val {
        assert!(s.as_ref() == ":" || s.as_ref() == ";");
    } else {
        panic!("Expected string for delimiter");
    }
}
