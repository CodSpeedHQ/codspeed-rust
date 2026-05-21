use codspeed_criterion_compat::{criterion_group, Criterion, IterManualOptions};

fn iter_manual_with_setup(c: &mut Criterion) {
    // rounds=3 is below criterion's sample_size >= 10 floor — proves
    // iter_manual fully bypasses the adaptive outer sampler.
    c.bench_function("iter_manual_setup", |b| {
        b.iter_manual_setup(
            IterManualOptions::new().rounds(3).iters(100).warmup(2),
            || (0u64..256).collect::<Vec<_>>(),
            |input| input.iter().copied().sum::<u64>(),
        );
    });
}

fn iter_manual_with_teardown(c: &mut Criterion) {
    c.bench_function("iter_manual_setup_teardown", |b| {
        b.iter_manual_setup_teardown(
            IterManualOptions::new().rounds(7).iters(50).warmup(1),
            || (0u64..128).collect::<Vec<_>>(),
            |input| input.iter().copied().sum::<u64>(),
            drop,
        );
    });
}

// Setup body deliberately does a chunk of work so it stands out in flamegraphs.
// The measured region should NOT include this work.
fn iter_manual_heavy_setup(c: &mut Criterion) {
    c.bench_function("iter_manual_heavy_setup", |b| {
        b.iter_manual_setup(
            IterManualOptions::new().rounds(3).iters(50).warmup(1),
            || {
                // Heavy setup: build a large vector. Should NOT appear inside
                // the measured region of the flamegraph.
                let mut v = Vec::with_capacity(10_000);
                for i in 0..10_000u64 {
                    v.push(i.wrapping_mul(31));
                }
                v
            },
            |input| input.iter().copied().sum::<u64>(),
        );
    });
}

#[cfg(feature = "async_futures")]
fn iter_manual_async_with_setup(c: &mut Criterion) {
    use codspeed_criterion_compat::async_executor::FuturesExecutor;
    c.bench_function("iter_manual_async_setup", |b| {
        b.to_async(FuturesExecutor).iter_manual_setup(
            IterManualOptions::new().rounds(3).iters(100).warmup(2),
            || (0u64..256).collect::<Vec<_>>(),
            |input| {
                let sum = input.iter().copied().sum::<u64>();
                async move { sum }
            },
        );
    });
}

#[cfg(feature = "async_futures")]
fn iter_manual_async_with_teardown(c: &mut Criterion) {
    use codspeed_criterion_compat::async_executor::FuturesExecutor;
    c.bench_function("iter_manual_async_setup_teardown", |b| {
        b.to_async(FuturesExecutor).iter_manual_setup_teardown(
            IterManualOptions::new().rounds(7).iters(50).warmup(1),
            || (0u64..128).collect::<Vec<_>>(),
            |input| {
                let sum = input.iter().copied().sum::<u64>();
                async move { sum }
            },
            |input| async move { drop(input) },
        );
    });
}

#[cfg(not(feature = "async_futures"))]
fn iter_manual_async_with_setup(_c: &mut Criterion) {}
#[cfg(not(feature = "async_futures"))]
fn iter_manual_async_with_teardown(_c: &mut Criterion) {}

criterion_group!(
    benches,
    iter_manual_with_setup,
    iter_manual_with_teardown,
    iter_manual_heavy_setup,
    iter_manual_async_with_setup,
    iter_manual_async_with_teardown,
);
