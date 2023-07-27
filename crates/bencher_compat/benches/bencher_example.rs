use codspeed::codspeed::black_box;
use codspeed_bencher_compat::{benchmark_group, benchmark_main, Bencher};

pub fn a(bench: &mut Bencher) {
    bench.iter(|| (0..100).fold(0, |x, y| black_box(x + y)))
}

pub fn b(bench: &mut Bencher) {
    const N: usize = 1024;
    bench.iter(|| vec![0u8; N]);

    bench.bytes = N as u64;
}

mod c {
    use super::*;

    pub fn a(bench: &mut Bencher) {
        bench.iter(|| (0..100).fold(0, |x, y| black_box(x + y)))
    }

    pub fn b(bench: &mut Bencher) {
        const N: usize = 1024;
        bench.iter(|| vec![0u8; N]);

        bench.bytes = N as u64;
    }
}

benchmark_group!(benches, a, b, c::a, c::b);
benchmark_main!(benches);
