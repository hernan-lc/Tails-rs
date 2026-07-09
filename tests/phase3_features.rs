use tails::TailsRuntime;

// ---- console.log ----
#[test]
fn test_console_log_basic() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"console.log("hello world");"#);
    assert!(r.is_ok());
}

#[test]
fn test_console_log_multiple_args() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"console.log("a", "b", "c");"#);
    assert!(r.is_ok());
}

#[test]
fn test_console_warn() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"console.warn("warning");"#);
    assert!(r.is_ok());
}

#[test]
fn test_console_error() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"console.error("error msg");"#);
    assert!(r.is_ok());
}

#[test]
fn test_console_info() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"console.info("info");"#);
    assert!(r.is_ok());
}

// ---- Math ----
#[test]
fn test_math_pi() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"Math.PI;"#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(std::f64::consts::PI));
}

#[test]
fn test_math_e() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"Math.E;"#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(std::f64::consts::E));
}

#[test]
fn test_math_abs() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"Math.abs(-5);"#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(5.0));
}

#[test]
fn test_math_floor() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"Math.floor(3.7);"#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(3.0));
}

#[test]
fn test_math_ceil() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"Math.ceil(3.2);"#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(4.0));
}

#[test]
fn test_math_round() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"Math.round(3.5);"#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(4.0));
}

#[test]
fn test_math_min() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"Math.min(3, 1, 4, 1, 5);"#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(1.0));
}

#[test]
fn test_math_max() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"Math.max(3, 1, 4, 1, 5);"#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(5.0));
}

#[test]
fn test_math_random() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"let x = Math.random(); x >= 0.0;"#);
    assert!(r.is_ok());
}

#[test]
fn test_math_pow() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"Math.pow(2, 10);"#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(1024.0));
}

#[test]
fn test_math_sqrt() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"Math.sqrt(16);"#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(4.0));
}

#[test]
fn test_math_log() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"Math.log(1);"#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(0.0));
}

#[test]
fn test_math_sin() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"Math.sin(0);"#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(0.0));
}

#[test]
fn test_math_cos() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"Math.cos(0);"#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(1.0));
}

#[test]
fn test_math_tan() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"Math.tan(0);"#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(0.0));
}

// ---- JSON ----
#[test]
fn test_json_parse_simple() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"let obj = JSON.parse('{"a": 1}'); obj.a;"#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(1_f64));
}

#[test]
fn test_json_parse_array() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"let arr = JSON.parse('[1, 2, 3]'); arr[1];"#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(2_f64));
}

#[test]
fn test_json_stringify_object() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"let s = JSON.stringify({a: 1}); s;"#);
    assert!(r.is_ok());
    if let tails::Value::String(s) = r.unwrap() {
        assert!(s.contains("\"a\""));
        assert!(s.contains("1"));
    } else {
        panic!("Expected string");
    }
}

#[test]
fn test_json_roundtrip() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let obj = JSON.parse('{"x": 10, "y": "hello"}');
        let s = JSON.stringify(obj);
        let obj2 = JSON.parse(s);
        obj2.x;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(10_f64));
}

// ---- Object.keys/values/entries/assign ----
#[test]
fn test_object_keys() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let keys = Object.keys({a: 1, b: 2, c: 3});
        keys.length;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(3.0));
}

#[test]
fn test_object_values() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let vals = Object.values({x: 10, y: 20});
        vals.length;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(2.0));
}

#[test]
fn test_object_entries() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let entries = Object.entries({a: 1});
        entries.length;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(1.0));
}

#[test]
fn test_object_assign() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let target = {a: 1};
        Object.assign(target, {b: 2}, {c: 3});
        target.b + target.c;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(5_f64));
}

// ---- Global functions ----
#[test]
fn test_parse_int() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"parseInt("42");"#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(42_f64));
}

#[test]
#[allow(clippy::approx_constant)]
fn test_parse_float() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"parseFloat("3.14");"#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(3.14));
}

#[test]
fn test_is_nan() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"isNaN(NaN);"#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Boolean(true));
}

#[test]
fn test_is_finite() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"isFinite(42);"#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Boolean(true));
}

#[test]
fn test_is_nan_with_number() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"isNaN(42);"#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Boolean(false));
}

// ---- Timer stubs ----
#[test]
fn test_set_timeout_returns_id() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"let id = setTimeout(function() {}, 100); id;"#);
    assert!(r.is_ok());
}

#[test]
fn test_clear_timeout() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"clearTimeout(1);"#);
    assert!(r.is_ok());
}

// ---- Array.prototype methods ----
#[test]
fn test_array_push() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let arr = [1, 2];
        let len = arr.push(3);
        arr.length;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(3.0));
}

#[test]
fn test_array_pop() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let arr = [1, 2, 3];
        let popped = arr.pop();
        arr.length;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(2.0));
}

#[test]
fn test_array_shift() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let arr = [1, 2, 3];
        let shifted = arr.shift();
        shifted;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(1_f64));
}

#[test]
fn test_array_unshift() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let arr = [2, 3];
        arr.unshift(1);
        arr.length;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(3.0));
}

#[test]
fn test_array_slice() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let arr = [1, 2, 3, 4, 5];
        let sliced = arr.slice(1, 3);
        sliced.length;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(2.0));
}

#[test]
fn test_array_splice() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let arr = [1, 2, 3, 4];
        let removed = arr.splice(1, 2);
        removed.length;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(2.0));
}

#[test]
fn test_array_index_of() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let arr = [10, 20, 30];
        arr.indexOf(20);
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(1_f64));
}

