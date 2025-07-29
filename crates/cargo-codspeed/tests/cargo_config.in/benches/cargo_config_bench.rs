use codspeed_criterion_compat::{criterion_group, criterion_main, Criterion};

fn bench_failing_without_custom_flag(c: &mut Criterion) {
    c.bench_function("custom_flag_disabled", |b| {
        b.iter(|| {
            // This will cause a compilation error if custom_feature_flag is not set
            #[cfg(not(custom_feature_flag))]
            compile_error!(
                "custom_feature_flag is not enabled - .cargo/config.toml rustflags not applied"
            );

            #[cfg(not(target_feature_flag))]
            compile_error!(
                "target_feature_flag is not enabled - .cargo/config.toml rustflags not applied"
            );
        })
    });
}

criterion_group!(benches, bench_failing_without_custom_flag);

criterion_main!(benches);
