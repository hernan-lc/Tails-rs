// ============================================================
// Tails-rs — Custom Runtime Example (Rust API)
// Run:  cargo run --example custom_runtime
// ============================================================

use tails::{RuntimeConfig, TailsRuntime, Value};

fn main() -> anyhow::Result<()> {
    println!("=== Custom Runtime Demo ===\n");

    // 1. Basic runtime with defaults
    let mut rt = TailsRuntime::default();
    println!("eval 2+2: {:?}", rt.eval("2 + 2")?);

    // 2. Runtime with type checking enabled
    let mut rt = TailsRuntime::new(RuntimeConfig {
        enable_type_checking: true,
        max_heap_size: 64 * 1024 * 1024,
    })?;

    // 3. Set a custom global from Rust, use it in TS
    rt.set_global("rustGreeting", Value::String("Hello from Rust!".into()));
    let result = rt.eval(r#"rustGreeting + " from TypeScript""#)?;
    println!("cross-language: {}", result);

    // 4. Read a value back from the runtime
    rt.eval("let answer = 42;")?;
    println!("global 'answer': {:?}", rt.get_global("answer"));

    // 5. Evaluate a module with exports
    let result = rt.eval_module(
        r#"
        export function multiply(a: number, b: number): number {
            return a * b;
        }
        export const VERSION = "1.0";
        export default multiply;
        "#,
        std::path::Path::new("."),
    )?;
    println!("module eval result: {:?}", result);

    // 6. Read a specific export by name
    if let Some(add_val) = rt.get_module_export(".", "multiply") {
        println!("export 'multiply': {:?}", add_val);
    }
    if let Some(ver_val) = rt.get_module_export(".", "VERSION") {
        println!("export 'VERSION': {}", ver_val);
    }

    // 7. Import a TS file by path
    let import_path = std::path::Path::new("examples/math_utils.ts");
    let import_result = rt.import(import_path)?;
    println!("imported math_utils: {:?}", import_result);
    if let Some(add_val) = rt.get_module_export(import_path.to_string_lossy().as_ref(), "add") {
        println!("math_utils.add: {:?}", add_val);
    }
    if let Some(pi_val) = rt.get_module_export(import_path.to_string_lossy().as_ref(), "PI") {
        println!("math_utils.PI: {}", pi_val);
    }

    // 8. Multiple independent runtimes (e.g. isolated contexts)
    let mut rt_a = TailsRuntime::default();
    let mut rt_b = TailsRuntime::default();
    rt_a.set_global("ctx", Value::String("A".into()));
    rt_b.set_global("ctx", Value::String("B".into()));
    println!("runtime A: {}", rt_a.eval("ctx")?);
    println!("runtime B: {}", rt_b.eval("ctx")?);

    println!("\n=== Done ===");
    Ok(())
}
