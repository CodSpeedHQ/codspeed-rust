fn main() {
    divan::main();
}

fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

#[divan::bench]
fn fib_20() {
    divan::black_box(fibonacci(divan::black_box(20)));
}
