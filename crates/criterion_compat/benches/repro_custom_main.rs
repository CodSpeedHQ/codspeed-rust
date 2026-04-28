/// Reproducer for COD-2324: URI formatting issues when users bypass criterion_group!/criterion_main!
/// and use a custom main function (like the rtmalloc project does).
use codspeed_criterion_compat::{criterion_group, BenchmarkId, Criterion};

fn bench_with_group(c: &mut Criterion) {
    let mut group = c.benchmark_group("my_group");
    group.bench_function("my_bench", |b| b.iter(|| 2 + 2));
    group.bench_function(BenchmarkId::new("parameterized", 42), |b| b.iter(|| 2 + 2));
    group.finish();
}

criterion_group!(benches, bench_with_group);

#[cfg(codspeed)]
fn main() {
    // Pattern A: Using new_instrumented() but calling bench functions directly (not through criterion_group!)
    let mut criterion = Criterion::new_instrumented();
    bench_with_group(&mut criterion);

    // Pattern B: Calling through criterion_group!-generated function (should work correctly)
    let mut criterion2 = Criterion::new_instrumented();
    benches(&mut criterion2);
}

#[cfg(not(codspeed))]
fn main() {
    // Without codspeed, just run through upstream criterion directly
    let mut criterion = Criterion::default().configure_from_args();
    bench_with_group(&mut criterion);
}
