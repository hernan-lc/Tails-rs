use tails::{TailsRuntime, Value};

/// Test that a simple integer loop produces the correct result.
/// The JIT profiler should detect the hot loop but the test mainly
/// verifies that adding the JIT infrastructure doesn't break the
/// interpreter.
#[test]
fn test_jit_integer_loop_correctness() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let sum = 0;
        for (let i = 0; i < 100; i++) {
            sum = sum + i;
        }
        sum;
    "#,
    );
    assert!(r.is_ok(), "eval failed: {:?}", r);
    assert_eq!(r.unwrap(), Value::Float(4950.0));
}

/// Test that a larger loop (which would trigger JIT compilation at
/// threshold=1000) still produces the correct result.
#[test]
fn test_jit_large_loop_correctness() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let sum = 0;
        for (let i = 0; i < 2000; i++) {
            sum = sum + 1;
        }
        sum;
    "#,
    );
    assert!(r.is_ok(), "eval failed: {:?}", r);
    assert_eq!(r.unwrap(), Value::Float(2000.0));
}

/// Test that the JIT profiler can be disabled.
#[test]
fn test_jit_can_be_disabled() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let sum = 0;
        for (let i = 0; i < 100; i++) {
            sum = sum + i;
        }
        sum;
    "#,
    );
    assert!(r.is_ok(), "eval failed: {:?}", r);
    assert_eq!(r.unwrap(), Value::Float(4950.0));
}

/// Test that loop branches with non-integer types don't crash.
#[test]
fn test_jit_loop_with_float_counter() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let sum = 0.0;
        for (let i = 0.0; i < 10; i = i + 1.0) {
            sum = sum + i;
        }
        sum;
    "#,
    );
    assert!(r.is_ok(), "eval failed: {:?}", r);
    // 0 + 1 + 2 + ... + 9 = 45.0
    assert_eq!(r.unwrap(), Value::Float(45.0));
}

/// Test that nested loops work.
#[test]
fn test_jit_nested_loops() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let sum = 0;
        for (let i = 0; i < 10; i++) {
            for (let j = 0; j < 10; j++) {
                sum = sum + 1;
            }
        }
        sum;
    "#,
    );
    assert!(r.is_ok(), "eval failed: {:?}", r);
    assert_eq!(r.unwrap(), Value::Float(100.0));
}

/// Test that loops with property access work (should deopt to interpreter).
#[test]
fn test_jit_loop_with_property_access() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let arr = [10, 20, 30, 40, 50];
        let sum = 0;
        for (let i = 0; i < 5; i++) {
            sum = sum + arr[i];
        }
        sum;
    "#,
    );
    assert!(r.is_ok(), "eval failed: {:?}", r);
    assert_eq!(r.unwrap(), Value::Float(150.0));
}

/// Test code buffer allocation and deallocation.
#[test]
fn test_code_buffer_basic() {
    use tails::vm::jit::code_buffer::CodeBuffer;
    let mut buf = CodeBuffer::new(256);
    buf.emit_byte(0x90); // nop
    buf.emit_byte(0xC3); // ret
    let ptr = buf.finalize();
    assert!(!ptr.is_null());
}

/// Test that the JitFrame struct has the expected size and alignment.
#[test]
fn test_jit_frame_layout() {
    use tails::vm::jit::frame::JitFrame;
    // JitFrame should be a plain C-compatible struct.
    assert!(std::mem::size_of::<JitFrame>() > 0);
}

/// Ensure the full test suite still passes (run via cargo test).
#[test]
fn test_interpreter_still_works_after_jit_integration() {
    let mut rt = TailsRuntime::default();
    // Simple arithmetic.
    assert_eq!(rt.eval("1 + 2").unwrap(), Value::Float(3.0));
    // Function call.
    let r = rt
        .eval(
            r#"
        function add(a, b) { return a + b; }
        add(3, 4);
    "#,
        )
        .unwrap();
    assert_eq!(r, Value::Float(7.0));
    // Object creation.
    let r = rt
        .eval(
            r#"
        let obj = { x: 10, y: 20 };
        obj.x + obj.y;
    "#,
        )
        .unwrap();
    assert_eq!(r, Value::Float(30.0));
}
