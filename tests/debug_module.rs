use std::path::Path;
use tails::TailsRuntime;

#[test]
fn debug_counter() {
    let mut runtime = TailsRuntime::default();
    let source = r#"
let count = 0;
export function increment() {
    count = count + 1;
    return count;
}
export function getCount() {
    return count;
}
"#;
    let dir = Path::new(".");
    runtime.eval_module(source, dir).unwrap();

    let count_val = runtime.get_global("count");
    let inc_val = runtime.get_global("increment");
    let get_val = runtime.get_global("getCount");
    eprintln!(
        "After eval_module: count={:?} increment={:?} getCount={:?}",
        count_val, inc_val, get_val
    );

    match runtime.eval("increment()") {
        Ok(v) => eprintln!("increment() = {:?}", v),
        Err(e) => eprintln!("increment() ERROR: {:?}", e),
    }
}
