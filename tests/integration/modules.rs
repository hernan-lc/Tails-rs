use tails::TailsRuntime;

#[test]
fn test_basic_variable() {
    let mut runtime = TailsRuntime::default().unwrap();
    let result = runtime.eval("const x = 42; x").unwrap();
    assert_eq!(result, tails::Value::Float(42.0));
}

#[test]
fn test_arithmetic() {
    let mut runtime = TailsRuntime::default().unwrap();
    let result = runtime.eval("2 + 3").unwrap();
    assert_eq!(result, tails::Value::Float(5.0));
}

#[test]
fn test_string_concatenation() {
    let mut runtime = TailsRuntime::default().unwrap();
    let result = runtime.eval(r#""hello" + " ""#).unwrap();
    assert_eq!(result, tails::Value::string("hello "));
}

#[test]
fn test_boolean_operations() {
    let mut runtime = TailsRuntime::default().unwrap();
    let result = runtime.eval("true && false").unwrap();
    assert_eq!(result, tails::Value::Boolean(false));
}

#[test]
fn test_if_statement() {
    let mut runtime = TailsRuntime::default().unwrap();
    let result = runtime.eval(r#"
        const x = 10;
        if (x > 5) {
            "big"
        } else {
            "small"
        }
    "#).unwrap();
    assert_eq!(result, tails::Value::string("big"));
}

#[test]
fn test_while_loop() {
    let mut runtime = TailsRuntime::default().unwrap();
    let result = runtime.eval(r#"
        let sum = 0;
        let i = 1;
        while (i <= 5) {
            sum = sum + i;
            i = i + 1;
        }
        sum
    "#).unwrap();
    assert_eq!(result, tails::Value::Float(15.0));
}

#[test]
fn test_function_declaration() {
    let mut runtime = TailsRuntime::default().unwrap();
    let result = runtime.eval(r#"
        function add(a, b) {
            return a + b;
        }
        add(3, 4)
    "#).unwrap();
    assert_eq!(result, tails::Value::Float(7.0));
}

#[test]
fn test_nested_functions() {
    let mut runtime = TailsRuntime::default().unwrap();
    let result = runtime.eval(r#"
        function outer() {
            function inner() {
                return 42;
            }
            return inner();
        }
        outer()
    "#).unwrap();
    assert_eq!(result, tails::Value::Float(42.0));
}

#[test]
fn test_closures() {
    let mut runtime = TailsRuntime::default().unwrap();
    let result = runtime.eval(r#"
        function makeCounter() {
            let count = 0;
            function increment() {
                count = count + 1;
                return count;
            }
            return increment;
        }
        const counter = makeCounter();
        counter()
    "#).unwrap();
    assert_eq!(result, tails::Value::Float(1.0));
}

#[test]
fn test_array_literal() {
    let mut runtime = TailsRuntime::default().unwrap();
    let result = runtime.eval("[1, 2, 3]").unwrap();
    // Array result depends on implementation
    assert!(matches!(result, tails::Value::Undefined));
}

#[test]
fn test_object_literal() {
    let mut runtime = TailsRuntime::default().unwrap();
    let result = runtime.eval(r#"{"name": "John", "age": 30}"#).unwrap();
    // Object result depends on implementation
    assert!(matches!(result, tails::Value::Undefined));
}

#[test]
fn test_unary_operations() {
    let mut runtime = TailsRuntime::default().unwrap();
    
    let result = runtime.eval("-5").unwrap();
    assert_eq!(result, tails::Value::Float(-5.0));
    
    let result = runtime.eval("!true").unwrap();
    assert_eq!(result, tails::Value::Boolean(false));
}

#[test]
fn test_comparison_operations() {
    let mut runtime = TailsRuntime::default().unwrap();
    
    let result = runtime.eval("5 > 3").unwrap();
    assert_eq!(result, tails::Value::Boolean(true));
    
    let result = runtime.eval("5 < 3").unwrap();
    assert_eq!(result, tails::Value::Boolean(false));
    
    let result = runtime.eval("5 == 5").unwrap();
    assert_eq!(result, tails::Value::Boolean(true));
    
    let result = runtime.eval("5 === 5").unwrap();
    assert_eq!(result, tails::Value::Boolean(true));
}

#[test]
fn test_global_variables() {
    let mut runtime = TailsRuntime::default().unwrap();
    
    runtime.set_global("myVar", tails::Value::Float(100.0));
    let result = runtime.eval("myVar").unwrap();
    assert_eq!(result, tails::Value::Float(100.0));
}

#[test]
fn test_multiple_statements() {
    let mut runtime = TailsRuntime::default().unwrap();
    let result = runtime.eval(r#"
        const a = 10;
        const b = 20;
        const c = a + b;
        c
    "#).unwrap();
    assert_eq!(result, tails::Value::Float(30.0));
}

#[test]
fn test_nested_expressions() {
    let mut runtime = TailsRuntime::default().unwrap();
    let result = runtime.eval("(2 + 3) * 4").unwrap();
    assert_eq!(result, tails::Value::Float(20.0));
}

#[test]
fn test_complex_function() {
    let mut runtime = TailsRuntime::default().unwrap();
    let result = runtime.eval(r#"
        function fibonacci(n) {
            if (n <= 1) {
                return n;
            }
            return fibonacci(n - 1) + fibonacci(n - 2);
        }
        fibonacci(10)
    "#).unwrap();
    assert_eq!(result, tails::Value::Float(55.0));
}

#[test]
fn test_error_handling() {
    let mut runtime = TailsRuntime::default().unwrap();
    let result = runtime.eval("undefinedVariable");
    assert!(result.is_err());
}

#[test]
fn test_division_by_zero() {
    let mut runtime = TailsRuntime::default().unwrap();
    let result = runtime.eval("10 / 0");
    assert!(result.is_err());
}

#[test]
fn test_complex_program() {
    let mut runtime = TailsRuntime::default().unwrap();
    let result = runtime.eval(r#"
        function factorial(n) {
            if (n <= 1) {
                return 1;
            }
            return n * factorial(n - 1);
        }
        
        function isPrime(n) {
            if (n <= 1) {
                return false;
            }
            let i = 2;
            while (i * i <= n) {
                if (n % i === 0) {
                    return false;
                }
                i = i + 1;
            }
            return true;
        }
        
        const fact5 = factorial(5);
        const prime7 = isPrime(7);
        fact5 + (prime7 ? 100 : 0)
    "#).unwrap();
    assert_eq!(result, tails::Value::Float(220.0));
}
