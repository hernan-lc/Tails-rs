use tails::{TailsRuntime, Value};

// ---- BigInt ----
#[test]
fn test_bigint_literal() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"42n;"#).unwrap();
    assert_eq!(r, Value::BigInt(42));
}

#[test]
fn test_bigint_typeof() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"typeof 100n;"#).unwrap();
    assert_eq!(r, Value::String("bigint".to_string()));
}

#[test]
fn test_bigint_addition() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"10n + 20n;"#).unwrap();
    assert_eq!(r, Value::BigInt(30));
}

#[test]
fn test_bigint_subtraction() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"50n - 20n;"#).unwrap();
    assert_eq!(r, Value::BigInt(30));
}

#[test]
fn test_bigint_multiplication() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"6n * 7n;"#).unwrap();
    assert_eq!(r, Value::BigInt(42));
}

#[test]
fn test_bigint_division() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"100n / 4n;"#).unwrap();
    assert_eq!(r, Value::BigInt(25));
}

#[test]
fn test_bigint_modulo() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"17n % 5n;"#).unwrap();
    assert_eq!(r, Value::BigInt(2));
}

#[test]
fn test_bigint_power() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"2n ** 10n;"#).unwrap();
    assert_eq!(r, Value::BigInt(1024));
}

#[test]
fn test_bigint_negate() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"-42n;"#).unwrap();
    assert_eq!(r, Value::BigInt(-42));
}

#[test]
fn test_bigint_comparison() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"10n < 20n;"#).unwrap();
    assert_eq!(r, Value::Boolean(true));
}

#[test]
fn test_bigint_equality() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"42n === 42n;"#).unwrap();
    assert_eq!(r, Value::Boolean(true));
}

#[test]
fn test_bigint_constructor() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"BigInt(123);"#).unwrap();
    assert_eq!(r, Value::BigInt(123));
}

#[test]
fn test_bigint_from_string() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"BigInt("456");"#).unwrap();
    assert_eq!(r, Value::BigInt(456));
}

// ---- Date ----
#[test]
fn test_date_constructor() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"let d = new Date(0); d.getTime();"#).unwrap();
    assert_eq!(r, Value::Float(0.0));
}

#[test]
fn test_date_from_millis() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"let d = new Date(1000); d.getTime();"#).unwrap();
    assert_eq!(r, Value::Float(1000.0));
}

#[test]
fn test_date_from_components() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(r#"let d = new Date(2024, 0, 15, 12, 30, 45); d.getFullYear();"#)
        .unwrap();
    assert_eq!(r, Value::Float(2024.0));
}

#[test]
fn test_date_get_month() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(r#"let d = new Date(2024, 5, 15); d.getMonth();"#)
        .unwrap();
    assert_eq!(r, Value::Float(5.0));
}

#[test]
fn test_date_get_date() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(r#"let d = new Date(2024, 0, 15); d.getDate();"#)
        .unwrap();
    assert_eq!(r, Value::Float(15.0));
}

#[test]
fn test_date_to_iso_string() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"let d = new Date(0); d.toISOString();"#).unwrap();
    assert_eq!(r, Value::String("1970-01-01T00:00:00.000Z".to_string()));
}

#[test]
fn test_date_value_of() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"let d = new Date(12345); d.valueOf();"#).unwrap();
    assert_eq!(r, Value::Float(12345.0));
}

#[test]
fn test_date_now_static() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"typeof Date.now();"#).unwrap();
    assert_eq!(r, Value::String("number".to_string()));
}

#[test]
fn test_date_parse_iso() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(r#"Date.parse("2024-01-15T00:00:00.000Z");"#)
        .unwrap();
    // Should be some number > 0
    match r {
        Value::Float(f) => assert!(f > 0.0),
        _ => panic!("Expected Float"),
    }
}

// ---- RegExp ----
#[test]
fn test_regexp_constructor() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(r#"let re = new RegExp("hello"); re.test("hello world");"#)
        .unwrap();
    assert_eq!(r, Value::Boolean(true));
}

#[test]
fn test_regexp_test_false() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(r#"let re = new RegExp("xyz"); re.test("hello world");"#)
        .unwrap();
    assert_eq!(r, Value::Boolean(false));
}

#[test]
fn test_regexp_with_flags() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(r#"let re = new RegExp("hello", "i"); re.test("HELLO world");"#)
        .unwrap();
    assert_eq!(r, Value::Boolean(true));
}

#[test]
fn test_regexp_to_string() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(r#"let re = new RegExp("abc", "gi"); re.toString();"#)
        .unwrap();
    assert_eq!(r, Value::String("/abc/gi".to_string()));
}

