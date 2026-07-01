use std::path::Path;
use tails::TailsRuntime;

fn fixture_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/require_cjs")
}

fn run_fixture(name: &str) -> String {
    let path = fixture_dir().join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e))
}

fn fixture_file(name: &str) -> std::path::PathBuf {
    fixture_dir().join(name)
}

#[test]
#[allow(clippy::approx_constant)]
fn test_require_exports_object() {
    let mut runtime = TailsRuntime::default();
    let source = run_fixture("main_exports.ts");
    let base = fixture_file("main_exports.ts");
    let result = runtime.eval_module(&source, &base).unwrap();

    // result should be the math.cjs exports object
    let greeting = runtime.get_property(&result, "greeting").unwrap();
    assert_eq!(greeting, tails::Value::String("hello from CJS".to_string()));

    let pi = runtime.get_property(&result, "PI").unwrap();
    assert_eq!(pi, tails::Value::Float(3.14159));
}

#[test]
fn test_require_exports_function() {
    let mut runtime = TailsRuntime::default();
    let source = run_fixture("main_function.ts");
    let base = fixture_file("main_function.ts");
    let result = runtime.eval_module(&source, &base).unwrap();

    // result should be the greeter function
    let output = runtime
        .call_function(
            &result,
            &tails::Value::Undefined,
            &[tails::Value::String("World".to_string())],
        )
        .unwrap();
    assert_eq!(output, tails::Value::String("Hello, World!".to_string()));
}

#[test]
fn test_require_caching() {
    let mut runtime = TailsRuntime::default();
    let source = run_fixture("main_cache.ts");
    let base = fixture_file("main_cache.ts");
    let result = runtime.eval_module(&source, &base).unwrap();
    assert_eq!(result, tails::Value::Boolean(true));
}

#[test]
fn test_require_circular_deps() {
    let mut runtime = TailsRuntime::default();
    let source = run_fixture("main_circular.ts");
    let base = fixture_file("main_circular.ts");
    let result = runtime.eval_module(&source, &base).unwrap();

    let a_x = runtime.get_property(&result, "a_x").unwrap();
    assert_eq!(a_x, tails::Value::Float(10.0));

    let b_y = runtime.get_property(&result, "b_y").unwrap();
    assert_eq!(b_y, tails::Value::Float(20.0));
}

#[test]
fn test_require_native_path() {
    let mut runtime = TailsRuntime::default();
    let source = r#"
        const path = require("path");
        path.join("a", "b", "c")
    "#;
    let result = runtime.eval(source).unwrap();
    assert_eq!(result, tails::Value::String("a/b/c".to_string()));
}

#[test]
fn test_require_dirname_filename() {
    let mut runtime = TailsRuntime::default();
    let source = r#"
        __dirname + "|" + __filename
    "#;
    let result = runtime
        .eval_module(source, Path::new("/tmp/test.ts"))
        .unwrap();
    let s = match result {
        tails::Value::String(s) => s,
        other => panic!("Expected string, got {:?}", other),
    };
    assert!(s.contains("/tmp"), "Expected /tmp in result: {}", s);
    assert!(s.contains("test.ts"), "Expected test.ts in result: {}", s);
}

#[test]
fn test_require_missing_module() {
    let mut runtime = TailsRuntime::default();
    let source = r#"require("./nonexistent.cjs")"#;
    let result = runtime.eval_module(source, Path::new("/tmp/test.ts"));
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message().contains("Cannot find module"));
}

#[test]
fn test_require_invalid_argument() {
    let mut runtime = TailsRuntime::default();
    let source = "require(123)";
    let result = runtime.eval_module(source, Path::new("/tmp/test.ts"));
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message().contains("expected a string"));
}
