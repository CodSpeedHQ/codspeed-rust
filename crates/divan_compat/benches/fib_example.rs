fn fib(n: u32) -> u32 {
    match n {
        0 => 0,
        1 => 1,
        n => fib(n - 1) + fib(n - 2),
    }
}

#[codspeed_divan_compat::bench]
fn fib_30() -> u32 {
    codspeed_divan_compat::black_box(fib(30))
}

#[codspeed_divan_compat::bench]
fn fib_20() -> u32 {
    codspeed_divan_compat::black_box(fib(20))
}

#[codspeed_divan_compat::bench]
fn fib_10() -> u32 {
    codspeed_divan_compat::black_box(fib(10))
}

fn main() {
    codspeed_divan_compat::main();
}
