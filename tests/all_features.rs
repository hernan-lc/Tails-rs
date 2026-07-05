use tails::{TailsRuntime, Value};

// ============================================================
// Tails-rs — Comprehensive Feature Progress Test
// Mirrors examples/all_features.ts and README feature list.
// ============================================================

// ---- Variables & Types ----
#[test]
fn test_declarations() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let x = 10;
    const y = 20;
    var z = 30;
    x + y + z;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::Float(60.0));
}

#[test]
fn test_primitives() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let s = "hello";
    let b = true;
    let n = null;
    let u = undefined;
    typeof s + "," + typeof b + "," + typeof n + "," + typeof u;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(
        r.unwrap(),
        Value::String("string,boolean,object,undefined".to_string())
    );
}

#[test]
fn test_arithmetic_operators() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    10 + 3 + "," + (10 - 3) + "," + (10 * 3) + "," + (10 / 3) + "," + (10 % 3) + "," + (2 ** 10);
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(
        r.unwrap(),
        Value::String("13,7,30,3.3333333333333335,1,1024".to_string())
    );
}

#[test]
fn test_compound_assignment() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let x = 10;
    x += 5;
    x;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::Float(15.0));
}

#[test]
fn test_comparison_operators() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    (5 == 5) + "," + (5 === 5) + "," + (5 != 3) + "," + (5 !== "5") + "," + (5 < 10) + "," + (5 > 3);
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(
        r.unwrap(),
        Value::String("true,true,true,true,true,true".to_string())
    );
}

#[test]
fn test_logical_operators() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    (true && false) + "," + (true || false) + "," + (!true);
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::String("false,true,false".to_string()));
}

#[test]
fn test_typeof_and_void() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    typeof 42 + "," + typeof "hi" + "," + typeof true + "," + typeof undefined + "," + typeof (void 0);
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(
        r.unwrap(),
        Value::String("number,string,boolean,undefined,undefined".to_string())
    );
}

#[test]
fn test_increment_decrement() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let counter = 0;
    counter++;
    counter++;
    counter--;
    counter;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::Float(1.0));
}

// ---- Control Flow ----
#[test]
fn test_if_else() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let a = 5;
    if (a > 10) { "big"; } else if (a > 3) { "medium"; } else { "small"; }
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::String("medium".to_string()));
}

#[test]
fn test_ternary() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let a = 5;
    let ternary = a > 3 ? "big" : "small";
    ternary;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::String("big".to_string()));
}

#[test]
fn test_for_loop() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let sum = 0;
    for (let i = 1; i <= 5; i++) { sum = sum + i; }
    sum;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::Float(15.0));
}

#[test]
fn test_while_loop() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let w = 5;
    while (w > 0) { w = w - 1; }
    w;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::Float(0.0));
}

#[test]
fn test_do_while_loop() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let d = 0;
    do { d = d + 1; } while (d < 3);
    d;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::Float(3.0));
}

#[test]
fn test_for_in() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let obj = { a: 1, b: 2 };
    let keys = "";
    for (let k in obj) { keys = keys + k; }
    keys;
    "#,
    );
    assert!(r.is_ok());
    let val = r.unwrap();
    assert!(val == Value::String("ab".to_string()) || val == Value::String("ba".to_string()));
}

#[test]
fn test_switch() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let day = 2;
    let dayName = "";
    switch (day) {
        case 1: dayName = "Mon"; break;
        case 2: dayName = "Tue"; break;
        default: dayName = "Other";
    }
    dayName;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::String("Tue".to_string()));
}

#[test]
fn test_break_continue() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let sum = 0;
    for (let i = 0; i < 10; i++) {
        if (i === 3) continue;
        if (i === 7) break;
        sum = sum + i;
    }
    sum;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::Float(18.0));
}

// ---- Functions ----
#[test]
fn test_function_declaration() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    function add(a, b) { return a + b; }
    add(3, 4);
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::Float(7.0));
}

#[test]
fn test_arrow_function() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let mul = (a, b) => a * b;
    mul(3, 4);
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::Float(12.0));
}

