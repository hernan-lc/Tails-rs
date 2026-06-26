#[test]
fn test_print_bytecode() {
    let compiler = tails::compiler::Compiler::new(false);
    let module = compiler.compile(r#"
        var p = new Promise(function(resolve, reject) {
            resolve(42);
        });
    "#).unwrap();
    
    eprintln!("Functions:");
    for (i, f) in module.functions.iter().enumerate() {
        eprintln!("  [{}] name={:?} params={:?} bytecode_index={} closure_var_count={}", 
            i, f.name, f.params, f.bytecode_index, f.closure_var_count);
    }
    
    eprintln!("\nInstructions:");
    for (i, inst) in module.instructions.iter().enumerate() {
        eprintln!("  [{}] {:?}", i, inst);
    }
}
