/// Benchmarking options set directly by the user in `#[divan::bench]` and
/// `#[divan::bench_group]`.
///
/// Changes to fields must be reflected in the "Options" sections of the docs
/// for `#[divan::bench]` and `#[divan::bench_group]`.
#[derive(Default)]
pub struct BenchOptions<'a> {
    pub(crate) _marker: std::marker::PhantomData<&'a ()>,
}