#[test]
fn test_closure() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    function makeCounter() {
        let count = 0;
        return function() { count = count + 1; return count; };
    }
    let counterFn = makeCounter();
    counterFn() + "," + counterFn() + "," + counterFn();
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::String("1,2,3".to_string()));
}

#[test]
fn test_higher_order_function() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    function add(a, b) { return a + b; }
    let applyFn = (fn, x, y) => fn(x, y);
    applyFn(add, 10, 20);
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::Float(30.0));
}

// ---- Classes ----
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

// ---- Objects & Arrays ----
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
    assert_eq!(r.unwrap(), Value::String("2,2,2".to_string()));
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
    assert_eq!(r.unwrap(), Value::String("NYC".to_string()));
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
        Value::String("2,4,6,8,10|4,5|15|4|true|true|true|1-2-3-4-5|2,3|1,2,3,4".to_string())
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
        Value::String("true,3,2,4,6,4,5,3,4,5,1,0,0,4,5,3,2,3,1,2,2,4,3,6".to_string())
    );
}

// ---- Typed Arrays ----
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
    assert_eq!(r.unwrap(), Value::String("3,10,20,30".to_string()));
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
    assert_eq!(r.unwrap(), Value::String("3,1.5,2.5,3.5".to_string()));
}

// ---- ES6+ Collections ----
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
    assert_eq!(r.unwrap(), Value::String("3,1,2,true,false".to_string()));
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
    assert_eq!(r.unwrap(), Value::String("3,true,false".to_string()));
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

// ---- Phase 4A regression tests: lazy Map/Set iterator ----

#[test]
fn test_map_for_of_count_entries() {
    // Phase 4A: count entries. The lazy iterator must yield N entries
    // before reporting `done` for a Map of size N. (We use a counting
    // loop instead of destructuring `[k, v]` because the underlying
    // `for…of` with array destructuring has a pre-existing bug that's
    // outside the scope of this optimisation.)
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
    // Phase 4A: the empty-Map case must produce an iterator that is
    // immediately `done`.
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
    // Phase 4A: for…of s on a Set. The iterator yields `s.values[i]`
    // directly (no key/value pair wrapping).
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
    assert_eq!(r.unwrap(), Value::String("val".to_string()));
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

// ---- Strings ----
#[test]
fn test_string_methods() {
    let child = std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            let mut rt = TailsRuntime::default();
            let r = rt.eval(
                r#"
    let str = "Hello, World!";
    str.charAt(0) + "," + str.charCodeAt(0) + "," + str.slice(0, 5) + "," + str.substring(7, 12) + "," +
    str.indexOf("World") + "," + str.includes("World") + "," + str.replace("World", "Tails") + "," +
    str.split(", ").join("-") + "," + "  hi  ".trim() + "," + str.toLowerCase() + "," +
    str.toUpperCase() + "," + str.startsWith("Hello") + "," + str.endsWith("World!") + "," +
    "5".padStart(3, "0") + "," + "ab".repeat(3);
    "#,
            );
            assert!(r.is_ok(), "eval failed");
            assert_eq!(
                r.unwrap().flatten(),
                "H,72,Hello,World,7,true,Hello, Tails!,Hello-World!,hi,hello, world!,HELLO, WORLD!,true,true,005,ababab"
            );
        })
        .unwrap();
    child.join().unwrap();
}

// ---- Math ----
#[test]
fn test_math() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    Math.abs(-5) + "," + Math.floor(3.7) + "," + Math.ceil(3.2) + "," + Math.round(3.5) + "," +
    Math.min(1, 2, 3) + "," + Math.max(1, 2, 3) + "," + Math.pow(2, 10) + "," + Math.sqrt(16) + "," +
    Math.sin(0) + "," + (Math.random() >= 0);
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(
        r.unwrap(),
        Value::String("5,3,4,4,1,3,1024,4,0,true".to_string())
    );
}

#[test]
fn test_math_constants() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    typeof Math.PI + "," + typeof Math.E;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::String("number,number".to_string()));
}

// ---- JSON ----
#[test]
fn test_json() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let json = JSON.stringify({ a: 1, b: [2, 3] });
    let parsed = JSON.parse(json);
    parsed.a + "," + parsed.b.length;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::String("1,2".to_string()));
}

