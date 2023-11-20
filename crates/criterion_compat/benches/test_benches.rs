use codspeed_criterion_compat::{
    criterion_group, criterion_main, Bencher, BenchmarkGroup, BenchmarkId, Criterion,
};
use criterion::measurement::WallTime;

fn bench(c: &mut Criterion) {
    // Setup (construct data, allocate memory, etc)
    let input = 5u64;
    c.bench_with_input(BenchmarkId::new("with_input", input), &input, |b, i| {
        b.iter(|| {
            let mut x = 0;
            for _ in 0..*i {
                x += 2;
            }
            x
        })
    });
}

fn bench_with_explicit_lifetime(c: &mut Criterion) {
    let input = 5u64;
    c.bench_with_input(
        BenchmarkId::new("with_input", input),
        &input,
        |b: &mut Bencher<'_>, i| {
            b.iter(|| {
                let mut x = 0;
                for _ in 0..*i {
                    x += 2;
                }
                x
            })
        },
    );
}

#[cfg(codspeed)]
fn bench_using_group_without_explicit_measurement(c: &mut Criterion) {
    let mut group = c.benchmark_group("group");
    fn using_group(g: &mut BenchmarkGroup) {
        g.bench_function("bench_without_explicit_measurement", |b| b.iter(|| 2 + 2));
    }
    using_group(&mut group);
    group.finish();
}

fn bench_using_group_with_explicit_measurement(c: &mut Criterion) {
    let mut group = c.benchmark_group("group");
    fn using_group(g: &mut BenchmarkGroup<'_, WallTime>) {
        g.bench_function("bench_explicit_measurement", |b| b.iter(|| 2 + 2));
    }
    using_group(&mut group);
    group.finish();
}

mod nested {
    use super::*;
    pub fn bench(c: &mut Criterion) {
        // Setup (construct data, allocate memory, etc)
        let input = 5u64;
        c.bench_with_input(BenchmarkId::new("with_input", input), &input, |b, i| {
            b.iter(|| {
                let mut x = 0;
                for _ in 0..*i {
                    x += 2;
                }
                x
            })
        });
    }
}

criterion_group!(
    benches,
    bench,
    bench_with_explicit_lifetime,
    nested::bench,
    bench_using_group_with_explicit_measurement,
);

#[cfg(not(codspeed))]
criterion_main!(benches);

#[cfg(codspeed)]
criterion_group!(
    only_codspeed,
    bench_using_group_without_explicit_measurement
);

#[cfg(codspeed)]
criterion_main!(benches, only_codspeed);
