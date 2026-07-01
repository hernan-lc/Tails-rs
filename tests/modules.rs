use std::path::Path;
use tails::TailsRuntime;

fn fixture(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/modules")
        .join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e))
}

fn module_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/modules")
}

#[test]
#[allow(clippy::approx_constant)]
fn test_named_exports() {
    let mut runtime = TailsRuntime::default();
    let source = fixture("math.ts");
    let dir = module_dir();
    runtime.eval_module(&source, &dir).unwrap();

    let result = runtime.eval("add(2, 3)").unwrap();
    assert_eq!(result, tails::Value::Float(5.0));

    let result = runtime.eval("multiply(4, 5)").unwrap();
    assert_eq!(result, tails::Value::Float(20.0));

    let result = runtime.eval("PI").unwrap();
    assert_eq!(result, tails::Value::Float(3.14159));
}

#[test]
fn test_default_export_function() {
    let mut runtime = TailsRuntime::default();
    let source = fixture("greeter.ts");
    let dir = module_dir();
    runtime.eval_module(&source, &dir).unwrap();

    let result = runtime.eval("greet('World')").unwrap();
    assert_eq!(result, tails::Value::String("Hello, World!".to_string()));
}

#[test]
fn test_multiple_named_exports() {
    let mut runtime = TailsRuntime::default();
    let source = fixture("constants.ts");
    let dir = module_dir();
    runtime.eval_module(&source, &dir).unwrap();

    let result = runtime.eval("MAX_SIZE").unwrap();
    assert_eq!(result, tails::Value::Float(100.0));

    let result = runtime.eval("MIN_SIZE").unwrap();
    assert_eq!(result, tails::Value::Float(1.0));

    let result = runtime.eval("DEFAULT_NAME").unwrap();
    assert_eq!(result, tails::Value::String("test".to_string()));
}

#[test]
fn test_module_state_isolation() {
    let mut runtime = TailsRuntime::default();
    let source = fixture("counter.ts");
    let dir = module_dir();
    runtime.eval_module(&source, &dir).unwrap();

    let result = runtime.eval("increment()").unwrap();
    assert_eq!(result, tails::Value::Float(1.0));

    let result = runtime.eval("increment()").unwrap();
    assert_eq!(result, tails::Value::Float(2.0));

    let result = runtime.eval("getCount()").unwrap();
    assert_eq!(result, tails::Value::Float(2.0));
}

#[test]
fn test_import_named_from_module() {
    let mut runtime = TailsRuntime::default();
    let source = r#"
        import { add, multiply } from "./tests/fixtures/modules/math.ts";
        add(10, 20)
    "#;
    let result = runtime.eval(source).unwrap();
    assert_eq!(result, tails::Value::Float(30.0));
}

#[test]
fn test_import_default_from_module() {
    let mut runtime = TailsRuntime::default();
    let source = r#"
        import greet from "./tests/fixtures/modules/greeter.ts";
        greet("Tails")
    "#;
    let result = runtime.eval(source).unwrap();
    assert_eq!(result, tails::Value::String("Hello, Tails!".to_string()));
}

#[test]
fn test_import_star_from_module() {
    let mut runtime = TailsRuntime::default();
    let source = r#"
        import * as math from "./tests/fixtures/modules/math.ts";
        math.add(100, 200)
    "#;
    let result = runtime.eval(source).unwrap();
    assert_eq!(result, tails::Value::Float(300.0));
}

#[test]
fn test_import_with_local_alias() {
    let mut runtime = TailsRuntime::default();
    let source = r#"
        import { add as sum } from "./tests/fixtures/modules/math.ts";
        sum(5, 7)
    "#;
    let result = runtime.eval(source).unwrap();
    assert_eq!(result, tails::Value::Float(12.0));
}

#[test]
fn test_cross_module_function_call() {
    let mut runtime = TailsRuntime::default();
    let source = r#"
        import { sumAndProduct } from "./tests/fixtures/modules/uses_math.ts";
        sumAndProduct(2, 3)
    "#;
    let result = runtime.eval(source).unwrap();
    assert_eq!(result, tails::Value::Float(11.0));
}

#[test]
fn test_import_from_chain() {
    let mut runtime = TailsRuntime::default();
    let source = r#"
        import { fromB } from "./tests/fixtures/modules/lib_b.ts";
        fromB()
    "#;
    let result = runtime.eval(source).unwrap();
    assert_eq!(result, tails::Value::String("AB".to_string()));
}

#[test]
fn test_export_default_class() {
    let mut runtime = TailsRuntime::default();
    let source = r#"
        import Calculator from "./tests/fixtures/modules/default_class.ts";
        const calc = new Calculator();
        calc.add(10).add(20).getResult()
    "#;
    let result = runtime.eval(source).unwrap();
    assert_eq!(result, tails::Value::Float(30.0));
}

#[test]
fn test_exported_values_visible() {
    let mut runtime = TailsRuntime::default();
    let source = fixture("has_own_global.ts");
    let dir = module_dir();
    runtime.eval_module(&source, &dir).unwrap();

    let result = runtime.eval("getSecret()").unwrap();
    assert_eq!(result, tails::Value::String("private".to_string()));

    let result = runtime.eval("exposed").unwrap();
    assert_eq!(result, tails::Value::Float(42.0));
}

#[test]
fn test_module_does_not_pollute_global() {
    let mut runtime = TailsRuntime::default();
    let source = fixture("has_own_global.ts");
    let dir = module_dir();
    runtime.eval_module(&source, &dir).unwrap();

    let result = runtime
        .eval("typeof mySecret !== 'undefined' ? 'leaked' : 'ok'")
        .unwrap();
    assert_eq!(result, tails::Value::String("ok".to_string()));
}

#[test]
fn test_import_multiple_modules_same_runtime() {
    let mut runtime = TailsRuntime::default();

    let math_source = fixture("math.ts");
    let dir = module_dir();
    runtime.eval_module(&math_source, &dir).unwrap();

    let constants_source = fixture("constants.ts");
    runtime.eval_module(&constants_source, &dir).unwrap();

    let result = runtime.eval("add(MAX_SIZE, MIN_SIZE)").unwrap();
    assert_eq!(result, tails::Value::Float(101.0));
}

#[test]
fn test_missing_module_throws_error() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        import foo from "./nonexistent_module.ts";
    "#,
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message().contains("Cannot find module"));
}
