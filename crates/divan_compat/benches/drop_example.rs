use codspeed_divan_compat::Bencher;

struct LargeInput {
    data: Vec<u8>,
}

impl Drop for LargeInput {
    #[inline(never)]
    fn drop(&mut self) {
        // Simulate a large drop by performing some computation
        let sum: u8 = self.data.iter().copied().sum();
        std::hint::black_box(sum); // Prevent optimization
    }
}

impl LargeInput {
    fn new() -> Self {
        Self {
            data: vec![42; 1024 * 1024 /* 1MiB */],
        }
    }

    fn process(&self) -> u64 {
        // Simulate some work on the data
        std::thread::sleep(std::time::Duration::from_millis(50));
        self.data.iter().map(|&x| x as u64).sum()
    }
}

#[codspeed_divan_compat::bench]
fn bench_large_input(bencher: Bencher) {
    bencher
        .with_inputs(LargeInput::new)
        .bench_values(|input| input.process());
}

#[codspeed_divan_compat::bench]
fn bench_large_input_local(bencher: Bencher) {
    let input = LargeInput::new();
    bencher.bench_local(|| input.process());
}

fn main() {
    codspeed_divan_compat::main();
}
