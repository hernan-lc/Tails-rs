use tails::TailsRuntime;

/// Regression test for: try/catch with a catch clause inside a `for...of` loop body
/// used to corrupt the iterator because the scope-cleanup Pop instructions for the
/// catch parameter were emitted at/after `finally_pc`, making the normal-path Jump
/// (which targets `finally_pc`) execute them. This consumed the for-of iterator from
/// the enclosing scope, causing `IteratorNext` to fail with "reading 'next' from
/// undefined".
///
/// This bug was first observed when running Zod's `generateFastpass` function, which
/// uses `try { console.log(...) } catch(e) {}` inside `for (const key of keys)`.
#[test]
fn for_of_with_try_catch_in_body() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        const keys = ["age", "status", "role", "email", "active", "id", "tags", "name", "nickname"];
        const ids = Object.create(null);
        let counter = 0;
        for (const key of keys) {
            try { console.log("loop", key); } catch(e) {}
            ids[key] = "key_" + counter;
            counter++;
        }
        Object.keys(ids).length;
    "#,
    );
    assert!(
        r.is_ok(),
        "for-of with try/catch should not corrupt iterator: {:?}",
        r
    );
    assert_eq!(r.unwrap(), tails::Value::Integer(9));
}

/// Same pattern but with a non-empty catch body to ensure the catch path also works.
#[test]
fn for_of_with_try_catch_nonempty_catch() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        const data = [1, 2, 3, 4];
        const result = {};
        for (const x of data) {
            try {
                if (x === 3) throw new Error("skip");
                result[x] = "ok";
            } catch(e) {
                result[x] = "caught";
            }
        }
        result[1] + "," + result[3] + "," + result[4];
    "#,
    );
    assert!(
        r.is_ok(),
        "for-of with try/catch (non-empty catch) should work: {:?}",
        r
    );
    assert_eq!(r.unwrap(), tails::Value::string("ok,caught,ok"));
}

/// Ensure try/catch/finally inside for-of also works correctly.
#[test]
fn for_of_with_try_catch_finally() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        const arr = ["a", "b", "c"];
        const out = [];
        const log = [];
        for (const x of arr) {
            try {
                out.push(x);
            } catch(e) {
                out.push("err");
            } finally {
                log.push("f");
            }
        }
        out.join(",") + "|" + log.join(",");
    "#,
    );
    assert!(
        r.is_ok(),
        "for-of with try/catch/finally should work: {:?}",
        r
    );
    assert_eq!(r.unwrap(), tails::Value::string("a,b,c|f,f,f"));
}

/// Ensure the fix doesn't break regular try/catch outside of loops.
#[test]
fn try_catch_basic_still_works() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let result;
        try {
            throw new Error("boom");
        } catch(e) {
            result = e.message;
        }
        result;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::string("boom"));
}

/// Ensure try/catch/finally basic behavior still works.
#[test]
fn try_catch_finally_basic_still_works() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let result = "";
        try {
            result += "t";
        } catch(e) {
            result += "c";
        } finally {
            result += "f";
        }
        result;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::string("tf"));
}

/// Nested try/catch inside for-of.
#[test]
fn for_of_with_nested_try_catch() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        const arr = [1, 2, 3];
        let sum = 0;
        for (const x of arr) {
            try {
                try {
                    if (x === 2) throw new Error("nope");
                    sum += x;
                } catch(inner) {
                    sum += 100;
                }
            } catch(outer) {
                sum += 1000;
            }
        }
        sum;
    "#,
    );
    assert!(
        r.is_ok(),
        "nested try/catch inside for-of should work: {:?}",
        r
    );
    assert_eq!(r.unwrap(), tails::Value::Integer(104)); // 1 + 100 + 3
}
