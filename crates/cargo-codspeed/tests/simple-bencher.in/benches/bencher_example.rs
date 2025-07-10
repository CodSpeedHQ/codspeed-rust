use bencher::{benchmark_group, benchmark_main, Bencher};
use codspeed::codspeed::black_box;

pub fn a(bench: &mut Bencher) {
    bench.iter(|| (0..100).fold(0, |x, y| black_box(x + y)))
}

pub fn b(bench: &mut Bencher) {
    const N: usize = 1024;
    bench.iter(|| vec![0u8; N]);

    bench.bytes = N as u64;
}

benchmark_group!(benches, a, b);
benchmark_main!(benches);
