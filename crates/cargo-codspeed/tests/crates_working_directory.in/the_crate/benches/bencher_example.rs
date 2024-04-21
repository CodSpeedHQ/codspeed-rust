use std::hint::black_box;

use codspeed_bencher_compat::{benchmark_group, benchmark_main, Bencher};

pub fn a(bench: &mut Bencher) {
    // Open ./input.txt file
    std::fs::read_to_string("./input.txt").expect("Failed to read file");
    bench.iter(|| (0..100).fold(0, |x, y| black_box(x + y)))
}

benchmark_group!(benches, a);
benchmark_main!(benches);