#[test]
fn test_array_includes() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let arr = [1, 2, 3];
        arr.includes(2);
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Boolean(true));
}

#[test]
fn test_array_find() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let arr = [1, 2, 3, 4];
        let found = arr.find(function(x) { return x > 2; });
        found;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(3_f64));
}

#[test]
fn test_array_find_index() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let arr = [1, 2, 3, 4];
        arr.findIndex(function(x) { return x > 2; });
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(2_f64));
}

#[test]
fn test_array_map() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let arr = [1, 2, 3];
        let doubled = arr.map(function(x) { return x * 2; });
        doubled.length;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(3.0));
}

#[test]
fn test_array_filter() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let arr = [1, 2, 3, 4, 5];
        let evens = arr.filter(function(x) { return x % 2 === 0; });
        evens.length;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(2.0));
}

#[test]
fn test_array_reduce() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let arr = [1, 2, 3, 4];
        let sum = arr.reduce(function(acc, x) { return acc + x; }, 0);
        sum;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(10_f64));
}

#[test]
fn test_array_for_each() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let sum = 0;
        [1, 2, 3].forEach(function(x) { sum = sum + x; });
        sum;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(6_f64));
}

#[test]
fn test_array_some() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        [1, 2, 3].some(function(x) { return x > 2; });
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Boolean(true));
}

#[test]
fn test_array_every() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        [2, 4, 6].every(function(x) { return x % 2 === 0; });
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Boolean(true));
}

#[test]
fn test_array_join() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let arr = [1, 2, 3];
        arr.join("-");
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::string("1-2-3"));
}

#[test]
fn test_array_reverse() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let arr = [1, 2, 3];
        arr.reverse();
        arr[0];
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(3_f64));
}

#[test]
fn test_array_sort() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let arr = [3, 1, 4, 1, 5];
        arr.sort();
        arr[0];
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(1_f64));
}

#[test]
fn test_array_concat() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let arr = [1, 2].concat([3, 4]);
        arr.length;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(4.0));
}

#[test]
fn test_array_flat() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let arr = [[1, 2], [3, 4]];
        let flat = arr.flat();
        flat.length;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(4.0));
}

// ---- String.prototype methods ----
#[test]
fn test_string_char_at() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#""hello".charAt(1);"#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::string("e"));
}

#[test]
fn test_string_char_code_at() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#""A".charCodeAt(0);"#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(65_f64));
}

#[test]
fn test_string_slice() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#""hello".slice(1, 3);"#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::string("el"));
}

#[test]
fn test_string_substring() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#""hello".substring(1, 4);"#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::string("ell"));
}

#[test]
fn test_string_index_of() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#""hello world".indexOf("world");"#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(6_f64));
}

#[test]
fn test_string_includes() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#""hello".includes("ell");"#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Boolean(true));
}

#[test]
fn test_string_replace() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#""hello world".replace("world", "JS");"#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::string("hello JS"));
}

#[test]
fn test_string_split() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let parts = "a,b,c".split(",");
        parts.length;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(3.0));
}

#[test]
fn test_string_trim() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#""  hello  ".trim();"#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::string("hello"));
}

#[test]
fn test_string_to_lower_case() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#""HELLO".toLowerCase();"#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::string("hello"));
}

#[test]
fn test_string_to_upper_case() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#""hello".toUpperCase();"#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::string("HELLO"));
}

#[test]
fn test_string_starts_with() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#""hello".startsWith("hel");"#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Boolean(true));
}

#[test]
fn test_string_ends_with() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#""hello".endsWith("llo");"#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Boolean(true));
}

#[test]
fn test_string_repeat() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#""ha".repeat(3);"#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::string("hahaha"));
}

#[test]
fn test_string_pad_start() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#""5".padStart(3, "0");"#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::string("005"));
}

#[test]
fn test_string_pad_end() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#""5".padEnd(3, "0");"#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::string("500"));
}

// ---- Number methods ----
#[test]
fn test_number_parse_int() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"Number.parseInt("42");"#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(42_f64));
}

#[test]
#[allow(clippy::approx_constant)]
fn test_number_parse_float() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"Number.parseFloat("3.14");"#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(3.14));
}

#[test]
fn test_number_is_nan() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"Number.isNaN(NaN);"#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Boolean(true));
}

#[test]
fn test_number_is_finite() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(r#"Number.isFinite(42);"#);
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Boolean(true));
}

// ---- Combined/integration tests ----
#[test]
fn test_array_map_with_arrow() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let arr = [1, 2, 3];
        let result = arr.map(x => x * 10);
        result[0] + result[1] + result[2];
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(60_f64));
}

#[test]
fn test_array_filter_reduce_combined() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let data = [1, 2, 3, 4, 5, 6];
        let sum = data
            .filter(function(x) { return x % 2 === 0; })
            .reduce(function(acc, x) { return acc + x; }, 0);
        sum;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(12_f64));
}

#[test]
fn test_json_parse_and_object_keys() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let data = JSON.parse('{"name": "Alice", "age": 30}');
        let keys = Object.keys(data);
        keys.length;
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(2.0));
}

#[test]
fn test_string_split_and_join() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let parts = "hello-world-foo".split("-");
        parts.join(" ");
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::string("hello world foo"));
}

#[test]
fn test_math_in_array_reduce() {
    let mut rt = TailsRuntime::default();
    let r = rt.eval(
        r#"
        let arr = [-3, 1, -4, 1, 5];
        let result = arr.map(function(x) { return Math.abs(x); });
        result.reduce(function(acc, x) { return acc + x; }, 0);
    "#,
    );
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), tails::Value::Float(14_f64));
}