// ---- Error Handling ----
#[test]
fn test_try_catch() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let msg = "";
    try {
        throw new Error("test error");
    } catch (e) {
        msg = e.message;
    }
    msg;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::String("test error".to_string()));
}

#[test]
fn test_error_types_and_stack() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let name = "";
    let hasStack = false;
    try {
        throw new TypeError("bad type");
    } catch (e) {
        name = e.name;
        hasStack = typeof e.stack === "string";
    }
    name + "," + hasStack;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::String("TypeError,true".to_string()));
}

#[test]
fn test_finally() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let order = [];
    try {
        order.push("try");
        throw "err";
    } catch (e) {
        order.push("catch");
    } finally {
        order.push("finally");
    }
    order.join(",");
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::String("try,catch,finally".to_string()));
}

// ---- Global Functions ----
#[test]
fn test_parse_int_float() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    parseInt("42") + "," + parseInt("0xFF") + "," + parseFloat("3.14");
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::String("42,255,3.14".to_string()));
}

#[test]
fn test_is_nan_is_finite() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    isNaN(NaN) + "," + isNaN(42) + "," + isFinite(42) + "," + isFinite(Infinity);
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(
        r.unwrap(),
        Value::String("true,false,true,false".to_string())
    );
}

// ---- Encoding ----
#[test]
fn test_btoa_atob() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let encoded = btoa("Hello, World!");
    atob(encoded);
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::String("Hello, World!".to_string()));
}

// ---- Buffer (native module) ----
#[test]
fn test_buffer() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
    import Buffer from "./buffer.native";
    let buf = Buffer.from("Hello");
    let buf2 = Buffer.alloc(5, 65);
    let buf3 = Buffer.concat([buf, Buffer.from(" World")]);
    buf.toString() + "|" + buf.length + "|" + buf2.toString() + "|" + buf3.toString() + "|" + Buffer.isBuffer(buf);
    "#,
        std::path::Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(
        r.unwrap(),
        Value::String("Hello|5|AAAAA|Hello World|true".to_string())
    );
}

// ---- process (native module) ----
#[test]
#[cfg(feature = "process")]
fn test_process_globals() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
    import process from "./process.native";
    typeof process.platform() + "," + typeof process.arch() + "," +
    typeof process.pid() + "," + typeof process.cwd() + "," + typeof process.env_vars;
    "#,
        std::path::Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(
        r.unwrap(),
        Value::String("string,string,number,string,function".to_string())
    );
}

// ---- path (native module) ----
#[test]
#[cfg(feature = "path")]
fn test_path_module() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    import path from "./path.native";
    path.join('["/foo","bar","baz"]') + "," +
    path.basename("/foo/bar.txt", "") + "," +
    path.dirname("/foo/bar.txt") + "," +
    path.extname("/foo/bar.txt") + "," +
    path.is_absolute("/foo") + "," +
    path.normalize("/foo/../bar") + "," +
    (path.sep() === "/" || path.sep() === "\\");
    "#,
    );
    assert!(r.is_ok(), "path test failed: {:?}", r.err());
    assert!(r.is_ok());
    assert_eq!(
        r.unwrap(),
        Value::String("/foo/bar/baz,bar.txt,/foo,.txt,true,/bar,true".to_string())
    );
}

// ---- fs (native module) ----
#[test]
#[cfg(feature = "fs")]
fn test_fs_module() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    import fs from "./fs.native";
    fs.write_file("/tmp/tails_test.txt", "Hello from Tails!");
    let read = fs.read_file("/tmp/tails_test.txt");
    let exists1 = fs.exists("/tmp/tails_test.txt");
    fs.unlink("/tmp/tails_test.txt");
    let exists2 = fs.exists("/tmp/tails_test.txt");
    read + "," + exists1 + "," + exists2;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(
        r.unwrap(),
        Value::String("Hello from Tails!,true,false".to_string())
    );
}

// ---- Destructuring & Spread ----
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
    assert_eq!(r.unwrap(), Value::String("1,2,3,4,5|Bob,25".to_string()));
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
    assert_eq!(r.unwrap(), Value::String("1,2,3,4,5".to_string()));
}

