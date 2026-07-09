use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Duration;
use tails::{RuntimeConfig, TailsRuntime};

/// Shared Criterion config: short enough that a full `cargo bench` finishes,
/// long enough for stable-ish micro numbers. Override with CRITERION_* env.
fn bench_config() -> Criterion {
    Criterion::default()
        .sample_size(50)
        .measurement_time(Duration::from_secs(2))
        .warm_up_time(Duration::from_millis(500))
        // Skip plot generation so local iteration stays fast.
        .without_plots()
}

fn new_rt() -> TailsRuntime {
    TailsRuntime::new(RuntimeConfig::default()).expect("runtime")
}

fn bench_hello_world(c: &mut Criterion) {
    let mut rt = new_rt();
    c.bench_function("eval_hello_world", |b| {
        b.iter(|| {
            let _ = rt.eval(black_box("1 + 2"));
        })
    });
}

fn bench_arithmetic(c: &mut Criterion) {
    let script = r#"
        let x = 0;
        for (let i = 0; i < 100; i++) {
            x = x + i;
        }
        x
    "#;
    let mut rt = new_rt();
    c.bench_function("eval_arithmetic_100", |b| {
        b.iter(|| {
            let _ = rt.eval(black_box(script));
        })
    });
}

fn bench_arithmetic_1000(c: &mut Criterion) {
    let script = r#"
        let x = 0;
        for (let i = 0; i < 1000; i++) {
            x = x + i;
        }
        x
    "#;
    let mut rt = new_rt();
    c.bench_function("eval_arithmetic_1000", |b| {
        b.iter(|| {
            let _ = rt.eval(black_box(script));
        })
    });
}

fn bench_object_creation(c: &mut Criterion) {
    let script = r#"
        let obj = {};
        for (let i = 0; i < 20; i++) {
            obj["k" + i] = i;
        }
        obj
    "#;
    let mut rt = new_rt();
    c.bench_function("eval_object_creation_20", |b| {
        b.iter(|| {
            let _ = rt.eval(black_box(script));
        })
    });
}

fn bench_array_operations(c: &mut Criterion) {
    let script = r#"
        let arr = [];
        for (let i = 0; i < 20; i++) {
            arr.push(i);
        }
        arr
    "#;
    let mut rt = new_rt();
    c.bench_function("eval_array_push_20", |b| {
        b.iter(|| {
            let _ = rt.eval(black_box(script));
        })
    });
}

fn bench_array_push_100(c: &mut Criterion) {
    let script = r#"
        let arr = [];
        for (let i = 0; i < 100; i++) {
            arr.push(i * 2);
        }
        arr.length
    "#;
    let mut rt = new_rt();
    c.bench_function("eval_array_push_100", |b| {
        b.iter(|| {
            let _ = rt.eval(black_box(script));
        })
    });
}

fn bench_function_calls(c: &mut Criterion) {
    let script = r#"
        function fib(n) {
            if (n <= 1) return n;
            return fib(n - 1) + fib(n - 2);
        }
        fib(10)
    "#;
    let mut rt = new_rt();
    c.bench_function("eval_fib_10", |b| {
        b.iter(|| {
            let _ = rt.eval(black_box(script));
        })
    });
}

fn bench_function_calls_sum(c: &mut Criterion) {
    let script = r#"
        function add(a, b) { return a + b; }
        let total = 0;
        for (let i = 0; i < 100; i++) {
            total = add(total, i);
        }
        total
    "#;
    let mut rt = new_rt();
    c.bench_function("eval_call_sum_100", |b| {
        b.iter(|| {
            let _ = rt.eval(black_box(script));
        })
    });
}

fn bench_string_operations(c: &mut Criterion) {
    let script = r#"
        let s = "hello";
        for (let i = 0; i < 20; i++) {
            s = s + "x";
        }
        s.length
    "#;
    let mut rt = new_rt();
    c.bench_function("eval_string_concat_20", |b| {
        b.iter(|| {
            let _ = rt.eval(black_box(script));
        })
    });
}

fn bench_string_concat_50(c: &mut Criterion) {
    let script = r#"
        let s = "ab";
        for (let i = 0; i < 50; i++) {
            s = s + "cd";
        }
        s.length
    "#;
    let mut rt = new_rt();
    c.bench_function("eval_string_concat_50", |b| {
        b.iter(|| {
            let _ = rt.eval(black_box(script));
        })
    });
}

fn bench_string_concat_two_literals(c: &mut Criterion) {
    // String + String with both operands as locals (the Phase 5E hot path)
    let script = r#"
        let a = "hello";
        let b = "world";
        let out = "";
        for (let i = 0; i < 20; i++) {
            out = a + b;
        }
        out.length
    "#;
    let mut rt = new_rt();
    c.bench_function("eval_string_concat_local_20", |b| {
        b.iter(|| {
            let _ = rt.eval(black_box(script));
        })
    });
}

fn bench_loop_only(c: &mut Criterion) {
    // Pure loop with no body to measure dispatch overhead.
    let script = r#"
        let n = 0;
        for (let i = 0; i < 1000; i++) {
            n = i;
        }
        n
    "#;
    let mut rt = new_rt();
    c.bench_function("eval_loop_only_1000", |b| {
        b.iter(|| {
            let _ = rt.eval(black_box(script));
        })
    });
}

fn bench_nested_loop(c: &mut Criterion) {
    let script = r#"
        let total = 0;
        for (let i = 0; i < 50; i++) {
            for (let j = 0; j < 50; j++) {
                total = total + (i * j);
            }
        }
        total
    "#;
    let mut rt = new_rt();
    c.bench_function("eval_nested_loop_50x50", |b| {
        b.iter(|| {
            let _ = rt.eval(black_box(script));
        })
    });
}

fn bench_json_parse(c: &mut Criterion) {
    let script = r#"
        let obj = JSON.parse('{"a":1,"b":2,"c":3}');
        obj.a + obj.b + obj.c
    "#;
    let mut rt = new_rt();
    c.bench_function("eval_json_parse", |b| {
        b.iter(|| {
            let _ = rt.eval(black_box(script));
        })
    });
}

fn bench_closures(c: &mut Criterion) {
    // Regression: 1M-scale suite used to TIMEOUT; keep a moderate N here so
    // criterion finishes and still exercises MakeClosure + call.
    let script = r#"
        function makeAdder(n) {
            return function(x) { return x + n; };
        }
        let sum = 0;
        for (let i = 0; i < 1000; i++) {
            const add = makeAdder(i);
            sum = sum + add(1);
        }
        sum
    "#;
    let mut rt = new_rt();
    c.bench_function("eval_closures_1000", |b| {
        b.iter(|| {
            let _ = rt.eval(black_box(script));
        })
    });
}

criterion_group! {
    name = benches;
    config = bench_config();
    targets =
        bench_hello_world,
        bench_arithmetic,
        bench_arithmetic_1000,
        bench_object_creation,
        bench_array_operations,
        bench_array_push_100,
        bench_function_calls,
        bench_function_calls_sum,
        bench_string_operations,
        bench_string_concat_50,
        bench_string_concat_two_literals,
        bench_loop_only,
        bench_nested_loop,
        bench_json_parse,
        bench_closures
}
criterion_main!(benches);
