use tails::TailsRuntime;

#[test]
fn test_proxy_basic_creation() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        const target = { x: 1, y: 2 };
        const handler = {};
        const proxy = new Proxy(target, handler);
        proxy.x;
    "#,
    );
    assert!(result.is_ok());
}

#[test]
fn test_proxy_get_trap() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        const target = { x: 1 };
        const handler = {
            get: function(target, prop, receiver) {
                if (prop === "x") {
                    return 42;
                }
                return target[prop];
            }
        };
        const proxy = new Proxy(target, handler);
        proxy.x;
    "#,
    );
    assert!(result.is_ok());
}

#[test]
fn test_proxy_set_trap() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        const target = { x: 1 };
        const handler = {
            set: function(target, prop, value, receiver) {
                target[prop] = value * 2;
                return true;
            }
        };
        const proxy = new Proxy(target, handler);
        proxy.x = 5;
        target.x;
    "#,
    );
    assert!(result.is_ok());
}

#[test]
fn test_proxy_has_trap() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        const target = { x: 1 };
        const handler = {
            has: function(target, prop) {
                if (prop === "hidden") {
                    return false;
                }
                return prop in target;
            }
        };
        const proxy = new Proxy(target, handler);
        const r1 = "x" in proxy;
        const r2 = "hidden" in proxy;
        const r3 = "z" in proxy;
        r1;
    "#,
    );
    assert!(result.is_ok());
}

#[test]
fn test_proxy_delete_property_trap() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        const target = { x: 1, y: 2 };
        const handler = {
            deleteProperty: function(target, prop) {
                if (prop === "y") {
                    return false;
                }
                delete target[prop];
                return true;
            }
        };
        const proxy = new Proxy(target, handler);
        delete proxy.x;
    "#,
    );
    assert!(result.is_ok());
}

#[test]
fn test_reflect_get() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        const target = { x: 10 };
        Reflect.get(target, "x");
    "#,
    );
    assert!(result.is_ok());
}

#[test]
fn test_reflect_set() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        const target = { x: 1 };
        Reflect.set(target, "x", 99);
        target.x;
    "#,
    );
    assert!(result.is_ok());
}

#[test]
fn test_reflect_has() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        const target = { x: 1 };
        const r1 = Reflect.has(target, "x");
        const r2 = Reflect.has(target, "y");
        r1;
    "#,
    );
    assert!(result.is_ok());
}

#[test]
fn test_reflect_delete_property() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        const target = { x: 1, y: 2 };
        const r1 = Reflect.deleteProperty(target, "x");
        const r2 = target.x;
        r1;
    "#,
    );
    assert!(result.is_ok());
}

#[test]
fn test_reflect_apply() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        function add(a, b) { return a + b; }
        Reflect.apply(add, undefined, [3, 4]);
    "#,
    );
    assert!(result.is_ok());
}

#[test]
fn test_reflect_own_keys() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        const target = { a: 1, b: 2 };
        const keys = Reflect.ownKeys(target);
        keys.length;
    "#,
    );
    assert!(result.is_ok());
}

#[test]
fn test_reflect_get_prototype_of() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        const proto = {};
        const target = { x: 1 };
        Reflect.setPrototypeOf(target, proto);
        const protoOf = Reflect.getPrototypeOf(target);
        protoOf === proto;
    "#,
    );
    assert!(result.is_ok());
}

#[test]
fn test_reflect_is_extensible() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        const target = { x: 1 };
        Reflect.isExtensible(target);
    "#,
    );
    assert!(result.is_ok());
}

#[test]
fn test_reflect_prevent_extensions() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        const target = { x: 1 };
        Reflect.preventExtensions(target);
    "#,
    );
    assert!(result.is_ok());
}

#[test]
fn test_reflect_get_own_property_descriptor() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        const target = { x: 1 };
        const desc = Reflect.getOwnPropertyDescriptor(target, "x");
        desc.value;
    "#,
    );
    assert!(result.is_ok());
}