#[test]
fn test_regexp_exec_with_capture() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(r#"let re = new RegExp("(\\d+)-(\\d+)-(\\d+)"); re.exec("2024-01-15");"#)
        .unwrap();
    match r {
        Value::Array(idx) => {
            let arr = rt.get_array_element(&Value::Array(idx), 0);
            assert_eq!(arr, Some(Value::String("2024-01-15".to_string())));
            let g1 = rt.get_array_element(&Value::Array(idx), 1);
            assert_eq!(g1, Some(Value::String("2024".to_string())));
            let g2 = rt.get_array_element(&Value::Array(idx), 2);
            assert_eq!(g2, Some(Value::String("01".to_string())));
            let g3 = rt.get_array_element(&Value::Array(idx), 3);
            assert_eq!(g3, Some(Value::String("15".to_string())));
        }
        _ => panic!("Expected Array"),
    }
}

#[test]
fn test_regexp_source() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(r#"let re = new RegExp("hello", "gi"); re.source();"#)
        .unwrap();
    assert_eq!(r, Value::String("hello".to_string()));
}

#[test]
fn test_regexp_flags() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(r#"let re = new RegExp("abc", "gim"); re.flags();"#)
        .unwrap();
    assert_eq!(r, Value::String("gim".to_string()));
}

#[test]
fn test_regexp_global() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(r#"let re = new RegExp("a", "g"); re.global();"#)
        .unwrap();
    assert_eq!(r, Value::Boolean(true));
    let r = rt
        .eval(r#"let re = new RegExp("a"); re.global();"#)
        .unwrap();
    assert_eq!(r, Value::Boolean(false));
}

#[test]
fn test_regexp_ignore_case() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(r#"let re = new RegExp("a", "i"); re.ignoreCase();"#)
        .unwrap();
    assert_eq!(r, Value::Boolean(true));
}

#[test]
fn test_regexp_last_index() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(r#"let re = new RegExp("a", "g"); re.lastIndex();"#)
        .unwrap();
    assert_eq!(r, Value::Float(0.0));
}

#[test]
fn test_regexp_constructor_from_regexp() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(r#"let re1 = new RegExp("abc"); let re2 = new RegExp(re1); re2.source();"#)
        .unwrap();
    assert_eq!(r, Value::String("abc".to_string()));
}

// ---- Phase 3.4 — RegExp Lazy Result Cache ----

#[test]
fn test_regexp_lazy_cache_basic() {
    let mut rt = TailsRuntime::default();
    // Create a non-global regexp and test it multiple times on the same input
    let r = rt
        .eval(r#"
        let re = new RegExp("hello");
        let result1 = re.test("hello world");
        let result2 = re.test("hello world");
        let result3 = re.test("goodbye");
        result1 && result2 && !result3;
    "#)
        .unwrap();
    assert_eq!(r, Value::Boolean(true));
}

#[test]
fn test_regexp_lazy_cache_false_positive() {
    let mut rt = TailsRuntime::default();
    // Ensure cache correctly handles false results
    let r = rt
        .eval(r#"
        let re = new RegExp("xyz");
        let result1 = re.test("hello world");
        let result2 = re.test("hello world");
        !result1 && !result2;
    "#)
        .unwrap();
    assert_eq!(r, Value::Boolean(true));
}

#[test]
fn test_regexp_no_cache_for_global() {
    let mut rt = TailsRuntime::default();
    // Global regexp should not use cache (has stateful lastIndex)
    let r = rt
        .eval(r#"
        let re = new RegExp("a", "g");
        let result1 = re.test("aaa");
        let result2 = re.test("aaa");
        result1 && result2;
    "#)
        .unwrap();
    assert_eq!(r, Value::Boolean(true));
}

#[test]
fn test_regexp_no_cache_for_sticky() {
    let mut rt = TailsRuntime::default();
    // Sticky regexp should not use cache (has stateful lastIndex)
    let r = rt
        .eval(r#"
        let re = new RegExp("a", "y");
        let result1 = re.test("aaa");
        let result2 = re.test("aaa");
        result1 && result2;
    "#)
        .unwrap();
    assert_eq!(r, Value::Boolean(true));
}

// ---- Iterator Helpers ----
#[test]
fn test_symbol_iterator_on_array() {
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
fn test_array_iterator_method() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(
            r#"
        let arr = [1, 2, 3];
        let iter = arr[Symbol.iterator];
        typeof iter;
    "#,
        )
        .unwrap();
    assert_eq!(r, Value::String("function".to_string()));
}

#[test]
fn test_array_iterator_call() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(
            r#"
        let arr = [1, 2, 3];
        let iter = arr[Symbol.iterator]();
        typeof iter;
    "#,
        )
        .unwrap();
    assert_eq!(r, Value::String("object".to_string()));
}

#[test]
fn test_iterator_has_to_array() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(
            r#"
        let arr = [1, 2, 3];
        let iter = arr[Symbol.iterator]();
        typeof iter.toArray;
    "#,
        )
        .unwrap();
    assert_eq!(r, Value::String("function".to_string()));
}

#[test]
fn test_iterator_to_array_basic() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(
            r#"
        let arr = [1, 2, 3];
        let iter = arr[Symbol.iterator]();
        let result = iter.toArray();
        result.length;
    "#,
        )
        .unwrap();
    assert_eq!(r, Value::Float(3.0));
}

#[test]
fn test_iterator_to_array() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(
            r#"
        let arr = [1, 2, 3];
        let iter = arr[Symbol.iterator]();
        let result = iter.toArray();
        result.length;
    "#,
        )
        .unwrap();
    assert_eq!(r, Value::Float(3.0));
}

#[test]
fn test_iterator_map() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(
            r#"
        let arr = [1, 2, 3];
        let iter = arr[Symbol.iterator]();
        let mapped = iter.map(function(x) { return x * 2; });
        let result = mapped.toArray();
        result.length;
    "#,
        )
        .unwrap();
    assert_eq!(r, Value::Float(3.0));
}

