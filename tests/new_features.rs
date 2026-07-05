use tails::TailsRuntime;

// ============================================================================
// String / Number / Boolean constructors
// ============================================================================

#[test]
fn test_string_constructor_from_number() {
    let mut rt = TailsRuntime::default();
    let result = rt.eval(r#"String(42);"#).unwrap();
    assert_eq!(result.to_string(), "42");
}

#[test]
fn test_string_constructor_from_boolean() {
    let mut rt = TailsRuntime::default();
    let result = rt.eval(r#"String(true);"#).unwrap();
    assert_eq!(result.to_string(), "true");
}

#[test]
fn test_string_constructor_from_null() {
    let mut rt = TailsRuntime::default();
    let result = rt.eval(r#"String(null);"#).unwrap();
    assert_eq!(result.to_string(), "null");
}

#[test]
fn test_string_constructor_from_undefined() {
    let mut rt = TailsRuntime::default();
    let result = rt.eval(r#"String(undefined);"#).unwrap();
    assert_eq!(result.to_string(), "undefined");
}

#[test]
fn test_string_constructor_from_string() {
    let mut rt = TailsRuntime::default();
    let result = rt.eval(r#"String("hello");"#).unwrap();
    assert_eq!(result.to_string(), "hello");
}

#[test]
fn test_string_constructor_no_args() {
    let mut rt = TailsRuntime::default();
    let result = rt.eval(r#"String();"#).unwrap();
    assert_eq!(result.to_string(), "undefined");
}

#[test]
fn test_number_constructor_from_string() {
    let mut rt = TailsRuntime::default();
    let result = rt.eval(r#"Number("42");"#).unwrap();
    assert_eq!(result, tails::Value::Float(42.0));
}

#[test]
fn test_number_constructor_from_boolean() {
    let mut rt = TailsRuntime::default();
    let result = rt.eval(r#"Number(true);"#).unwrap();
    assert_eq!(result, tails::Value::Float(1.0));
}

#[test]
fn test_number_constructor_from_null() {
    let mut rt = TailsRuntime::default();
    let result = rt.eval(r#"Number(null);"#).unwrap();
    assert_eq!(result, tails::Value::Float(0.0));
}

#[test]
fn test_number_constructor_from_undefined() {
    let mut rt = TailsRuntime::default();
    let result = rt.eval(r#"Number(undefined);"#).unwrap();
    assert!(matches!(result, tails::Value::Float(f) if f.is_nan()));
}

#[test]
fn test_number_constructor_no_args() {
    let mut rt = TailsRuntime::default();
    let result = rt.eval(r#"Number();"#).unwrap();
    assert!(matches!(result, tails::Value::Float(f) if f.is_nan()));
}

#[test]
fn test_number_static_is_finite() {
    let mut rt = TailsRuntime::default();
    let result = rt.eval(r#"Number.isFinite(42);"#).unwrap();
    assert_eq!(result, tails::Value::Boolean(true));
}

#[test]
fn test_number_static_is_nan() {
    let mut rt = TailsRuntime::default();
    let result = rt.eval(r#"Number.isNaN(NaN);"#).unwrap();
    assert_eq!(result, tails::Value::Boolean(true));
}

#[test]
fn test_boolean_constructor_from_truthy() {
    let mut rt = TailsRuntime::default();
    let result = rt.eval(r#"Boolean(1);"#).unwrap();
    assert_eq!(result, tails::Value::Boolean(true));
}

#[test]
fn test_boolean_constructor_from_falsy() {
    let mut rt = TailsRuntime::default();
    let result = rt.eval(r#"Boolean(0);"#).unwrap();
    assert_eq!(result, tails::Value::Boolean(false));
}

#[test]
fn test_boolean_constructor_from_string() {
    let mut rt = TailsRuntime::default();
    let result = rt.eval(r#"Boolean("hello");"#).unwrap();
    assert_eq!(result, tails::Value::Boolean(true));
}

#[test]
fn test_boolean_constructor_empty_string() {
    let mut rt = TailsRuntime::default();
    let result = rt.eval(r#"Boolean("");"#).unwrap();
    assert_eq!(result, tails::Value::Boolean(false));
}

#[test]
fn test_boolean_constructor_no_args() {
    let mut rt = TailsRuntime::default();
    let result = rt.eval(r#"Boolean();"#).unwrap();
    assert_eq!(result, tails::Value::Boolean(false));
}

// ============================================================================
// Destructuring assignment expressions
// ============================================================================

#[test]
fn test_destructuring_assignment_array() {
    let mut rt = TailsRuntime::default();
    let result = rt
        .eval(
            r#"
        let a, b;
        [a, b] = [10, 20];
        a + b;
    "#,
        )
        .unwrap();
    assert_eq!(result.to_string(), "30");
}

#[test]
fn test_destructuring_assignment_with_expressions() {
    let mut rt = TailsRuntime::default();
    let result = rt
        .eval(
            r#"
        let a, b;
        [a, b] = [1 + 2, 3 * 4];
        a + b;
    "#,
        )
        .unwrap();
    assert_eq!(result.to_string(), "15");
}

#[test]
fn test_destructuring_assignment_reassign_existing() {
    let mut rt = TailsRuntime::default();
    let result = rt
        .eval(
            r#"
        let a = 1, b = 2;
        [a, b] = [b, a];
        a * 10 + b;
    "#,
        )
        .unwrap();
    assert_eq!(result.to_string(), "21");
}

#[test]
fn test_destructuring_assignment_three_elements() {
    let mut rt = TailsRuntime::default();
    let result = rt
        .eval(
            r#"
        let x, y, z;
        [x, y, z] = [100, 200, 300];
        x + y + z;
    "#,
        )
        .unwrap();
    assert_eq!(result.to_string(), "600");
}

#[test]
fn test_bigint_fibonacci_destructuring() {
    let mut rt = TailsRuntime::default();
    let result = rt
        .eval(
            r#"
        let [a, b] = [0n, 1n];
        const fib = [];
        while (fib.length < 10) {
            fib.push(a);
            [a, b] = [b, a + b];
        }
        fib.at(-1);
    "#,
        )
        .unwrap();
    // BigInt values include the 'n' suffix in their string representation
    assert_eq!(result.to_string(), "34n");
}

#[test]
fn test_destructuring_assignment_object() {
    let mut rt = TailsRuntime::default();
    let result = rt
        .eval(
            r#"
        let x, y;
        ({x, y} = {x: 5, y: 10});
        x + y;
    "#,
        )
        .unwrap();
    assert_eq!(result.to_string(), "15");
}

// ============================================================================
// Private class fields (#field)
// ============================================================================

#[test]
fn test_private_field_basic() {
    let mut rt = TailsRuntime::default();
    let result = rt
        .eval(
            r#"
        class Counter {
            #count;
            constructor() {
                this.#count = 0;
            }
            increment() {
                this.#count = this.#count + 1;
            }
            get() {
                return this.#count;
            }
        }
        const c = new Counter();
        c.increment();
        c.increment();
        c.increment();
        c.get();
    "#,
        )
        .unwrap();
    assert_eq!(result, tails::Value::Float(3.0));
}

#[test]
fn test_private_field_with_getter() {
    let mut rt = TailsRuntime::default();
    let result = rt
        .eval(
            r#"
        class Vec {
            #x;
            #y;
            constructor(x, y) {
                this.#x = x;
                this.#y = y;
            }
            get length() {
                return Math.hypot(this.#x, this.#y);
            }
        }
        const v = new Vec(3, 4);
        v.length;
    "#,
        )
        .unwrap();
    assert_eq!(result, tails::Value::Float(5.0));
}

#[test]
fn test_private_field_multiple_instances() {
    let mut rt = TailsRuntime::default();
    let result = rt
        .eval(
            r#"
        class Box {
            #value;
            constructor(v) {
                this.#value = v;
            }
            get() {
                return this.#value;
            }
        }
        const a = new Box(10);
        const b = new Box(20);
        a.get() + b.get();
    "#,
        )
        .unwrap();
    assert_eq!(result.to_string(), "30");
}

#[test]
fn test_private_field_stored_with_hash_prefix() {
    let mut rt = TailsRuntime::default();
    // Private fields are stored with '#' prefix in the property map,
    // so they are accessible as regular properties from outside
    let result = rt
        .eval(
            r#"
        class Foo {
            #secret;
            constructor() {
                this.#secret = 42;
            }
        }
        const f = new Foo();
        f.#secret;
    "#,
        )
        .unwrap();
    assert_eq!(result, tails::Value::Float(42.0));
}

// ============================================================================
// Array.prototype.at()
// ============================================================================

#[test]
fn test_array_at_positive_index() {
    let mut rt = TailsRuntime::default();
    let result = rt.eval(r#"[10, 20, 30].at(1);"#).unwrap();
    assert_eq!(result, tails::Value::Float(20.0));
}

#[test]
fn test_array_at_negative_index() {
    let mut rt = TailsRuntime::default();
    let result = rt.eval(r#"[10, 20, 30].at(-1);"#).unwrap();
    assert_eq!(result, tails::Value::Float(30.0));
}

#[test]
fn test_array_at_negative_index_middle() {
    let mut rt = TailsRuntime::default();
    let result = rt.eval(r#"[10, 20, 30].at(-2);"#).unwrap();
    assert_eq!(result, tails::Value::Float(20.0));
}

#[test]
fn test_array_at_out_of_bounds() {
    let mut rt = TailsRuntime::default();
    let result = rt.eval(r#"[10, 20].at(5);"#).unwrap();
    assert_eq!(result, tails::Value::Undefined);
}

#[test]
fn test_array_at_first() {
    let mut rt = TailsRuntime::default();
    let result = rt.eval(r#"[10, 20, 30].at(0);"#).unwrap();
    assert_eq!(result, tails::Value::Float(10.0));
}

// ============================================================================
// Math.hypot / Math.trunc / Math.sign
// ============================================================================

#[test]
fn test_math_hypot_two_args() {
    let mut rt = TailsRuntime::default();
    let result = rt.eval(r#"Math.hypot(3, 4);"#).unwrap();
    assert_eq!(result, tails::Value::Float(5.0));
}

#[test]
fn test_math_hypot_one_arg() {
    let mut rt = TailsRuntime::default();
    let result = rt.eval(r#"Math.hypot(5);"#).unwrap();
    assert_eq!(result, tails::Value::Float(5.0));
}

#[test]
fn test_math_hypot_zero_args() {
    let mut rt = TailsRuntime::default();
    let result = rt.eval(r#"Math.hypot();"#).unwrap();
    assert_eq!(result, tails::Value::Float(0.0));
}

#[test]
fn test_math_hypot_many_args() {
    let mut rt = TailsRuntime::default();
    let result = rt.eval(r#"Math.hypot(1, 2, 2);"#).unwrap();
    assert_eq!(result, tails::Value::Float(3.0));
}

#[test]
fn test_math_trunc_positive() {
    let mut rt = TailsRuntime::default();
    let result = rt.eval(r#"Math.trunc(3.7);"#).unwrap();
    assert_eq!(result, tails::Value::Float(3.0));
}

#[test]
fn test_math_trunc_negative() {
    let mut rt = TailsRuntime::default();
    let result = rt.eval(r#"Math.trunc(-3.7);"#).unwrap();
    assert_eq!(result, tails::Value::Float(-3.0));
}

#[test]
fn test_math_sign_positive() {
    let mut rt = TailsRuntime::default();
    let result = rt.eval(r#"Math.sign(42);"#).unwrap();
    assert_eq!(result, tails::Value::Float(1.0));
}

#[test]
fn test_math_sign_negative() {
    let mut rt = TailsRuntime::default();
    let result = rt.eval(r#"Math.sign(-42);"#).unwrap();
    assert_eq!(result, tails::Value::Float(-1.0));
}

#[test]
fn test_math_sign_zero() {
    let mut rt = TailsRuntime::default();
    let result = rt.eval(r#"Math.sign(0);"#).unwrap();
    assert_eq!(result, tails::Value::Float(0.0));
}

// ============================================================================
// Combined: Proxy + class with private fields + String()
// ============================================================================

#[test]
fn test_proxy_class_with_string() {
    let mut rt = TailsRuntime::default();
    let result = rt
        .eval(
            r#"
        class Vec {
            #x;
            #y;
            constructor(x, y) {
                this.#x = x;
                this.#y = y;
            }
            get length() {
                return Math.hypot(this.#x, this.#y);
            }
        }
        const audited = [];
        const v = new Proxy(new Vec(3, 4), {
            get(t, k) {
                audited.push(String(k));
                return Reflect.get(t, k);
            },
        });
        v.length;
    "#,
        )
        .unwrap();
    assert_eq!(result, tails::Value::Float(5.0));
}