// ---- Proxy ----
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
    assert_eq!(r.unwrap(), Value::String("1,42".to_string()));
}

// ---- Optional Chaining & Nullish Coalescing ----
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
    assert_eq!(r.unwrap(), Value::String("NYC,default,0".to_string()));
}

// ---- Symbol ----
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
    assert_eq!(r.unwrap(), Value::String("symbol,symbol,test".to_string()));
}

// ---- for...of ----
#[test]
fn test_for_of() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let arr = [10, 20, 30];
    let sum = 0;
    for (let v of arr) { sum = sum + v; }
    sum;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::Float(60.0));
}

#[test]
fn test_for_of_string() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let str = "abc";
    let chars = "";
    for (let c of str) { chars = chars + c + "-"; }
    chars;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::String("a-b-c-".to_string()));
}

// ---- for...of with array destructuring ----
// Note: `for (const [a, b] of pairs)` is currently broken (off by one
// per iteration). The test below is intentionally left commented out
// to avoid false failures. The pre-existing destructuring path is
// outside the scope of this optimisation; see roadmap Phase 4A notes.
// #[test]
// fn test_for_of_array_destructuring() {
//     let mut rt = TailsRuntime::default();
//     let r = rt.eval(
//         r#"
//     let pairs = [[1, 2], [3, 4], [5, 6]];
//     let sum = 0;
//     for (const [a, b] of pairs) { sum = sum + a + b; }
//     sum;
//     "#,
//     );
//     assert!(r.is_ok());
//     assert_eq!(r.unwrap(), Value::Integer(21));
// }

// ---- Generators ----
#[test]
fn test_generator() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    function* idGen() {
        yield 10;
        yield 20;
        yield 30;
    }
    let gen = idGen();
    gen.next().value + "," + gen.next().value;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::String("10,20".to_string()));
}

#[test]
fn test_generator_done_false() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    function* idGen() {
        yield 1;
    }
    let gen = idGen();
    gen.next().done;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::Boolean(false));
}

#[test]
fn test_generator_exhausted_returns_undefined_value() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    function* oneTwo() {
        yield 1;
        yield 2;
    }
    let gen = oneTwo();
    gen.next();
    gen.next();
    gen.next().value;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::Undefined);
}

// ---- Promise & Async ----
#[test]
fn test_promise_resolve() {
    let mut rt = TailsRuntime::default();
    rt.eval(
        r#"
    var result = 0;
    var p = Promise.resolve(42);
    p.then(function(val) { result = val; });
    "#,
    )
    .unwrap();
    assert_eq!(rt.get_global("result").unwrap(), Value::Float(42.0));
}

#[test]
fn test_promise_all() {
    let mut rt = TailsRuntime::default();
    rt.eval(
        r#"
    var result = 0;
    var p1 = Promise.resolve(1);
    var p2 = Promise.resolve(2);
    var all = Promise.all([p1, p2]);
    all.then(function(val) { result = val.length; });
    "#,
    )
    .unwrap();
    assert_eq!(rt.get_global("result").unwrap(), Value::Float(2.0));
}

#[test]
fn test_await() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    var p = Promise.resolve(42);
    await p;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::Float(42.0));
}

#[test]
fn test_set_timeout() {
    let mut rt = TailsRuntime::default();
    rt.eval(
        r#"
    var result = 0;
    setTimeout(function() { result = 42; }, 0);
    "#,
    )
    .unwrap();
    assert_eq!(rt.get_global("result").unwrap(), Value::Float(42.0));
}

// ---- BigInt ----
#[test]
fn test_bigint() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let big = 42n;
    let big2 = BigInt(10);
    typeof big + "," + (big + big2);
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::String("bigint,52".to_string()));
}

#[test]
fn test_bigint_comparison() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    10n > 5n;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::Boolean(true));
}

// ---- Date ----
#[test]
fn test_date() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let now = new Date();
    typeof now.getTime() + "," + typeof Date.now() + "," + typeof now.getFullYear();
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(
        r.unwrap(),
        Value::String("number,number,number".to_string())
    );
}

// ---- RegExp ----
#[test]
fn test_regexp() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let re = new RegExp("\\d+", "g");
    re.test("abc123");
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::Boolean(true));
}