#[test]
fn test_iterator_filter() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(
            r#"
        let arr = [1, 2, 3, 4, 5];
        let iter = arr[Symbol.iterator]();
        let filtered = iter.filter(function(x) { return x > 2; });
        let result = filtered.toArray();
        result.length;
    "#,
        )
        .unwrap();
    assert_eq!(r, Value::Float(3.0));
}

#[test]
fn test_iterator_take() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(
            r#"
        let arr = [1, 2, 3, 4, 5];
        let iter = arr[Symbol.iterator]();
        let taken = iter.take(3);
        let result = taken.toArray();
        result.length;
    "#,
        )
        .unwrap();
    assert_eq!(r, Value::Float(3.0));
}

#[test]
fn test_iterator_drop() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(
            r#"
        let arr = [1, 2, 3, 4, 5];
        let iter = arr[Symbol.iterator]();
        let dropped = iter.drop(2);
        let result = dropped.toArray();
        result.length;
    "#,
        )
        .unwrap();
    assert_eq!(r, Value::Float(3.0));
}

#[test]
fn test_iterator_for_each() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(
            r#"
        let sum = 0;
        let arr = [1, 2, 3];
        let iter = arr[Symbol.iterator]();
        iter.forEach(function(x) { sum = sum + x; });
        sum;
    "#,
        )
        .unwrap();
    assert_eq!(r, Value::Integer(6));
}

// ---- for await...of ----
#[test]
fn test_for_await_of_simple() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(
            r#"
        let sum = 0;
        let arr = [1, 2, 3];
        for (let val of arr) {
            sum = sum + val;
        }
        sum;
    "#,
        )
        .unwrap();
    assert_eq!(r, Value::Integer(6));
}

#[test]
fn test_for_await_of_with_promises() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(
            r#"
        let results = [];
        let arr = [Promise.resolve(1)];
        for await (let val of arr) {
            results.push(val);
        }
        results.length;
    "#,
        )
        .unwrap();
    assert_eq!(r, Value::Float(1.0));
}

#[test]
fn test_for_await_of_with_resolved_promises() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(
            r#"
        let results = [];
        let arr = [Promise.resolve(1), Promise.resolve(2), Promise.resolve(3)];
        for await (let val of arr) {
            results.push(val);
        }
        results.length;
    "#,
        )
        .unwrap();
    assert_eq!(r, Value::Float(3.0));
}

// ---- Phase 1.3 (BigInt / Symbol inlined in `LoadConst`) ----
//
// Before Phase 1.3, every `100n` and `Symbol("x")` literal in a hot loop
// went through the cascading `exec_load_store` dispatch (one extra branch
// per constant reference). The inline arm in
// `Interpreter::execute_from` now matches them on the same arm as Integer
// / Float / String / Boolean, so a `const BIG = 100n` reference is one
// discriminant match + one discriminant+payload clone.

#[test]
fn test_loadconst_bigint_in_hot_loop() {
    // If the inline path regressed to `exec_load_store` (which is the
    // cold path that *does* push), this would still pass — but the
    // value-equality assertions across many iterations catch any
    // accidental double-push or underflow.
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(
            r#"
            let acc = 0n;
            for (let i = 0; i < 1000; i++) {
                acc = acc + 100n;
            }
            acc;
        "#,
        )
        .unwrap();
    assert_eq!(r, Value::BigInt(100_000));
}

