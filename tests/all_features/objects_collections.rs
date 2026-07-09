use tails::{TailsRuntime, Value};

#[test]
fn test_object_methods() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let person = { name: "Alice", age: 30 };
    Object.keys(person).length + "," + Object.values(person).length + "," + Object.entries(person).length;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::string("2,2,2"));
}

#[test]
fn test_object_assign() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let person = { name: "Alice", age: 30 };
    Object.assign(person, { city: "NYC" });
    person.city;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::string("NYC"));
}

#[test]
fn test_object_get_own_property_descriptors_and_define_properties() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let src = { a: 1, b: 2 };
    let descs = Object.getOwnPropertyDescriptors(src);
    let out = Object.defineProperties({}, descs);
    out.a + "," + out.b + "," + (descs.a.value) + "," + (descs.b.enumerable);
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::string("1,2,1,true"));
}

#[test]
fn test_array_methods() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let arr = [1, 2, 3, 4, 5];
    arr.push(6);
    arr.pop();
    arr.map(function(x) { return x * 2; }).join(",") + "|" +
    arr.filter(function(x) { return x > 3; }).join(",") + "|" +
    arr.reduce(function(a, b) { return a + b; }, 0) + "|" +
    arr.find(function(x) { return x > 3; }) + "|" +
    arr.some(function(x) { return x > 4; }) + "|" +
    arr.every(function(x) { return x > 0; }) + "|" +
    arr.includes(3) + "|" +
    arr.join("-") + "|" +
    arr.slice(1, 3).join(",") + "|" +
    [[1, 2], [3, 4]].flat().join(",");
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(
        r.unwrap(),
        Value::string("2,4,6,8,10|4,5|15|4|true|true|true|1-2-3-4-5|2,3|1,2,3,4")
    );
}

#[test]
fn test_array_enhancements() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let arr = [1, 2, 3, 4, 5];
    Array.isArray(arr) + "," +
    Array.of(1, 2, 3).length + "," +
    Array.from([1, 2, 3], function(x) { return x * 2; }).join(",") + "," +
    [1, 2, 3, 4, 5].copyWithin(0, 3).join(",") + "," +
    [1, 2, 3, 4, 5].fill(0, 1, 3).join(",") + "," +
    [1, 2, 3, 4, 5].findLast(function(x) { return x < 4; }) + "," +
    [1, 2, 3, 4, 5].findLastIndex(function(x) { return x < 4; }) + "," +
    [1, 2, 3, 2, 1].lastIndexOf(2) + "," +
    [1, 2, 3].flatMap(function(x) { return [x, x * 2]; }).join(",");
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(
        r.unwrap(),
        Value::string("true,3,2,4,6,4,5,3,4,5,1,0,0,4,5,3,2,3,1,2,2,4,3,6")
    );
}

#[test]
fn test_typed_array_int32() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let ta = new Int32Array(3);
    ta.set(0, 10);
    ta.set(1, 20);
    ta.set(2, 30);
    ta.length + "," + ta.get(0) + "," + ta.get(1) + "," + ta.get(2);
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::string("3,10,20,30"));
}

#[test]
fn test_typed_array_float64() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let ta = new Float64Array([1.5, 2.5, 3.5]);
    ta.length + "," + ta.get(0) + "," + ta.get(1) + "," + ta.get(2);
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::string("3,1.5,2.5,3.5"));
}

#[test]
fn test_map() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let myMap = new Map();
    myMap.set("a", 1);
    myMap.set("b", 2);
    myMap.set("c", 3);
    myMap.size + "," + myMap.get("a") + "," + myMap.get("b") + "," + myMap.has("a") + "," + myMap.has("d");
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::string("3,1,2,true,false"));
}

#[test]
fn test_map_delete() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let myMap = new Map();
    myMap.set("a", 1);
    myMap.set("b", 2);
    myMap.set("c", 3);
    myMap.delete("c");
    myMap.size;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::Float(2.0));
}

