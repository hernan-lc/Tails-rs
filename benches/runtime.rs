use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tails::TailsRuntime;

fn bench_arithmetic(c: &mut Criterion) {
    let mut runtime = TailsRuntime::default().unwrap();
    
    c.bench_function("arithmetic", |b| {
        b.iter(|| {
            runtime.eval(black_box("2 + 3 * 4")).unwrap()
        })
    });
}

fn bench_string_concatenation(c: &mut Criterion) {
    let mut runtime = TailsRuntime::default().unwrap();
    
    c.bench_function("string_concat", |b| {
        b.iter(|| {
            runtime.eval(black_box(r#""hello" + " " + "world""#)).unwrap()
        })
    });
}

fn bench_fibonacci(c: &mut Criterion) {
    let mut runtime = TailsRuntime::default().unwrap();
    runtime.eval(r#"
        function fibonacci(n) {
            if (n <= 1) {
                return n;
            }
            return fibonacci(n - 1) + fibonacci(n - 2);
        }
    "#).unwrap();
    
    c.bench_function("fibonacci(20)", |b| {
        b.iter(|| {
            runtime.eval(black_box("fibonacci(20)")).unwrap()
        })
    });
}

fn bench_factorial(c: &mut Criterion) {
    let mut runtime = TailsRuntime::default().unwrap();
    runtime.eval(r#"
        function factorial(n) {
            if (n <= 1) {
                return 1;
            }
            return n * factorial(n - 1);
        }
    "#).unwrap();
    
    c.bench_function("factorial(20)", |b| {
        b.iter(|| {
            runtime.eval(black_box("factorial(20)")).unwrap()
        })
    });
}

fn bench_loop(c: &mut Criterion) {
    let mut runtime = TailsRuntime::default().unwrap();
    
    c.bench_function("loop_1000", |b| {
        b.iter(|| {
            runtime.eval(black_box(r#"
                let sum = 0;
                let i = 0;
                while (i < 1000) {
                    sum = sum + i;
                    i = i + 1;
                }
                sum
            "#)).unwrap()
        })
    });
}

fn bench_function_call(c: &mut Criterion) {
    let mut runtime = TailsRuntime::default().unwrap();
    runtime.eval(r#"
        function add(a, b) {
            return a + b;
        }
    "#).unwrap();
    
    c.bench_function("function_call", |b| {
        b.iter(|| {
            runtime.eval(black_box("add(1, 2)")).unwrap()
        })
    });
}

fn bench_nested_function(c: &mut Criterion) {
    let mut runtime = TailsRuntime::default().unwrap();
    runtime.eval(r#"
        function outer(x) {
            function inner(y) {
                return x + y;
            }
            return inner(10);
        }
    "#).unwrap();
    
    c.bench_function("nested_function", |b| {
        b.iter(|| {
            runtime.eval(black_box("outer(5)")).unwrap()
        })
    });
}

fn bench_closures(c: &mut Criterion) {
    let mut runtime = TailsRuntime::default().unwrap();
    runtime.eval(r#"
        function makeAdder(x) {
            return function(y) {
                return x + y;
            };
        }
        const add5 = makeAdder(5);
    "#).unwrap();
    
    c.bench_function("closure", |b| {
        b.iter(|| {
            runtime.eval(black_box("add5(10)")).unwrap()
        })
    });
}

criterion_group!(
    benches,
    bench_arithmetic,
    bench_string_concatenation,
    bench_fibonacci,
    bench_factorial,
    bench_loop,
    bench_function_call,
    bench_nested_function,
    bench_closures,
);
criterion_main!(benches);
