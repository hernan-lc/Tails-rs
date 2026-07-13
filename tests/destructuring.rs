use tails::TailsRuntime;

#[test]
fn test_array_destructuring_basic() {
    let mut runtime = TailsRuntime::default();
    let result = runtime
        .eval(
            r#"
        const [a, b, c] = [1, 2, 3];
        a + b + c;
    "#,
        )
        .unwrap();
    assert_eq!(result.to_string(), "6");
}

#[test]
fn test_array_destructuring_skip() {
    let mut runtime = TailsRuntime::default();
    let result = runtime
        .eval(
            r#"
        const [a, , c] = [1, 2, 3];
        a + c;
    "#,
        )
        .unwrap();
    assert_eq!(result.to_string(), "4");
}

#[test]
fn test_object_destructuring_basic() {
    let mut runtime = TailsRuntime::default();
    let result = runtime
        .eval(
            r#"
        const {x, y} = {x: 10, y: 20};
        x + y;
    "#,
        )
        .unwrap();
    assert_eq!(result.to_string(), "30");
}

#[test]
fn test_object_destructuring_renamed() {
    let mut runtime = TailsRuntime::default();
    let result = runtime
        .eval(
            r#"
        const {x: a, y: b} = {x: 10, y: 20};
        a + b;
    "#,
        )
        .unwrap();
    assert_eq!(result.to_string(), "30");
}

#[test]
fn test_array_spread() {
    let mut runtime = TailsRuntime::default();
    let result = runtime
        .eval(
            r#"
        const arr1 = [1, 2, 3];
        const arr2 = [...arr1, 4, 5];
        arr2.length;
    "#,
        )
        .unwrap();
    assert_eq!(result.to_string(), "5");
}

#[test]
fn test_object_spread() {
    let mut runtime = TailsRuntime::default();
    let result = runtime
        .eval(
            r#"
        const obj1 = {a: 1, b: 2};
        const obj2 = {...obj1, c: 3};
        obj2.a + obj2.b + obj2.c;
    "#,
        )
        .unwrap();
    assert_eq!(result.to_string(), "6");
}

#[test]
fn test_destructuring_with_default() {
    let mut runtime = TailsRuntime::default();
    let result = runtime
        .eval(
            r#"
        const {a = 10, b = 20} = {a: 5};
        a + b;
    "#,
        )
        .unwrap();
    assert_eq!(result.to_string(), "25");
}

#[test]
fn test_nested_destructuring() {
    let mut runtime = TailsRuntime::default();
    let result = runtime
        .eval(
            r#"
        const {a: {b, c}} = {a: {b: 1, c: 2}};
        b + c;
    "#,
        )
        .unwrap();
    assert_eq!(result.to_string(), "3");
}

#[test]
fn test_computed_property_destructuring() {
    let mut runtime = TailsRuntime::default();
    let result = runtime
        .eval(
            r#"
        const obj = { foo: 42, bar: 99 };
        const key = "foo";
        const { [key]: val } = obj;
        val;
    "#,
        )
        .unwrap();
    assert_eq!(result.to_string(), "42");
}

#[test]
fn test_computed_property_destructuring_with_rest() {
    let mut runtime = TailsRuntime::default();
    let result = runtime
        .eval(
            r#"
        const obj = { foo: 1, bar: 2, baz: 3 };
        const type = "foo";
        const { [type]: oneOfs, ...rest } = obj;
        oneOfs + rest.bar + rest.baz;
    "#,
        )
        .unwrap();
    assert_eq!(result.to_string(), "6");
}

#[test]
fn test_computed_property_destructuring_string_literal() {
    let mut runtime = TailsRuntime::default();
    let result = runtime
        .eval(
            r#"
        const obj = { abc: 100, xyz: 200 };
        const { ["abc"]: a, ["xyz"]: b } = obj;
        a + b;
    "#,
        )
        .unwrap();
    assert_eq!(result.to_string(), "300");
}