#[test]
fn test_set() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let mySet = new Set();
    mySet.add(1);
    mySet.add(2);
    mySet.add(3);
    mySet.add(2);
    mySet.size + "," + mySet.has(1) + "," + mySet.has(4);
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::string("3,true,false"));
}

#[test]
fn test_set_delete() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let mySet = new Set();
    mySet.add(1);
    mySet.add(2);
    mySet.delete(1);
    mySet.size;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::Float(1.0));
}

#[test]
fn test_map_for_of_count_entries() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        const m = new Map();
        for (let i = 0; i < 100; i++) {
            m.set(i, i * 2);
        }
        let count = 0;
        for (const entry of m) {
            count = count + 1;
        }
        count
        "#,
    );
    assert_eq!(r.unwrap(), Value::Float(100.0));
}

#[test]
fn test_map_for_of_empty() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        const m = new Map();
        let count = 0;
        for (const entry of m) {
            count = count + 1;
        }
        count
        "#,
    );
    assert_eq!(r.unwrap(), Value::Float(0.0));
}

#[test]
fn test_set_for_of_sum_values() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        const s = new Set();
        s.add(10);
        s.add(20);
        s.add(30);
        let sum = 0;
        for (const v of s) {
            sum = sum + v;
        }
        sum
        "#,
    );
    assert_eq!(r.unwrap(), Value::Float(60.0));
}

#[test]
fn test_set_for_of_count_entries() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        const s = new Set();
        for (let i = 0; i < 50; i++) {
            s.add(i);
        }
        let count = 0;
        for (const v of s) {
            count = count + 1;
        }
        count
        "#,
    );
    assert_eq!(r.unwrap(), Value::Float(50.0));
}

#[test]
fn test_weakmap() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let wm = new WeakMap();
    let k = {};
    wm.set(k, "val");
    wm.get(k);
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::string("val"));
}

#[test]
fn test_weakset() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let ws = new WeakSet();
    let k = {};
    ws.add(k);
    ws.has(k);
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::Boolean(true));
}

#[test]
fn test_destructuring() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let [p, q, ...rest] = [1, 2, 3, 4, 5];
    p + "," + q + "," + rest.join(",") + "|" +
    (function() { var n2 = ""; var age = 0; var _obj = { name: "Bob", age: 25 }; n2 = _obj.name; age = _obj.age; return n2 + "," + ("" + age); })();
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::string("1,2,3,4,5|Bob,25"));
}

#[test]
fn test_spread() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let arr2 = [1, 2, 3];
    let arr3 = [...arr2, 4, 5];
    arr3.join(",");
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::string("1,2,3,4,5"));
}

#[test]
fn test_proxy() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let handler = {
        get: function(target, prop) {
            return prop in target ? target[prop] : 42;
        }
    };
    let proxy = new Proxy({ x: 1 }, handler);
    proxy.x + "," + proxy.missing;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::string("1,42"));
}

#[test]
fn test_optional_chaining() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let user = { address: { city: "NYC" } };
    user?.address?.city + "," + (null ?? "default") + "," + (0 ?? "default");
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::string("NYC,default,0"));
}

#[test]
fn test_symbol() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let sym = Symbol("test");
    typeof sym + "," + typeof Symbol.for("test") + "," + Symbol.keyFor(Symbol.for("test"));
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::string("symbol,symbol,test"));
}

#[test]
fn test_object_freeze_seal() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let frozen = { x: 1 };
    Object.freeze(frozen);
    let sealed = { y: 2 };
    Object.seal(sealed);
    Object.isFrozen(frozen) + "," + Object.isSealed(sealed) + "," + Object.is(1, 1) + "," + Object.is(NaN, NaN);
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::string("true,true,true,true"));
}

#[test]
fn test_object_prevent_extensions() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let obj = { x: 1 };
    Object.preventExtensions(obj);
    Object.isExtensible(obj);
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::Boolean(false));
}

#[test]
fn test_reflect() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let obj = { x: 1 };
    Reflect.get(obj, "x") + "," + Reflect.isExtensible(obj) + "," + Reflect.preventExtensions(obj);
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::string("1,true,true"));
}
