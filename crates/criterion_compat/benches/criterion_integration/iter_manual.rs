use codspeed_criterion_compat::{criterion_group, Criterion, IterManualOptions};

fn iter_manual_simple(c: &mut Criterion) {
    c.bench_function("iter_manual_simple", |b| {
        b.iter_manual_unstable(
            IterManualOptions::new().rounds(3).iters(5).warmup(1),
            || std::thread::sleep(std::time::Duration::from_millis(100)),
        );
    });
}

fn iter_manual_with_external_setup(c: &mut Criterion) {
    c.bench_function("iter_manual_with_external_setup", |b| {
        // Setup deliberately does a chunk of work so it stands out in flamegraphs.
        // The measured region should NOT include this work.
        let input: Vec<u64> = (0..10_000u64).map(|i| i.wrapping_mul(31)).collect();
        b.iter_manual_unstable(
            IterManualOptions::new().rounds(3).iters(50).warmup(1),
            || input.iter().copied().sum::<u64>(),
        );
    });
}

#[cfg(feature = "async_futures")]
fn iter_manual_async(c: &mut Criterion) {
    use codspeed_criterion_compat::async_executor::FuturesExecutor;
    c.bench_function("iter_manual_async", |b| {
        b.to_async(FuturesExecutor).iter_manual_unstable(
            IterManualOptions::new().rounds(3).iters(100).warmup(2),
            || async { (0u64..256).sum::<u64>() },
        );
    });
}

#[cfg(not(feature = "async_futures"))]
fn iter_manual_async(_c: &mut Criterion) {}

criterion_group!(
    benches,
    iter_manual_simple,
    iter_manual_with_external_setup,
    iter_manual_async,
);
