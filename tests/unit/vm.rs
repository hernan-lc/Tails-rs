use tails::TailsRuntime;

#[test]
fn test_load_constant() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval("42").unwrap();
    assert_eq!(result, tails::Value::Float(42.0));
}

#[test]
fn test_store_global() {
    let mut runtime = TailsRuntime::default();
    runtime.eval("const x = 10;").unwrap();
    let result = runtime.get_global("x").unwrap();
    assert_eq!(result, tails::Value::Float(10.0));
}

#[test]
fn test_load_global() {
    let mut runtime = TailsRuntime::default();
    runtime.set_global("myVar", tails::Value::Float(100.0));
    let result = runtime.eval("myVar").unwrap();
    assert_eq!(result, tails::Value::Float(100.0));
}

#[test]
fn test_addition() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval("2 + 3").unwrap();
    assert_eq!(result, tails::Value::Float(5.0));
}

#[test]
fn test_subtraction() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval("10 - 4").unwrap();
    assert_eq!(result, tails::Value::Float(6.0));
}

#[test]
fn test_multiplication() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval("3 * 4").unwrap();
    assert_eq!(result, tails::Value::Float(12.0));
}

#[test]
fn test_division() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval("10 / 2").unwrap();
    assert_eq!(result, tails::Value::Float(5.0));
}

#[test]
fn test_modulo() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval("10 % 3").unwrap();
    assert_eq!(result, tails::Value::Float(1.0));
}

#[test]
fn test_negate() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval("-5").unwrap();
    assert_eq!(result, tails::Value::Float(-5.0));
}

#[test]
fn test_not() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval("!true").unwrap();
    assert_eq!(result, tails::Value::Boolean(false));
}

#[test]
fn test_string_addition() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(r#""hello" + " ""#).unwrap();
    assert_eq!(result, tails::Value::String("hello ".to_string()));
}

#[test]
fn test_boolean_operations() {
    let mut runtime = TailsRuntime::default();
    
    let result = runtime.eval("true && false").unwrap();
    assert_eq!(result, tails::Value::Boolean(false));
    
    let result = runtime.eval("true || false").unwrap();
    assert_eq!(result, tails::Value::Boolean(true));
}

#[test]
fn test_comparison_operations() {
    let mut runtime = TailsRuntime::default();
    
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
fn test_if_else() {
    let mut runtime = TailsRuntime::default();
    
    let result = runtime.eval("let x; if (true) { x = 1; } else { x = 2; } x").unwrap();
    assert_eq!(result, tails::Value::Float(1.0));
    
    let result = runtime.eval("let x; if (false) { x = 1; } else { x = 2; } x").unwrap();
    assert_eq!(result, tails::Value::Float(2.0));
}

#[test]
fn test_while_loop() {
    let mut runtime = TailsRuntime::default();
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
fn test_function_declaration_and_call() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(r#"
        function add(a, b) {
            return a + b;
        }
        add(3, 4)
    "#).unwrap();
    assert_eq!(result, tails::Value::Float(7.0));
}

#[test]
fn test_nested_function_calls() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(r#"
        function square(x) {
            return x * x;
        }
        function sum(a, b) {
            return a + b;
        }
        sum(square(3), square(4))
    "#).unwrap();
    assert_eq!(result, tails::Value::Float(25.0));
}

#[test]
fn test_recursive_function() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(r#"
        function factorial(n) {
            if (n <= 1) {
                return 1;
            }
            return n * factorial(n - 1);
        }
        factorial(5)
    "#).unwrap();
    assert_eq!(result, tails::Value::Float(120.0));
}

#[test]
fn test_fibonacci() {
    let mut runtime = TailsRuntime::default();
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
fn test_global_variable_manipulation() {
    let mut runtime = TailsRuntime::default();
    
    runtime.set_global("x", tails::Value::Float(10.0));
    let result = runtime.eval("x + 5").unwrap();
    assert_eq!(result, tails::Value::Float(15.0));
    
    runtime.eval("x = x * 2").unwrap();
    let result = runtime.get_global("x").unwrap();
    assert_eq!(result, tails::Value::Float(20.0));
}

#[test]
fn test_multiple_variables() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(r#"
        const a = 10;
        const b = 20;
        const c = a + b;
        c
    "#).unwrap();
    assert_eq!(result, tails::Value::Float(30.0));
}

#[test]
fn test_complex_program() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(r#"
        function isEven(n) {
            return n % 2 === 0;
        }
        
        function sumArray(arr) {
            let sum = 0;
            let i = 0;
            while (i < arr.length) {
                sum = sum + arr[i];
                i = i + 1;
            }
            return sum;
        }
        
        const numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let evenSum = 0;
        let i = 0;
        while (i < numbers.length) {
            if (isEven(numbers[i])) {
                evenSum = evenSum + numbers[i];
            }
            i = i + 1;
        }
        evenSum
    "#).unwrap();
    assert_eq!(result, tails::Value::Float(30.0));
}

#[test]
fn test_error_handling() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval("undefinedVariable").unwrap();
    assert_eq!(result, tails::Value::Undefined);
}

#[test]
fn test_division_by_zero() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval("10 / 0").unwrap();
    assert_eq!(result, tails::Value::Float(f64::INFINITY));
}

#[test]
fn test_nested_blocks() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(r#"
        const x = 10;
        const y = 20;
        const z = 30;
        x + y + z
    "#).unwrap();
    assert_eq!(result, tails::Value::Float(60.0));
}

#[test]
fn test_complex_control_flow() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(r#"
        let result = 0;
        for (let i = 0; i < 10; i = i + 1) {
            if (i % 2 === 0) {
                result = result + i;
            } else {
                result = result - i;
            }
        }
        result
    "#).unwrap();
    assert_eq!(result, tails::Value::Float(-5.0));
}
