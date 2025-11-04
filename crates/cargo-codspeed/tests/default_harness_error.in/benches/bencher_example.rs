use codspeed::codspeed::black_box;
use codspeed_bencher_compat::{benchmark_group, benchmark_main, Bencher};

pub fn a(bench: &mut Bencher) {
    bench.iter(|| (0..100).fold(0, |x, y| black_box(x + y)))
}

benchmark_group!(benches, a);
benchmark_main!(benches);
