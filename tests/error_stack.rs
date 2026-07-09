use tails::TailsRuntime;

#[test]
fn test_error_stack_has_function_names() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        function inner() {
            throw new Error("test");
        }
        function middle() {
            inner();
        }
        let stack;
        try {
            middle();
        } catch(e) {
            stack = e.stack;
        }
        stack;
    "#,
    );
    assert!(r.is_ok());
    if let tails::Value::String(stack) = r.unwrap() {
        assert!(
            stack.contains("Error: test"),
            "Stack should contain error message: {}",
            stack
        );
        assert!(
            stack.contains("inner"),
            "Stack should contain 'inner': {}",
            stack
        );
        assert!(
            stack.contains("middle"),
            "Stack should contain 'middle': {}",
            stack
        );
    } else {
        panic!("Expected string for stack trace");
    }
}

#[test]
fn test_type_error_stack() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        function doWork() {
            throw new TypeError("bad type");
        }
        let result;
        try {
            doWork();
        } catch(e) {
            result = e.name + ": " + e.message;
        }
        result;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(
        r.unwrap(),
        tails::Value::string("TypeError: bad type")
    );
}

#[test]
fn test_error_stack_has_at() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        function foo() {
            throw new Error("err");
        }
        let stack;
        try {
            foo();
        } catch(e) {
            stack = e.stack;
        }
        stack;
    "#,
    );
    assert!(r.is_ok());
    if let tails::Value::String(stack) = r.unwrap() {
        assert!(
            stack.contains("at foo"),
            "Stack should contain 'at foo': {}",
            stack
        );
    } else {
        panic!("Expected string for stack trace");
    }
}

#[test]
fn test_error_stack_has_line_numbers() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        function outer() {
            function inner() {
                throw new Error("line test");
            }
            inner();
        }
        let stack;
        try {
            outer();
        } catch(e) {
            stack = e.stack;
        }
        stack;
    "#,
    );
    assert!(r.is_ok());
    if let tails::Value::String(stack) = r.unwrap() {
        assert!(
            stack.contains("inner"),
            "Stack should contain 'inner': {}",
            stack
        );
        assert!(
            stack.contains("outer"),
            "Stack should contain 'outer': {}",
            stack
        );
    } else {
        panic!("Expected string for stack trace");
    }
}

#[test]
fn test_error_stack_at_format() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        function myFunc() {
            throw new Error("format test");
        }
        let stack;
        try {
            myFunc();
        } catch(e) {
            stack = e.stack;
        }
        stack;
    "#,
    );
    assert!(r.is_ok());
    if let tails::Value::String(stack) = r.unwrap() {
        assert!(
            stack.contains("at myFunc"),
            "Stack should contain 'at myFunc': {}",
            stack
        );
    } else {
        panic!("Expected string for stack trace");
    }
}

#[test]
fn test_error_without_message() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let e = new Error();
        e.name;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::string("Error"));
}

#[test]
fn test_error_message_property() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let e = new Error("custom message");
        e.message;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(
        r.unwrap(),
        tails::Value::string("custom message")
    );
}
