use codspeed_criterion_compat::{criterion_group, Criterion};
use std::time::Duration;

fn sleep_benchmarks(c: &mut Criterion) {
    c.bench_function("sleep_1ms", |b| {
        b.iter(|| std::thread::sleep(Duration::from_millis(1)))
    });

    c.bench_function("sleep_10ms", |b| {
        b.iter(|| std::thread::sleep(Duration::from_millis(10)))
    });

    c.bench_function("sleep_50ms", |b| {
        b.iter(|| std::thread::sleep(Duration::from_millis(50)))
    });

    c.bench_function("sleep_100ms", |b| {
        b.iter(|| std::thread::sleep(Duration::from_millis(100)))
    });
}

criterion_group!(benches, sleep_benchmarks);