#[test]
fn test_reflect_define_property() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        const target = {};
        Reflect.defineProperty(target, "x", { value: 42 });
        target.x;
    "#,
    );
    assert!(result.is_ok());
}

#[test]
fn test_reflect_set_prototype_of() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        const target = {};
        const proto = { greet: function() { return "hello"; } };
        Reflect.setPrototypeOf(target, proto);
        target.greet();
    "#,
    );
    assert!(result.is_ok());
}

#[test]
fn test_object_define_property() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        const target = {};
        Object.defineProperty(target, "x", { value: 42 });
        target.x;
    "#,
    );
    assert!(result.is_ok());
}

#[test]
fn test_object_get_own_property_descriptor() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        const target = { x: 10 };
        const desc = Object.getOwnPropertyDescriptor(target, "x");
        desc.value;
    "#,
    );
    assert!(result.is_ok());
}

#[test]
fn test_proxy_reflect_integration() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        const target = { x: 1 };
        const handler = {
            get: function(target, prop, receiver) {
                return Reflect.get(target, prop, receiver) + 100;
            }
        };
        const proxy = new Proxy(target, handler);
        proxy.x;
    "#,
    );
    assert!(result.is_ok());
}

#[test]
fn test_proxy_no_traps_passthrough() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        const target = { x: 1, y: 2 };
        const handler = {};
        const proxy = new Proxy(target, handler);
        const r1 = proxy.x;
        proxy.z = 3;
        const r2 = proxy.z;
        r1;
    "#,
    );
    assert!(result.is_ok());
}

#[test]
fn test_reflect_construct() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        function MyClass(name) {
            this.name = name;
        }
        const obj = Reflect.construct(MyClass, ["test"]);
        obj.name;
    "#,
    );
    assert!(result.is_ok());
}

#[test]
fn test_proxy_in_operator() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        const target = { x: 1 };
        const handler = {
            has: function(target, prop) {
                if (prop === "secret") return true;
                return prop in target;
            }
        };
        const proxy = new Proxy(target, handler);
        const r1 = "x" in proxy;
        const r2 = "secret" in proxy;
        const r3 = "y" in proxy;
        r1;
    "#,
    );
    assert!(result.is_ok());
}

#[test]
fn test_proxy_method_chaining() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        const target = { x: 1 };
        const handler = {
            get: function(target, prop, receiver) {
                if (prop === "x") {
                    return target[prop] + 10;
                }
                return target[prop];
            }
        };
        const proxy = new Proxy(target, handler);
        proxy.x + 1;
    "#,
    );
    assert!(result.is_ok());
}

#[test]
fn test_proxy_apply_trap() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        function add(a, b) { return a + b; }
        const handler = {
            apply: function(target, thisArg, args) {
                return Reflect.apply(target, thisArg, args) * 2;
            }
        };
        const proxy = new Proxy(add, handler);
        proxy(3, 4);
    "#,
    );
    assert!(result.is_ok());
}

#[test]
fn test_reflect_construct_with_new_target() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        function Base(name) {
            this.name = name;
        }
        function Derived(name) {
            return Reflect.construct(Base, [name], Derived);
        }
        const obj = new Derived("test");
        obj.name;
    "#,
    );
    assert!(result.is_ok());
}

#[test]
fn test_new_target_meta_property() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        function F() {
            this.kind = (new.target === F) ? "ctor" : "plain";
        }
        const a = new F();          // new.target === F
        const b = F.call({});       // new.target is undefined
        a.kind + "|" + (b === undefined);
    "#,
    );
    assert_eq!(
        result.unwrap(),
        tails::objects::Value::from_string("ctor|true".to_string())
    );
}

#[test]
fn test_proxy_handler_get_method() {
    let mut runtime = TailsRuntime::default();
    let result = runtime.eval(
        r#"
        const target = { x: 1 };
        const handler = {
            get: function(target, prop, receiver) {
                return Reflect.get(target, prop, receiver) + 100;
            }
        };
        const proxy = new Proxy(target, handler);
        proxy.x;
    "#,
    );
    assert!(result.is_ok());
}
