use tails::TailsRuntime;
use tails::Value;

fn eval_num(source: &str) -> f64 {
    let mut runtime = TailsRuntime::default();
    match runtime.eval(source).unwrap() {
        Value::Integer(n) => n as f64,
        Value::Float(n) => n,
        other => panic!("expected number, got {:?}", other),
    }
}

#[test]
fn test_basic_allocation() {
    let result = eval_num(
        r#"
        let obj = { a: 1, b: 2 };
        obj.a + obj.b;
    "#,
    );
    assert_eq!(result, 3.0);
}

#[test]
fn test_unreachable_objects_collected() {
    let result = eval_num(
        r#"
        let result = 0;
        for (let i = 0; i < 100; i++) {
            let temp = { value: i };
            result = temp.value;
        }
        result;
    "#,
    );
    assert_eq!(result, 99.0);
}

#[test]
fn test_reachable_objects_survive() {
    let result = eval_num(
        r#"
        let kept = { x: 42 };
        for (let i = 0; i < 100; i++) {
            let temp = { value: i };
        }
        kept.x;
    "#,
    );
    assert_eq!(result, 42.0);
}

#[test]
fn test_gc_does_not_break_functions() {
    let result = eval_num(
        r#"
        function add(a, b) { return a + b; }
        let result = 0;
        for (let i = 0; i < 100; i++) {
            let temp = { v: i };
            result = add(result, 1);
        }
        result;
    "#,
    );
    assert_eq!(result, 100.0);
}

#[test]
fn test_gc_with_closures() {
    let result = eval_num(
        r#"
        function makeCounter() {
            let count = 0;
            return function() {
                count = count + 1;
                return count;
            };
        }
        let counter = makeCounter();
        let result = 0;
        for (let i = 0; i < 50; i++) {
            result = counter();
        }
        result;
    "#,
    );
    assert_eq!(result, 50.0);
}

#[test]
fn test_gc_with_arrays() {
    let result = eval_num(
        r#"
        let arr = [1, 2, 3, 4, 5];
        let sum = 0;
        for (let i = 0; i < arr.length; i++) {
            sum = sum + arr[i];
        }
        sum;
    "#,
    );
    assert_eq!(result, 15.0);
}

#[test]
fn test_gc_with_nested_objects() {
    let result = eval_num(
        r#"
        let outer = {
            inner: { value: 10 },
            other: { value: 20 }
        };
        outer.inner.value + outer.other.value;
    "#,
    );
    assert_eq!(result, 30.0);
}

#[test]
fn test_gc_with_string_operations() {
    let result = eval_num(
        r#"
        let result = "";
        for (let i = 0; i < 10; i++) {
            result = result + "x";
        }
        result.length;
    "#,
    );
    assert_eq!(result, 10.0);
}

#[test]
fn test_multiple_gc_cycles() {
    let result = eval_num(
        r#"
        let sum = 0;
        for (let cycle = 0; cycle < 5; cycle++) {
            for (let i = 0; i < 100; i++) {
                let temp = { cycle: cycle, i: i };
            }
            sum = sum + cycle;
        }
        sum;
    "#,
    );
    assert_eq!(result, 10.0);
}

#[test]
fn test_gc_with_global_objects() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        let globalObj = { name: "global" };
        let result = "";
        for (let i = 0; i < 50; i++) {
            let temp = { value: i };
        }
        globalObj.name;
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::string("global"));
}

#[test]
fn test_gc_with_large_allocations() {
    let result = eval_num(
        r#"
        let sum = 0;
        for (let i = 0; i < 500; i++) {
            let obj = { a: i, b: i + 1, c: i + 2 };
            sum = sum + obj.a;
        }
        sum;
    "#,
    );
    assert_eq!(result, 124750.0);
}

#[test]
fn test_gc_preserves_this() {
    let result = eval_num(
        r#"
        function Method() {
            this.value = 42;
            for (let i = 0; i < 100; i++) {
                let temp = { v: i };
            }
        }
        let obj = new Method();
        obj.value;
    "#,
    );
    assert_eq!(result, 42.0);
}

#[test]
fn test_gc_with_classes() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        class Person {
            constructor(name, age) {
                this.name = name;
                this.age = age;
            }
        }
        let p = new Person("Alice", 30);
        for (let i = 0; i < 100; i++) {
            let temp = new Person("temp", i);
        }
        p.name;
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::string("Alice"));
}

#[test]
fn test_existing_basic_still_works() {
    let mut runtime = TailsRuntime::default();
    assert!(runtime.eval("42;").is_ok());
    assert!(runtime.eval("2 + 3;").is_ok());
    assert!(runtime.eval(r#""hello";"#).is_ok());
    assert!(runtime.eval("true;").is_ok());
    assert!(runtime.eval("null;").is_ok());
    assert!(runtime.eval("undefined;").is_ok());
}

#[test]
fn test_existing_functions_still_work() {
    let result = eval_num(
        r#"
        function factorial(n) {
            if (n <= 1) return 1;
            return n * factorial(n - 1);
        }
        factorial(5);
    "#,
    );
    assert_eq!(result, 120.0);
}

#[test]
fn test_gc_with_prototype_chain() {
    let result = eval_num(
        r#"
        let parent = { x: 10 };
        let child = { y: 20 };
        let sum = 0;
        for (let i = 0; i < 50; i++) {
            let temp = { v: i };
            sum = sum + 1;
        }
        parent.x + child.y;
    "#,
    );
    assert_eq!(result, 30.0);
}

#[test]
fn test_gc_reuse_after_collection() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        let sum = 0;
        for (let round = 0; round < 5; round++) {
            let temp = { round: round };
            sum = sum + temp.round;
        }
        sum;
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Float(10.0));
}
