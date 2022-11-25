use codspeed_criterion_compat::{criterion_group, Criterion};

fn some_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("\"*group/\"");
    group.bench_function("\"*benchmark/\" '", |b| b.iter(|| 1 + 1));
    group.finish();
}

criterion_group!(benches, some_benchmark);
