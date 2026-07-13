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

fn eval_fixture(runtime: &mut TailsRuntime, name: &str) -> tails::Value {
    let source = run_fixture(name);
    let base = fixture_file(name);
    runtime.eval_module(&source, &base).unwrap()
}

// ── Cross-module `new` of a CJS-exported class ───────────────────────────────

#[test]
fn test_require_cross_module_new_class() {
    let mut runtime = TailsRuntime::default();
    let result = eval_fixture(&mut runtime, "main_cross_new.ts");
    assert_eq!(result, tails::Value::Float(1.0));
}

// ── CJS named exports (exports.x = ...) ─────────────────────────────────────

#[test]
#[allow(clippy::approx_constant)]
fn test_require_exports_object() {
    let mut runtime = TailsRuntime::default();
    let result = eval_fixture(&mut runtime, "main_exports.ts");

    let greeting = runtime.get_property(&result, "greeting").unwrap();
    assert_eq!(greeting, tails::Value::string("hello from CJS"));

    let pi = runtime.get_property(&result, "PI").unwrap();
    assert_eq!(pi, tails::Value::Float(3.14159));

    let add = runtime.get_property(&result, "add").unwrap();
    assert!(
        matches!(add, tails::Value::Function(_)),
        "add should be a function"
    );
}

// ── CJS default function export (module.exports = fn) ───────────────────────

#[test]
fn test_require_exports_function() {
    let mut runtime = TailsRuntime::default();
    let result = eval_fixture(&mut runtime, "main_function.ts");

    let output = runtime
        .call_function(
            &result,
            &tails::Value::Undefined,
            &[tails::Value::string("World")],
        )
        .unwrap();
    assert_eq!(output, tails::Value::string("Hello, World!"));
}

// ── CJS require caching (identity preservation) ──────────────────────────────

#[test]
fn test_require_caching() {
    let mut runtime = TailsRuntime::default();
    let result = eval_fixture(&mut runtime, "main_cache.ts");
    assert_eq!(result, tails::Value::Boolean(true));
}

// ── CJS circular dependencies ────────────────────────────────────────────────

#[test]
fn test_require_circular_deps() {
    let mut runtime = TailsRuntime::default();
    let result = eval_fixture(&mut runtime, "main_circular.ts");

    let a_x = runtime.get_property(&result, "a_x").unwrap();
    assert_eq!(a_x, tails::Value::Float(10.0));

    let b_y = runtime.get_property(&result, "b_y").unwrap();
    assert_eq!(b_y, tails::Value::Float(20.0));
}

// ── CJS chained dependencies (a → b → c) ────────────────────────────────────

#[test]
fn test_require_chain() {
    let mut runtime = TailsRuntime::default();
    let result = eval_fixture(&mut runtime, "main_chain.ts");

    let from_a = runtime.get_property(&result, "from_a").unwrap();
    assert_eq!(from_a, tails::Value::string("A"));

    let b_from_b = runtime.get_property(&result, "b_from_b").unwrap();
    assert_eq!(b_from_b, tails::Value::string("B"));

    // chain_c exports module.exports = 42
    let b_c_val = runtime.get_property(&result, "b_c_val").unwrap();
    assert_eq!(b_c_val, tails::Value::Float(42.0));
}

// ── CJS module.exports reassignment to object literal ────────────────────────

#[test]
fn test_require_obj_literal() {
    let mut runtime = TailsRuntime::default();
    let result = eval_fixture(&mut runtime, "main_obj_literal.ts");

    let color = runtime.get_property(&result, "color").unwrap();
    assert_eq!(color, tails::Value::string("red"));

    let size = runtime.get_property(&result, "size").unwrap();
    assert_eq!(size, tails::Value::Float(42.0));

    let deep = runtime.get_property(&result, "deep").unwrap();
    assert_eq!(deep, tails::Value::Boolean(true));
}

// ── Using CJS exports in ESM (calling functions, accessing properties) ───────

#[test]
#[allow(clippy::approx_constant)]
fn test_require_use_exports() {
    let mut runtime = TailsRuntime::default();
    let result = eval_fixture(&mut runtime, "main_use_exports.ts");

    let sum = runtime.get_property(&result, "sum").unwrap();
    assert_eq!(sum, tails::Value::Float(5.0));

    let product = runtime.get_property(&result, "product").unwrap();
    assert_eq!(product, tails::Value::Float(20.0));

    let pi = runtime.get_property(&result, "pi").unwrap();
    assert_eq!(pi, tails::Value::Float(3.14159));
}

// ── Native module require (path) ─────────────────────────────────────────────

#[test]
#[cfg(feature = "path")]
fn test_require_native_path() {
    let mut runtime = TailsRuntime::default();
    let source = r#"
        const path = require("path");
        path.join("a", "b", "c")
    "#;
    let result = runtime.eval(source).unwrap();
    let sep = std::path::MAIN_SEPARATOR;
    assert_eq!(result, tails::Value::string(format!("a{sep}b{sep}c")));
}

// ── __dirname and __filename ─────────────────────────────────────────────────

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
        tails::Value::String(s) => s.to_string(),
        tails::Value::Cons(c) => c.flatten(),
        other => panic!("Expected string, got {:?}", other),
    };
    assert!(s.contains("/tmp"), "Expected /tmp in result: {}", s);
    assert!(s.contains("test.ts"), "Expected test.ts in result: {}", s);
}

// ── Missing module error ─────────────────────────────────────────────────────

#[test]
fn test_require_missing_module() {
    let mut runtime = TailsRuntime::default();
    let source = r#"require("./nonexistent.cjs")"#;
    let result = runtime.eval_module(source, Path::new("/tmp/test.ts"));
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message().contains("Cannot find module"));
}

// ── Invalid argument error ───────────────────────────────────────────────────

#[test]
fn test_require_invalid_argument() {
    let mut runtime = TailsRuntime::default();
    let source = "require(123)";
    let result = runtime.eval_module(source, Path::new("/tmp/test.ts"));
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message().contains("expected a string"));
}
