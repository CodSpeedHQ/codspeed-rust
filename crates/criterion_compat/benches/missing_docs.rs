//! Harness macros expand to items that pass `missing_docs`.
#![deny(missing_docs)]

use codspeed_criterion_compat::{black_box, criterion_group, criterion_main, Criterion};

fn sum(c: &mut Criterion) {
    c.bench_function("sum", |b| b.iter(|| black_box(1u64) + 1));
}

criterion_group!(benches, sum);
criterion_main!(benches);
