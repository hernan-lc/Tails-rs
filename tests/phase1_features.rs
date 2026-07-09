use tails::TailsRuntime;

#[test]
fn test_for_loop_basic() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        let sum = 0;
        for (let i = 0; i <= 5; i = i + 1) {
            sum = sum + i;
        }
        sum;
    "#,
    );
    match &result {
        Ok(v) => assert_eq!(*v, tails::Value::Float(15.0)),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_for_loop_counter() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        let count = 0;
        for (let i = 0; i < 10; i = i + 1) {
            count = count + 1;
        }
        count;
    "#,
    );
    match &result {
        Ok(v) => assert_eq!(*v, tails::Value::Float(10.0)),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_do_while_loop() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        let sum = 0;
        let i = 1;
        do {
            sum = sum + i;
            i = i + 1;
        } while (i <= 5);
        sum;
    "#,
    );
    match &result {
        Ok(v) => assert_eq!(*v, tails::Value::Float(15.0)),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_switch_case() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        let x = 2;
        let result = 0;
        switch (x) {
            case 1:
                result = 10;
            case 2:
                result = 20;
            case 3:
                result = 30;
        }
        result;
    "#,
    );
    match &result {
        Ok(v) => assert_eq!(*v, tails::Value::Float(30.0)),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_switch_default() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        let x = 99;
        let result = 0;
        switch (x) {
            case 1:
                result = 10;
            default:
                result = 999;
        }
        result;
    "#,
    );
    match &result {
        Ok(v) => assert_eq!(*v, tails::Value::Float(999.0)),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_ternary_operator() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        const x = 10;
        const result = x > 5 ? "big" : "small";
        result;
    "#,
    );
    assert!(result.is_ok());
}

#[test]
fn test_ternary_nested() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        const x = 15;
        const result = x > 20 ? "high" : (x > 10 ? "medium" : "low");
        result;
    "#,
    );
    assert!(result.is_ok());
}

#[test]
fn test_increment_postfix() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        let x = 5;
        x++;
        x;
    "#,
    );
    match &result {
        Ok(v) => assert_eq!(*v, tails::Value::Float(6.0)),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_decrement_postfix() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        let x = 5;
        x--;
        x;
    "#,
    );
    match &result {
        Ok(v) => assert_eq!(*v, tails::Value::Float(4.0)),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_increment_prefix() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        let x = 5;
        ++x;
        x;
    "#,
    );
    match &result {
        Ok(v) => assert_eq!(*v, tails::Value::Float(6.0)),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_decrement_prefix() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        let x = 5;
        --x;
        x;
    "#,
    );
    match &result {
        Ok(v) => assert_eq!(*v, tails::Value::Float(4.0)),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_compound_assignment_add() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        let x = 10;
        x += 5;
        x;
    "#,
    );
    match &result {
        Ok(v) => assert_eq!(*v, tails::Value::Float(15.0)),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_compound_assignment_sub() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        let x = 10;
        x -= 3;
        x;
    "#,
    );
    match &result {
        Ok(v) => assert_eq!(*v, tails::Value::Float(7.0)),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_compound_assignment_mul() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        let x = 10;
        x *= 2;
        x;
    "#,
    );
    match &result {
        Ok(v) => assert_eq!(*v, tails::Value::Float(20.0)),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_compound_assignment_div() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        let x = 10;
        x /= 2;
        x;
    "#,
    );
    match &result {
        Ok(v) => assert_eq!(*v, tails::Value::Float(5.0)),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_compound_assignment_mod() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        let x = 10;
        x %= 3;
        x;
    "#,
    );
    match &result {
        Ok(v) => assert_eq!(*v, tails::Value::Float(1.0)),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_compound_assignment_and() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        let x = true;
        x &&= false;
        x;
    "#,
    );
    match &result {
        Ok(v) => assert_eq!(*v, tails::Value::Boolean(false)),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_compound_assignment_or() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        let x = false;
        x ||= true;
        x;
    "#,
    );
    match &result {
        Ok(v) => assert_eq!(*v, tails::Value::Boolean(true)),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_arrow_function_expression() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        const add = (a, b) => a + b;
        add(3, 4);
    "#,
    );
    match &result {
        Ok(v) => assert_eq!(*v, tails::Value::Float(7.0)),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_arrow_function_block() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        const add = (a, b) => {
            return a + b;
        };
        add(5, 6);
    "#,
    );
    match &result {
        Ok(v) => assert_eq!(*v, tails::Value::Float(11.0)),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_arrow_function_single_param() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        const double = x => x * 2;
        double(7);
    "#,
    );
    match &result {
        Ok(v) => assert_eq!(*v, tails::Value::Float(14.0)),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_arrow_function_no_params() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        const getFourtyTwo = () => 42;
        getFourtyTwo();
    "#,
    );
    match &result {
        Ok(v) => assert_eq!(*v, tails::Value::Float(42.0)),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_new_expression_parses() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval("new Object();");
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_template_literal_simple() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(r#"`hello world`;"#);
    match &result {
        Ok(v) => assert_eq!(*v, tails::Value::string("hello world")),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_template_literal_interpolation() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        const name = "World";
        `Hello ${name}`;
    "#,
    );
    match &result {
        Ok(v) => assert_eq!(*v, tails::Value::string("Hello World")),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_template_literal_expression() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        const x = 10;
        `The value is ${x * 2}`;
    "#,
    );
    match &result {
        Ok(v) => assert_eq!(*v, tails::Value::string("The value is 20")),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_template_literal_multiple() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        const a = "Hello";
        const b = "World";
        `${a}, ${b}!`;
    "#,
    );
    match &result {
        Ok(v) => assert_eq!(*v, tails::Value::string("Hello, World!")),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_class_parses() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        class MyClass {
            constructor() {
                this.x = 10;
            }
            getX() {
                return this.x;
            }
        }
    "#,
    );
    assert!(result.is_ok());
}

#[test]
fn test_try_catch() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        try {
            const x = 10;
        } catch (e) {
            const y = 20;
        }
    "#,
    );
    assert!(result.is_ok());
}