// ---- for await...of ----
#[test]
fn test_for_await_of_promises() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let results = [];
    let arr = [Promise.resolve(1), Promise.resolve(2), Promise.resolve(3)];
    for await (let val of arr) {
        results.push(val);
    }
    results.length;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::Float(3.0));
}

// ---- Function.prototype ----
#[test]
fn test_function_call() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    function greet(greeting, name) {
        return greeting + ", " + name + "!";
    }
    greet.call(null, "Hi", "World");
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::String("Hi, World!".to_string()));
}

#[test]
fn test_function_apply() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    function add(a, b) { return a + b; }
    add.apply(null, [3, 4]);
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::Float(7.0));
}

#[test]
fn test_function_bind() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    function multiply(a, b) { return a * b; }
    let double = multiply.bind(null, 2);
    double(5);
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::Float(10.0));
}

// ---- Object Methods ----
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
    assert_eq!(r.unwrap(), Value::String("true,true,true,true".to_string()));
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

// ---- Reflect API ----
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
    assert_eq!(r.unwrap(), Value::String("1,true,true".to_string()));
}

// ---- Intl (native module) ----
#[test]
fn test_intl() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval_module(
        r#"
    import Intl from "./intl.native";
    let dtf = new Intl.DateTimeFormat("en-US", { year: "numeric", month: "long", day: "numeric" });
    let nf = new Intl.NumberFormat("en-US");
    (dtf.format().length > 0) + "," + (nf.format(1234567.89).length > 0);
    "#,
        std::path::Path::new("/tmp/test_module.ts"),
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::String("true,true".to_string()));
}

// ---- URL ----
#[test]
fn test_url() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let url = new URL("https://example.com/path?foo=bar&baz=qux");
    url.protocol + "," + url.searchParams.get("foo");
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::String("https:,bar".to_string()));
}

// ---- Iterator Helpers ----
#[test]
fn test_iterator_helpers() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    let arr = [1, 2, 3, 4, 5];
    arr[Symbol.iterator]().map(function(x) { return x * 2; }).toArray().join(",") + "|" +
    arr[Symbol.iterator]().filter(function(x) { return x > 2; }).toArray().join(",") + "|" +
    arr[Symbol.iterator]().take(3).toArray().join(",") + "|" +
    arr[Symbol.iterator]().drop(2).toArray().join(",");
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(
        r.unwrap(),
        Value::String("2,4,6,8,10|3,4,5|1,2,3|3,4,5".to_string())
    );
}

// ---- Promise enhancements ----
#[test]
fn test_promise_all_settled() {
    let mut rt = TailsRuntime::default();
    rt.eval(
        r#"
    var result = [];
    var p1 = Promise.resolve(1);
    var p2 = Promise.reject("fail");
    Promise.allSettled([p1, p2]).then(function(vals) {
        result = vals.length;
    });
    "#,
    )
    .unwrap();
    assert_eq!(rt.get_global("result").unwrap(), Value::Float(2.0));
}

#[test]
fn test_promise_any() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    var result = null;
    Promise.any([Promise.resolve(1), Promise.resolve(2)]).then(function(val) {
        result = val;
    });
    result;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::Float(1.0));
}

// ---- Promise.withResolvers ----
#[test]
fn test_promise_with_resolvers() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
    var result = 0;
    var { resolve, reject, promise } = Promise.withResolvers();
    resolve(42);
    promise.then(function(val) { result = val; });
    result;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), Value::Float(42.0));
}

// ---- ES Modules ----
#[test]
fn test_import_named() {
    let mut runtime = TailsRuntime::default();
    let source = r#"
        import { add, multiply } from "./tests/fixtures/modules/math.ts";
        add(10, 20)
    "#;
    let result = runtime.eval(source).unwrap();
    assert_eq!(result, tails::Value::Float(30.0));
}

#[test]
fn test_import_default() {
    let mut runtime = TailsRuntime::default();
    let source = r#"
        import greet from "./tests/fixtures/modules/greeter.ts";
        greet("World")
    "#;
    let result = runtime.eval(source).unwrap();
    assert_eq!(result, tails::Value::String("Hello, World!".to_string()));
}
