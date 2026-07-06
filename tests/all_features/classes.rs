use tails::{TailsRuntime, Value};

#[test]
fn test_class_basic() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    class Animal {
        constructor(name) { this.name = name; }
        speak() { return this.name + " makes a noise"; }
        get label() { return "animal:" + this.name; }
        set label(v) { this.name = v; }
    }
    let a = new Animal("Rex");
    a.speak();
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::String("Rex makes a noise".to_string()));
}

#[test]
fn test_class_inheritance() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    class Animal {
        constructor(name) { this.name = name; }
        speak() { return this.name + " makes a noise"; }
    }
    class Dog extends Animal {
        speak() { return this.name + " barks"; }
    }
    let d = new Dog("Rex");
    d.speak();
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::String("Rex barks".to_string()));
}

#[test]
fn test_class_getter_setter() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    class Animal {
        constructor(name) { this.name = name; }
        get label() { return "animal:" + this.name; }
        set label(v) { this.name = v; }
    }
    let d = new Animal("Rex");
    let originalLabel = d.label;
    d.label = "Spot";
    originalLabel + "," + d.name;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::String("animal:Rex,Rex".to_string()));
}

#[test]
fn test_instanceof() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    class Animal {}
    class Dog extends Animal {}
    let d = new Dog("Rex");
    (d instanceof Animal) + "," + (d instanceof Dog);
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::String("true,true".to_string()));
}

#[test]
fn test_class_static_method() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    class MathHelper {
        static double(x) { return x * 2; }
    }
    MathHelper.double(5);
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::Float(10.0));
}
