use tails::TailsRuntime;
use tails::Value;

// ============================================
// Object literal getters
// ============================================

#[test]
fn test_object_getter_basic() {
    let mut rt = TailsRuntime::default();
    let result = rt
        .eval(
            r#"
        const obj = {
            _x: 10,
            get x() { return this._x; }
        };
        obj.x;
    "#,
        )
        .unwrap();
    assert_eq!(result, Value::Integer(10));
}

#[test]
fn test_object_getter_computed() {
    let mut rt = TailsRuntime::default();
    let result = rt
        .eval(
            r#"
        const obj = {
            get "computed-key"() { return 42; }
        };
        obj["computed-key"];
    "#,
        )
        .unwrap();
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_object_getter_calls_function() {
    let mut rt = TailsRuntime::default();
    let result = rt
        .eval(
            r#"
        const obj = {
            _val: 5,
            get doubled() { return this._val * 2; }
        };
        obj.doubled;
    "#,
        )
        .unwrap();
    assert_eq!(result, Value::Integer(10));
}

#[test]
fn test_object_getter_not_function() {
    let mut rt = TailsRuntime::default();
    let result = rt.eval(
        r#"
        const obj = {
            get x() { return 42; }
        };
        typeof obj.x;
    "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::string("number"));
}

// ============================================
// Object literal setters
// ============================================

#[test]
fn test_object_setter_basic() {
    let mut rt = TailsRuntime::default();
    let result = rt
        .eval(
            r#"
        const obj = {
            _val: 0,
            set x(v) { this._val = v; },
            get x() { return this._val; }
        };
        obj.x = 99;
        obj.x;
    "#,
        )
        .unwrap();
    assert_eq!(result, Value::Integer(99));
}

#[test]
fn test_object_setter_only() {
    let mut rt = TailsRuntime::default();
    let result = rt
        .eval(
            r#"
        const obj = {
            _log: [],
            set data(v) { this._log.push(v); }
        };
        obj.data = "hello";
        obj.data = "world";
        obj._log.length;
    "#,
        )
        .unwrap();
    assert_eq!(result, Value::Integer(2));
}

// ============================================
// Rest parameters in function declarations
// ============================================

#[test]
fn test_rest_params_basic() {
    let mut rt = TailsRuntime::default();
    let result = rt
        .eval(
            r#"
        function sum(...args) {
            let s = 0;
            for (let i = 0; i < args.length; i++) {
                s = s + args[i];
            }
            return s;
        }
        sum(1, 2, 3);
    "#,
        )
        .unwrap();
    assert_eq!(result, Value::Integer(6));
}

#[test]
fn test_rest_params_with_normal() {
    let mut rt = TailsRuntime::default();
    let result = rt
        .eval(
            r#"
        function log(level, ...msgs) {
            return level + ":" + msgs.join(",");
        }
        log("info", "a", "b", "c");
    "#,
        )
        .unwrap();
    assert_eq!(result, Value::string("info:a,b,c"));
}

#[test]
fn test_rest_params_empty() {
    let mut rt = TailsRuntime::default();
    let result = rt
        .eval(
            r#"
        function fn(...args) {
            return args.length;
        }
        fn();
    "#,
        )
        .unwrap();
    assert_eq!(result, Value::Integer(0));
}

#[test]
fn test_rest_params_typed() {
    let mut rt = TailsRuntime::default();
    let result = rt.eval(
        r#"
        function sum(...args: number[]) {
            let s = 0;
            for (let i = 0; i < args.length; i++) {
                s = s + args[i];
            }
            return s;
        }
        sum(1, 2, 3);
    "#,
    );
    assert!(
        result.is_ok(),
        "typed rest params should parse: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), Value::Integer(6));
}

// ============================================
// Rest parameters in arrow functions
// ============================================

#[test]
fn test_rest_params_arrow() {
    let mut rt = TailsRuntime::default();
    let result = rt
        .eval(
            r#"
        const sum = (...args) => {
            let s = 0;
            for (let i = 0; i < args.length; i++) {
                s = s + args[i];
            }
            return s;
        };
        sum(10, 20, 30);
    "#,
        )
        .unwrap();
    assert_eq!(result, Value::Integer(60));
}

// ============================================
// for...in with getters (should skip __getter_ keys)
// ============================================

#[test]
fn test_forin_object_literal() {
    let mut rt = TailsRuntime::default();
    let result = rt
        .eval(
            r#"
        const obj = { a: 1, b: 2, c: 3 };
        let keys = [];
        for (const k in obj) {
            keys.push(k);
        }
        keys.sort().join(",");
    "#,
        )
        .unwrap();
    assert_eq!(result, Value::string("a,b,c"));
}

// ============================================
// Object literal with method shorthand
// ============================================

#[test]
fn test_object_method_shorthand() {
    let mut rt = TailsRuntime::default();
    let result = rt
        .eval(
            r#"
        const obj = {
            greet(name) {
                return "Hello, " + name;
            }
        };
        obj.greet("World");
    "#,
        )
        .unwrap();
    assert_eq!(result, Value::string("Hello, World"));
}

#[test]
fn test_object_method_with_rest() {
    let mut rt = TailsRuntime::default();
    let result = rt
        .eval(
            r#"
        const obj = {
            sum(...args) {
                let s = 0;
                for (let i = 0; i < args.length; i++) {
                    s = s + args[i];
                }
                return s;
            }
        };
        obj.sum(1, 2, 3, 4);
    "#,
        )
        .unwrap();
    assert_eq!(result, Value::Integer(10));
}

// ============================================
// Computed property names
// ============================================

#[test]
fn test_computed_property() {
    let mut rt = TailsRuntime::default();
    let result = rt
        .eval(
            r#"
        const key = "myKey";
        const obj = { [key]: 42 };
        obj.myKey;
    "#,
        )
        .unwrap();
    assert_eq!(result, Value::Integer(42));
}

// ============================================
// Spread in object literals
// ============================================

#[test]
fn test_spread_object() {
    let mut rt = TailsRuntime::default();
    let result = rt
        .eval(
            r#"
        const a = { x: 1 };
        const b = { y: 2 };
        const c = { ...a, ...b, z: 3 };
        c.x + c.y + c.z;
    "#,
        )
        .unwrap();
    assert_eq!(result, Value::Integer(6));
}