#[test]
fn test_loadconst_bigint_in_switch() {
    // Switch dispatch uses a `LoadConst` for the discriminant, so a
    // regression in the BigInt arm would show up as a fall-through to
    // the cold `exec_load_store` path (and Stack underflow if the push
    // got skipped — see the comment in `mod.rs:472`).
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(
            r#"
            let pick = function (n) {
                switch (n) {
                    case 0n: return "zero";
                    case 1n: return "one";
                    case 2n: return "two";
                    case 42n: return "answer";
                    default: return "other";
                }
            };
            pick(42n);
        "#,
        )
        .unwrap();
    assert_eq!(r, Value::String("answer".to_string()));
}

#[test]
fn test_loadconst_symbol_equality() {
    // `Symbol("x")` is a unique immediate value — the inline arm must
    // preserve the symbol identity (not just push a default Symbol).
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(
            r#"
            let s = Symbol("x");
            s === s;
        "#,
        )
        .unwrap();
    assert_eq!(r, Value::Boolean(true));
}

// ---- Phase 2.3 (String + Integer / Float inlined in `add`) ----
//
// Before Phase 2.3, `"answer: " + 42` called the general
// `to_string_coerce` path which allocated a temporary `String` for the
// number, then concatenated. The new inlined arms do a single
// `String::with_capacity` + two `push_str` calls — fewer allocations,
// same observable output.

#[test]
fn test_add_string_plus_integer() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#""answer: " + 42;"#).unwrap();
    assert_eq!(r, Value::String("answer: 42".to_string()));
}

#[test]
fn test_add_integer_plus_string() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"42 + " is the answer";"#).unwrap();
    assert_eq!(r, Value::String("42 is the answer".to_string()));
}

#[test]
fn test_add_string_plus_float() {
    // Match `to_string_coerce`'s finite-integer special case: "5" not "5.0".
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#""x=" + 5.0;"#).unwrap();
    assert_eq!(r, Value::String("x=5".to_string()));
}

#[test]
fn test_add_float_plus_string() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"3.14 + " pi";"#).unwrap();
    assert_eq!(r, Value::String("3.14 pi".to_string()));
}

#[test]
fn test_add_string_plus_negative_integer() {
    // Negative integers: the inline `i64::to_string` produces "-5".
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#""temp: " + -5;"#).unwrap();
    assert_eq!(r, Value::String("temp: -5".to_string()));
}

#[test]
fn test_add_string_plus_float_no_integer_form() {
    // Non-integer-valued floats keep their decimal form.
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#""x=" + 0.5;"#).unwrap();
    assert_eq!(r, Value::String("x=0.5".to_string()));
}

// ---- Phase 1.5 (`Vec::with_capacity` for `collect_garbage` snapshot) ----
//
// The behavioural surface is the same: a Map/Set/Array on the stack
// must survive a GC cycle that fires while the snapshot is being
// built. This is regression coverage for the size-hint pre-allocation
// (which is otherwise invisible at the API level).

#[test]
fn test_gc_snapshot_capacity_does_not_drop_references() {
    // Force a GC by allocating many objects on the stack.
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(
            r#"
            let arr = [];
            for (let i = 0; i < 500; i++) {
                arr.push({ value: i, next: arr });
            }
            arr[499].value;
        "#,
        )
        .unwrap();
    assert_eq!(r, Value::Integer(499));
}

// ---- Phase 3.5 — Inline property storage for small objects ----

#[test]
fn test_inline_property_basic() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(r#"
        let obj = { x: 1, y: 2, z: 3 };
        obj.x + obj.y + obj.z;
    "#)
        .unwrap();
    assert_eq!(r, Value::Integer(6));
}

#[test]
fn test_inline_property_many_properties() {
    // Test that objects with >8 properties fall back to hashmap correctly
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(r#"
        let obj = { a: 1, b: 2, c: 3, d: 4, e: 5, f: 6, g: 7, h: 8, i: 9 };
        obj.a + obj.i;
    "#)
        .unwrap();
    assert_eq!(r, Value::Integer(10));
}

#[test]
fn test_inline_property_update_existing() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(r#"
        let obj = { x: 1 };
        obj.x = 42;
        obj.x;
    "#)
        .unwrap();
    assert_eq!(r, Value::Integer(42));
}

#[test]
fn test_inline_property_with_prototype() {
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(r#"
        let proto = { inherited: 99 };
        let obj = {};
        Object.setPrototypeOf(obj, proto);
        obj.own = 1;
        obj.inherited + obj.own;
    "#)
        .unwrap();
    assert_eq!(r, Value::Integer(100));
}

#[test]
fn test_inline_property_gc_preserves_references() {
    // Ensure GC correctly traces inline properties
    let mut rt = TailsRuntime::default();
    let r = rt
        .eval(
            r#"
            let obj = { value: [1, 2, 3] };
            // Force GC by allocating
            for (let i = 0; i < 1000; i++) {
                [];
            }
            obj.value[0];
        "#,
        )
        .unwrap();
    assert_eq!(r, Value::Integer(1));
}
