use codspeed_criterion_compat::{black_box, criterion_group, criterion_main, Criterion};

pub fn a(c: &mut Criterion) {
    c.bench_function("sum_fold", |b| {
        b.iter(|| (0..100).fold(0, |x, y| black_box(x + y)))
    });
}

pub fn b(c: &mut Criterion) {
    const N: usize = 1024;
    c.bench_function("build_vec", |b| b.iter(|| vec![0u8; N]));
}

criterion_group!(benches, a, b);
criterion_main!(benches);
