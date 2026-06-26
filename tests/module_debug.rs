use tails::TailsRuntime;
use std::path::Path;

#[test]
fn test_basic_call() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval("function add(a, b) { return a + b; } add(2, 3)").unwrap();
    assert_eq!(result, tails::Value::Float(5.0));
}

#[test]
fn test_inline_module() {
    let mut runtime = TailsRuntime::default();
    let source = r#"
        export function add(a, b) {
            return a + b;
        }
        export const PI = 3.14159;
    "#;
    let dir = Path::new(".");
    runtime.eval_module(&source, dir).unwrap();
    let result = runtime.eval("add(2, 3)").unwrap();
    assert_eq!(result, tails::Value::Float(5.0));
}
