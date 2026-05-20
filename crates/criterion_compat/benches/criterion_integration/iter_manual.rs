use codspeed_criterion_compat::{criterion_group, Criterion, IterManualOptions};

fn iter_manual_basic(c: &mut Criterion) {
    // rounds=5 is below criterion's sample_size >= 10 floor — proves
    // iter_manual fully bypasses the adaptive outer sampler.
    c.bench_function("iter_manual_basic", |b| {
        b.iter_manual(IterManualOptions::new(5, 100), || {
            let mut s = 0u64;
            for i in 0..32 {
                s = s.wrapping_add(i);
            }
            s
        });
    });
}

fn iter_manual_with_setup(c: &mut Criterion) {
    c.bench_function("iter_manual_setup", |b| {
        b.iter_manual_setup(
            IterManualOptions {
                rounds: 3,
                iterations: 100,
                warmup_rounds: 2,
            },
            || (0u64..256).collect::<Vec<_>>(),
            |input| input.iter().copied().sum::<u64>(),
        );
    });
}

fn iter_manual_with_teardown(c: &mut Criterion) {
    c.bench_function("iter_manual_setup_teardown", |b| {
        b.iter_manual_setup_teardown(
            IterManualOptions {
                rounds: 7,
                iterations: 50,
                warmup_rounds: 1,
            },
            || (0u64..128).collect::<Vec<_>>(),
            |input| input.iter().copied().sum::<u64>(),
            drop,
        );
    });
}

#[cfg(feature = "async_futures")]
fn iter_manual_async_basic(c: &mut Criterion) {
    use codspeed_criterion_compat::async_executor::FuturesExecutor;
    c.bench_function("iter_manual_async_basic", |b| {
        b.to_async(FuturesExecutor)
            .iter_manual(IterManualOptions::new(5, 100), || async {
                let mut s = 0u64;
                for i in 0..32 {
                    s = s.wrapping_add(i);
                }
                s
            });
    });
}

#[cfg(feature = "async_futures")]
fn iter_manual_async_with_setup(c: &mut Criterion) {
    use codspeed_criterion_compat::async_executor::FuturesExecutor;
    c.bench_function("iter_manual_async_setup", |b| {
        b.to_async(FuturesExecutor).iter_manual_setup(
            IterManualOptions {
                rounds: 3,
                iterations: 100,
                warmup_rounds: 2,
            },
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
            IterManualOptions {
                rounds: 7,
                iterations: 50,
                warmup_rounds: 1,
            },
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
fn iter_manual_async_basic(_c: &mut Criterion) {}
#[cfg(not(feature = "async_futures"))]
fn iter_manual_async_with_setup(_c: &mut Criterion) {}
#[cfg(not(feature = "async_futures"))]
fn iter_manual_async_with_teardown(_c: &mut Criterion) {}

criterion_group!(
    benches,
    iter_manual_basic,
    iter_manual_with_setup,
    iter_manual_with_teardown,
    iter_manual_async_basic,
    iter_manual_async_with_setup,
    iter_manual_async_with_teardown,
);
