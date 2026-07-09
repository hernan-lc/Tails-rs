use tails::TailsRuntime;

#[test]
fn test_basic_class_declaration() {
    let mut rt = TailsRuntime::default();
    let result = rt
        .eval(
            r#"
        class Foo {
            constructor(x) {
                this.x = x;
            }
        }
        var f = new Foo(42);
        f.x;
    "#,
        )
        .unwrap();
    assert_eq!(result, tails::Value::Float(42.0));
}

#[test]
fn test_class_instance_method() {
    let mut rt = TailsRuntime::default();
    let result = rt
        .eval(
            r#"
        class Foo {
            constructor(x) {
                this.x = x;
            }
            getX() {
                return this.x;
            }
        }
        var f = new Foo(10);
        f.getX();
    "#,
        )
        .unwrap();
    assert_eq!(result, tails::Value::Float(10.0));
}

#[test]
fn test_class_multiple_methods() {
    let mut rt = TailsRuntime::default();
    let result = rt
        .eval(
            r#"
        class Calc {
            constructor(a, b) {
                this.a = a;
                this.b = b;
            }
            add() {
                return this.a + this.b;
            }
            mul() {
                return this.a * this.b;
            }
        }
        var c = new Calc(3, 4);
        c.add() + c.mul();
    "#,
        )
        .unwrap();
    assert_eq!(result, tails::Value::Float(19.0));
}

#[test]
fn test_class_static_method() {
    let mut rt = TailsRuntime::default();
    let result = rt
        .eval(
            r#"
        class Foo {
            static create() {
                return 42;
            }
        }
        Foo.create();
    "#,
        )
        .unwrap();
    assert_eq!(result, tails::Value::Float(42.0));
}

#[test]
fn test_class_static_method_with_args() {
    let mut rt = TailsRuntime::default();
    let result = rt
        .eval(
            r#"
        class MathHelper {
            static add(a, b) {
                return a + b;
            }
        }
        MathHelper.add(3, 7);
    "#,
        )
        .unwrap();
    assert_eq!(result, tails::Value::Float(10.0));
}

#[test]
fn test_class_no_constructor() {
    let mut rt = TailsRuntime::default();
    let result = rt
        .eval(
            r#"
        class Foo {
            greet() {
                return "hello";
            }
        }
        var f = new Foo();
        f.greet();
    "#,
        )
        .unwrap();
    assert_eq!(result, tails::Value::string("hello"));
}

#[test]
fn test_class_extends_basic() {
    let mut rt = TailsRuntime::default();
    let result = rt
        .eval(
            r#"
        class Animal {
            constructor(name) {
                this.name = name;
            }
            speak() {
                return this.name + " speaks";
            }
        }
        class Dog extends Animal {
            constructor(name) {
                super(name);
            }
            fetch() {
                return this.name + " fetches";
            }
        }
        var d = new Dog("Rex");
        d.speak();
    "#,
        )
        .unwrap();
    assert_eq!(result, tails::Value::string("Rex speaks"));
}

#[test]
fn test_class_extends_instance_methods() {
    let mut rt = TailsRuntime::default();
    let result = rt
        .eval(
            r#"
        class Base {
            constructor(x) {
                this.x = x;
            }
            getX() {
                return this.x;
            }
        }
        class Child extends Base {
            constructor(x) {
                super(x);
            }
            doubleX() {
                return this.x * 2;
            }
        }
        var c = new Child(5);
        c.doubleX();
    "#,
        )
        .unwrap();
    assert_eq!(result, tails::Value::Float(10.0));
}

#[test]
fn test_class_getter() {
    let mut rt = TailsRuntime::default();
    let result = rt
        .eval(
            r#"
        class Person {
            constructor(name) {
                this._name = name;
            }
            get name() {
                return this._name;
            }
        }
        var p = new Person("Alice");
        p.name;
    "#,
        )
        .unwrap();
    assert_eq!(result, tails::Value::string("Alice"));
}

#[test]
fn test_class_setter() {
    let mut rt = TailsRuntime::default();
    let result = rt
        .eval(
            r#"
        class Person {
            constructor(name) {
                this._name = name;
            }
            get name() {
                return this._name;
            }
            set name(v) {
                this._name = v;
            }
        }
        var p = new Person("Alice");
        p.name = "Bob";
        p.name;
    "#,
        )
        .unwrap();
    assert_eq!(result, tails::Value::string("Bob"));
}

#[test]
fn test_class_expression() {
    let mut rt = TailsRuntime::default();
    let result = rt
        .eval(
            r#"
        var Foo = class {
            constructor(x) {
                this.x = x;
            }
        };
        var f = new Foo(99);
        f.x;
    "#,
        )
        .unwrap();
    assert_eq!(result, tails::Value::Float(99.0));
}

#[test]
fn test_class_expression_named() {
    let mut rt = TailsRuntime::default();
    let result = rt
        .eval(
            r#"
        var Foo = class MyClass {
            constructor(x) {
                this.x = x;
            }
        };
        var f = new Foo(5);
        f.x;
    "#,
        )
        .unwrap();
    assert_eq!(result, tails::Value::Float(5.0));
}

#[test]
fn test_instanceof_basic() {
    let mut rt = TailsRuntime::default();
    let result = rt
        .eval(
            r#"
        class Foo {}
        var f = new Foo();
        f instanceof Foo;
    "#,
        )
        .unwrap();
    assert_eq!(result, tails::Value::Boolean(true));
}

#[test]
fn test_instanceof_extends() {
    let mut rt = TailsRuntime::default();
    let result = rt
        .eval(
            r#"
        class Base {}
        class Child extends Base {}
        var c = new Child();
        c instanceof Child;
    "#,
        )
        .unwrap();
    assert_eq!(result, tails::Value::Boolean(true));
}

#[test]
fn test_instanceof_extends_parent() {
    let mut rt = TailsRuntime::default();
    let result = rt
        .eval(
            r#"
        class Base {}
        class Child extends Base {}
        var c = new Child();
        c instanceof Base;
    "#,
        )
        .unwrap();
    assert_eq!(result, tails::Value::Boolean(true));
}

#[test]
fn test_class_multiple_instances() {
    let mut rt = TailsRuntime::default();
    let result = rt
        .eval(
            r#"
        class Counter {
            constructor(start) {
                this.count = start;
            }
            inc() {
                this.count = this.count + 1;
                return this.count;
            }
        }
        var a = new Counter(0);
        var b = new Counter(10);
        a.inc();
        a.inc();
        a.inc();
        b.inc();
        a.count + b.count;
    "#,
        )
        .unwrap();
    assert_eq!(result, tails::Value::Float(14.0));
}
