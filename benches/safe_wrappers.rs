use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::ffi::CString;
use std::time::Duration;
use tails::ffi::safe_wrappers::{SafeCStr, SafePtr, SafeSlice};

fn bench_config() -> Criterion {
    Criterion::default()
        .sample_size(50)
        .measurement_time(Duration::from_secs(1))
        .warm_up_time(Duration::from_millis(300))
        .without_plots()
}

fn bench_safe_ptr_creation(c: &mut Criterion) {
    let value = 42i32;
    let ptr = &value as *const i32 as *mut i32;

    c.bench_function("safe_ptr_new", |b| {
        b.iter(|| unsafe { SafePtr::new(black_box(ptr)) })
    });
}

fn bench_safe_ptr_as_ref(c: &mut Criterion) {
    let value = 42i32;
    let ptr = &value as *const i32 as *mut i32;
    let safe_ptr = unsafe { SafePtr::new(ptr) };

    c.bench_function("safe_ptr_as_ref", |b| {
        b.iter(|| unsafe { black_box(safe_ptr.as_ref()) })
    });
}

fn bench_safe_cstr_creation(c: &mut Criterion) {
    let c_string = CString::new("hello world").unwrap();
    let ptr = c_string.as_ptr();

    c.bench_function("safe_cstr_new", |b| {
        b.iter(|| unsafe { SafeCStr::new(black_box(ptr)) })
    });
}

fn bench_safe_cstr_to_str(c: &mut Criterion) {
    let c_string = CString::new("hello world").unwrap();
    let safe_cstr = unsafe { SafeCStr::new(c_string.as_ptr()) };

    c.bench_function("safe_cstr_to_str", |b| {
        b.iter(|| black_box(safe_cstr.to_str()))
    });
}

fn bench_safe_slice_creation(c: &mut Criterion) {
    let data = [1i32, 2, 3, 4, 5];
    let ptr = data.as_ptr();
    let len = data.len();

    c.bench_function("safe_slice_new", |b| {
        b.iter(|| unsafe { SafeSlice::new(black_box(ptr), black_box(len)) })
    });
}

fn bench_safe_slice_as_slice(c: &mut Criterion) {
    let data = [1i32, 2, 3, 4, 5];
    let safe_slice = unsafe { SafeSlice::new(data.as_ptr(), data.len()) };

    c.bench_function("safe_slice_as_slice", |b| {
        b.iter(|| unsafe { black_box(safe_slice.as_slice()) })
    });
}

criterion_group! {
    name = safe_wrappers;
    config = bench_config();
    targets =
        bench_safe_ptr_creation,
        bench_safe_ptr_as_ref,
        bench_safe_cstr_creation,
        bench_safe_cstr_to_str,
        bench_safe_slice_creation,
        bench_safe_slice_as_slice
}
criterion_main!(safe_wrappers);