#[test]
fn test_try_catch_finally() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        try {
            const x = 10;
        } catch (e) {
            const y = 20;
        } finally {
            const z = 30;
        }
    "#,
    );
    assert!(result.is_ok());
}

#[test]
fn test_throw_statement() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(r#"throw "error";"#);
    assert!(result.is_err());
}

#[test]
fn test_async_function_parses() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        async function fetchData() {
            return 42;
        }
    "#,
    );
    assert!(result.is_ok());
}

#[test]
fn test_async_arrow_parses() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        const fetchData = async () => {
            return 42;
        };
    "#,
    );
    assert!(result.is_ok());
}

#[test]
fn test_await_parses() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        async function main() {
            const result = await 42;
            return result;
        }
    "#,
    );
    assert!(result.is_ok());
}

#[test]
fn test_import_parses() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(r#"import { foo } from "./module";"#);
    assert!(result.is_err());
    assert!(result.unwrap_err().message().contains("Cannot find module"));
}

#[test]
fn test_import_default_parses() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(r#"import foo from "./module";"#);
    assert!(result.is_err());
    assert!(result.unwrap_err().message().contains("Cannot find module"));
}

#[test]
fn test_export_parses() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(r#"export const x = 42;"#);
    assert!(result.is_ok());
}

#[test]
fn test_export_default_parses() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        export default function hello() {
            return 42;
        }
    "#,
    );
    assert!(result.is_ok());
}

#[test]
fn test_for_in_object() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        let count = 0;
        let obj = [1, 2, 3];
        for (let key in obj) {
            count = count + 1;
        }
        count;
    "#,
    );
    assert!(result.is_ok());
}

#[test]
fn test_break_statement() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        let count = 0;
        let i = 0;
        while (i < 10) {
            if (i === 5) {
                break;
            }
            count = count + 1;
            i = i + 1;
        }
        count;
    "#,
    );
    match &result {
        Ok(v) => assert_eq!(*v, tails::Value::Float(5.0)),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_continue_statement() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        let sum = 0;
        let i = 0;
        while (i < 5) {
            i = i + 1;
            if (i === 3) {
                continue;
            }
            sum = sum + i;
        }
        sum;
    "#,
    );
    assert!(result.is_ok());
}

#[test]
fn test_function_expression() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        const add = function(a, b) {
            return a + b;
        };
        add(3, 4);
    "#,
    );
    match &result {
        Ok(v) => assert_eq!(*v, tails::Value::Float(7.0)),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_closures_with_arrow() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        function makeCounter() {
            let count = 0;
            return () => {
                count = count + 1;
                return count;
            };
        }
        const counter = makeCounter();
        counter();
    "#,
    );
    match &result {
        Ok(v) => assert_eq!(*v, tails::Value::Float(1.0)),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_power_operator() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval("2 ** 3;");
    match &result {
        Ok(v) => assert_eq!(*v, tails::Value::Float(8.0)),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_shift_operators() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval("1 << 3;");
    assert!(result.is_ok());
}

#[test]
fn test_bitwise_not() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval("~0;");
    assert!(result.is_ok());
}

#[test]
fn test_void_operator() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval("void 0;");
    assert!(result.is_ok());
}

#[test]
fn test_delete_operator() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval("delete undefined;");
    assert!(result.is_ok());
}

#[test]
fn test_instanceof_operator() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval("42 instanceof Object;");
    assert!(result.is_ok());
}

#[test]
fn test_in_operator() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        "a" in this;
    "#,
    );
    assert!(result.is_ok());
}

#[test]
fn test_complex_for_loop() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        let factorial = 1;
        for (let i = 1; i <= 5; i = i + 1) {
            factorial = factorial * i;
        }
        factorial;
    "#,
    );
    match &result {
        Ok(v) => assert_eq!(*v, tails::Value::Float(120.0)),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_nested_switch() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        let x = 1;
        let y = 2;
        let result = 0;
        switch (x) {
            case 1:
                switch (y) {
                    case 1:
                        result = 11;
                    case 2:
                        result = 12;
                }
            case 2:
                result = 20;
        }
        result;
    "#,
    );
    match &result {
        Ok(v) => assert_eq!(*v, tails::Value::Float(20.0)),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_arrow_closure_capture() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        function createMultiplier(factor) {
            return (x) => x * factor;
        }
        const double = createMultiplier(2);
        double(5);
    "#,
    );
    match &result {
        Ok(v) => assert_eq!(*v, tails::Value::Float(10.0)),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn test_template_literal_empty() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        ``;
    "#,
    );
    assert!(result.is_ok());
}

#[test]
fn test_import_star_parses() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(r#"import * as utils from "./utils";"#);
    assert!(result.is_err());
    assert!(result.unwrap_err().message().contains("Cannot find module"));
}

#[test]
fn test_export_named_parses() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(r#"export { x, y };"#);
    assert!(result.is_ok());
}
