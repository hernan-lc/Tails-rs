//! Regression tests for the `for...of` local-clobbering bug.
//!
//! Root cause: a `for...of` loop whose iterator is created from a binding
//! declared *before* the loop would corrupt that binding when a discarded
//! bare expression statement (e.g. `doc;`) sat between the declaration and
//! the loop. The discarded `LoadLocal + Pop` was removed by the peephole
//! optimizer, shifting every following instruction earlier — but the
//! optimizer did not remap the `IteratorNext`/"done" jump target, so it
//! landed in the *middle* of the statement after the loop (the `console.log`
//! call in the repro), producing `TypeError: undefined is not a function`
//! (or a silently `undefined` value for the earlier binding).
//!
//! Fixed in `src/compiler/bytecode/mod.rs` (`peephole_optimize`): it now
//! remaps `IteratorNext` / `AsyncIteratorNext` / `TryJump` jump targets.

use tails::TailsRuntime;

/// The exact minimal reproduction from the bug report.
#[test]
fn for_of_keeps_binding_before_loop_with_discarded_expr() {
    let mut rt = TailsRuntime::default();
    let result = rt
        .eval(
            r#"
        function f() {
          const doc = {};
          const normalized = { keys: ["a", "b"] };
          doc;                                   // bare discarded expression statement
          const ids = {};
          let counter = 0;
          for (const key of normalized.keys) {
            ids[key] = `key_${counter++}`;
          }
          return typeof normalized;
        }
        f();
    "#,
        )
        .expect("eval failed");
    assert_eq!(result, tails::Value::string("object"));
}

/// The earlier binding must also still be *usable* (not merely typeof-able).
#[test]
fn for_of_binding_before_loop_still_usable() {
    let mut rt = TailsRuntime::default();
    let result = rt
        .eval(
            r#"
        function f() {
          const prefix = "p";
          const data = [1, 2, 3];
          prefix;                                // discarded expression statement
          let out = "";
          for (const v of data) {
            out = out + prefix + v + ",";
          }
          return out;
        }
        f();
    "#,
        )
        .expect("eval failed");
    assert_eq!(result, tails::Value::string("p1,p2,p3,"));
}

/// Removing any one trigger condition must still work (no discarded expr).
#[test]
fn for_of_without_discarded_expr_is_fine() {
    let mut rt = TailsRuntime::default();
    let result = rt
        .eval(
            r#"
        function f() {
          const normalized = { keys: ["a", "b"] };
          const ids = {};
          let counter = 0;
          for (const key of normalized.keys) {
            ids[key] = `key_${counter++}`;
          }
          return typeof normalized;
        }
        f();
    "#,
        )
        .expect("eval failed");
    assert_eq!(result, tails::Value::string("object"));
}

/// The iterable-binding must be referenced inside the loop's iterable expr.
#[test]
fn for_of_declared_and_used_in_iterable_with_discarded_expr() {
    let mut rt = TailsRuntime::default();
    let result = rt
        .eval(
            r#"
        function f() {
          const sentinel = { items: ["x", "y"] };
          sentinel;                              // discarded
          let acc = "";
          for (const it of sentinel.items) {
            acc = acc + it;
          }
          return sentinel.items.length;
        }
        f();
    "#,
        )
        .expect("eval failed");
    assert_eq!(result, tails::Value::Float(2.0));
}

/// The binding used inside the loop must survive the full loop and remain
/// callable afterwards (this is the Zod `generateFastpass` shape).
#[test]
fn for_of_binding_remains_callable_after_loop() {
    let mut rt = TailsRuntime::default();
    let result = rt
        .eval(
            r#"
        function f() {
          const doc = { write: (s) => s.length };
          const normalized = { keys: ["a", "b"] };
          doc;                                   // discarded expression statement
          let total = 0;
          for (const key of normalized.keys) {
            total = total + doc.write(key);
          }
          // `doc` must still be callable here. write("after") = 5, total = 2.
          return doc.write("after") + total;
        }
        f();
    "#,
        )
        .expect("eval failed");
    assert_eq!(result, tails::Value::Float(7.0));
}

/// `for await...of` exercises the same codegen path (AsyncIteratorNext).
#[test]
fn for_await_of_keeps_binding_before_loop_with_discarded_expr() {
    let mut rt = TailsRuntime::default();
    let result = rt
        .eval(
            r#"
        async function f() {
          const seq = ["a", "b"];
          const sink = [];
          seq;                                    // discarded expression statement
          for await (const v of seq) {
            sink.push(v);
          }
          return seq.length;
        }
        f();
    "#,
        )
        .expect("eval failed");
    assert_eq!(result, tails::Value::Float(2.0));
}
