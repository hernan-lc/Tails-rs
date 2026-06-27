use tails::{TailsRuntime, Value};

// ---- Symbol ----
#[test]
fn test_symbol_typeof() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"typeof Symbol();"#).unwrap();
    assert_eq!(r, Value::String("symbol".to_string()));
}

#[test]
fn test_symbol_unique() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(
            r#"
        let s1 = Symbol();
        let s2 = Symbol();
        s1 === s2;
    "#,
        )
        .unwrap();
    assert_eq!(r, Value::Boolean(false));
}

#[test]
fn test_symbol_iterator() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(
            r#"
        typeof Symbol.iterator;
    "#,
        )
        .unwrap();
    assert_eq!(r, Value::String("symbol".to_string()));
}

#[test]
fn test_symbol_for() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(
            r#"
        let s1 = Symbol.for("foo");
        let s2 = Symbol.for("foo");
        s1 === s2;
    "#,
        )
        .unwrap();
    assert_eq!(r, Value::Boolean(true));
}

#[test]
fn test_symbol_key_for() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(
            r#"
        let s = Symbol.for("bar");
        Symbol.keyFor(s);
    "#,
        )
        .unwrap();
    assert_eq!(r, Value::String("bar".to_string()));
}

// ---- for...of with iterator protocol ----
#[test]
fn test_for_of_array() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(
            r#"
        let arr = [10, 20, 30];
        let sum = 0;
        for (let x of arr) {
            sum = sum + x;
        }
        sum;
    "#,
        )
        .unwrap();
    assert_eq!(r, Value::Integer(60));
}

#[test]
fn test_for_of_string() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(
            r#"
        let s = "abc";
        let result = "";
        for (let c of s) {
            result = result + c;
        }
        result;
    "#,
        )
        .unwrap();
    assert_eq!(r, Value::String("abc".to_string()));
}

// ---- Function.prototype ----
#[test]
fn test_function_call() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(
            r#"
        function greet(greeting) {
            return greeting + " " + this.name;
        }
        greet.call({ name: "World" }, "Hello");
    "#,
        )
        .unwrap();
    assert_eq!(r, Value::String("Hello World".to_string()));
}

#[test]
fn test_function_apply() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(
            r#"
        function add(a, b) {
            return a + b;
        }
        add.apply(null, [3, 4]);
    "#,
        )
        .unwrap();
    assert_eq!(r, Value::Integer(7));
}

#[test]
fn test_function_bind() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(
            r#"
        function multiply(a, b) {
            return a * b;
        }
        let double = multiply.bind(null, 2);
        double(5);
    "#,
        )
        .unwrap();
    assert_eq!(r, Value::Integer(10));
}

// ---- Array enhancements ----
#[test]
fn test_array_is_array() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(
            r#"
        Array.isArray([1, 2, 3]);
    "#,
        )
        .unwrap();
    assert_eq!(r, Value::Boolean(true));
}

#[test]
fn test_array_is_array_not() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(
            r#"
        Array.isArray("hello");
    "#,
        )
        .unwrap();
    assert_eq!(r, Value::Boolean(false));
}

#[test]
fn test_array_of() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(
            r#"
        let arr = Array.of(1, 2, 3);
        arr.length;
    "#,
        )
        .unwrap();
    assert_eq!(r, Value::Float(3.0));
}

#[test]
fn test_array_from() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(
            r#"
        let arr = Array.from([1, 2, 3], function(x) { return x * 2; });
        arr.length;
    "#,
        )
        .unwrap();
    assert_eq!(r, Value::Float(3.0));
}

#[test]
fn test_array_fill() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(
            r#"
        let arr = [1, 2, 3, 4, 5];
        arr.fill(0, 1, 3);
        arr.join(",");
    "#,
        )
        .unwrap();
    assert_eq!(r, Value::String("1,0,0,4,5".to_string()));
}

#[test]
fn test_array_find_last() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(
            r#"
        let arr = [1, 2, 3, 4, 5];
        arr.findLast(function(x) { return x < 4; });
    "#,
        )
        .unwrap();
    assert_eq!(r, Value::Integer(3));
}

#[test]
fn test_array_find_last_index() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(
            r#"
        let arr = [1, 2, 3, 4, 5];
        arr.findLastIndex(function(x) { return x < 4; });
    "#,
        )
        .unwrap();
    assert_eq!(r, Value::Integer(2));
}

#[test]
fn test_array_last_index_of() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(
            r#"
        let arr = [1, 2, 3, 2, 1];
        arr.lastIndexOf(2);
    "#,
        )
        .unwrap();
    assert_eq!(r, Value::Integer(3));
}

// ---- Object methods ----
#[test]
fn test_object_is() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(
            r#"
        Object.is(NaN, NaN);
    "#,
        )
        .unwrap();
    assert_eq!(r, Value::Boolean(true));
}

#[test]
fn test_object_is_numbers() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(
            r#"
        Object.is(0, 0);
    "#,
        )
        .unwrap();
    assert_eq!(r, Value::Boolean(true));
}

#[test]
fn test_object_prevent_extensions() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(
            r#"
        let obj = { x: 1 };
        Object.preventExtensions(obj);
        Object.isExtensible(obj);
    "#,
        )
        .unwrap();
    assert_eq!(r, Value::Boolean(false));
}

#[test]
fn test_object_is_extensible_default() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(
            r#"
        let obj = { x: 1 };
        Object.isExtensible(obj);
    "#,
        )
        .unwrap();
    assert_eq!(r, Value::Boolean(true));
}

#[test]
fn test_object_freeze() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(
            r#"
        let obj = { x: 1 };
        Object.freeze(obj);
        Object.isFrozen(obj);
    "#,
        )
        .unwrap();
    assert_eq!(r, Value::Boolean(true));
}

#[test]
fn test_object_seal() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(
            r#"
        let obj = { x: 1 };
        Object.seal(obj);
        Object.isSealed(obj);
    "#,
        )
        .unwrap();
    assert_eq!(r, Value::Boolean(true));
}

// ---- Reflect enhancements ----
#[test]
fn test_reflect_is_extensible() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(
            r#"
        let obj = { x: 1 };
        Reflect.isExtensible(obj);
    "#,
        )
        .unwrap();
    assert_eq!(r, Value::Boolean(true));
}

#[test]
fn test_reflect_prevent_extensions() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(
            r#"
        let obj = { x: 1 };
        Reflect.preventExtensions(obj);
        Reflect.isExtensible(obj);
    "#,
        )
        .unwrap();
    assert_eq!(r, Value::Boolean(false));
}
