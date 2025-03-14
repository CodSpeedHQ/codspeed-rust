use codspeed::codspeed::black_box;
use codspeed_bencher_compat::{benchmark_group, benchmark_main, Bencher};

#[cfg(feature = "default_feature")]
pub fn with_default_feature(bench: &mut Bencher) {
    bench.iter(|| (0..100).fold(0, |x, y| black_box(x + y)))
}

#[cfg(not(feature = "default_feature"))]
pub fn without_default_feature(bench: &mut Bencher) {
    bench.iter(|| (0..100).fold(0, |x, y| black_box(x + y)))
}

#[cfg(not(feature = "sample_feature"))]
pub fn without_feature(bench: &mut Bencher) {
    bench.iter(|| (0..100).fold(0, |x, y| black_box(x + y)))
}

#[cfg(feature = "sample_feature")]
pub fn with_feature(bench: &mut Bencher) {
    bench.iter(|| (0..100).fold(0, |x, y| black_box(x + y)))
}

#[cfg(feature = "sample_feature")]
benchmark_group!(sample_benches, with_feature);
#[cfg(not(feature = "sample_feature"))]
benchmark_group!(sample_benches, without_feature);

#[cfg(feature = "default_feature")]
benchmark_group!(default_benches, with_default_feature);
#[cfg(not(feature = "default_feature"))]
benchmark_group!(default_benches, without_default_feature);

benchmark_main!(sample_benches, default_benches);
