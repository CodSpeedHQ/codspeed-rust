use codspeed::codspeed::black_box;
use codspeed_bencher_compat::{benchmark_group, benchmark_main, Bencher};

#[cfg(not(feature = "sample_feature"))]
pub fn without_feature(bench: &mut Bencher) {
    bench.iter(|| (0..100).fold(0, |x, y| black_box(x + y)))
}
#[cfg(not(feature = "sample_feature"))]
benchmark_group!(benches, without_feature);

#[cfg(feature = "sample_feature")]
pub fn with_feature(bench: &mut Bencher) {
    bench.iter(|| (0..100).fold(0, |x, y| black_box(x + y)))
}
#[cfg(feature = "sample_feature")]
benchmark_group!(benches, with_feature);

benchmark_main!(benches);
