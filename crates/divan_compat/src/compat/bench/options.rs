/// Benchmarking options set directly by the user in `#[divan::bench]` and
/// `#[divan::bench_group]`.
///
/// Changes to fields must be reflected in the "Options" sections of the docs
/// for `#[divan::bench]` and `#[divan::bench_group]`.
#[derive(Default)]
pub struct BenchOptions {
    /// Whether the benchmark should be ignored.
    ///
    /// This may be set within the attribute or with a separate
    /// [`#[ignore]`](https://doc.rust-lang.org/reference/attributes/testing.html#the-ignore-attribute).
    pub ignore: Option<bool>,

    /// Counters to be used with this benchmark.
    ///
    /// Note: Counters are not yet supported by codspeed-divan-compat.
    /// This field is provided for API compatibility but does not affect benchmarking.
    pub counters: Option<Vec<String>>,
}
