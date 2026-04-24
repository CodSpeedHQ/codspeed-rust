use std::hint::black_box;

#[inline(never)]
fn path_a() -> u32 {
    let start = std::time::Instant::now();
    while start.elapsed() < std::time::Duration::from_millis(1) {
        std::hint::spin_loop();
    }
    black_box(1)
}

#[inline(never)]
fn is_prime(n: u32) -> bool {
    if n < 2 {
        return false;
    }
    let mut i = 2;
    while i * i <= n {
        if n % i == 0 {
            return false;
        }
        i += 1;
    }
    true
}

#[inline(never)]
fn path_b() -> u32 {
    let mut sum = 0u32;
    for n in 2..black_box(20_000) {
        if is_prime(n) {
            sum = sum.wrapping_add(n);
        }
    }
    black_box(sum)
}

#[inline(never)]
fn shared(take_a: bool) -> u32 {
    let start = std::time::Instant::now();
    while start.elapsed() < std::time::Duration::from_millis(1) {
        std::hint::spin_loop();
    }
    if take_a {
        path_a()
    } else {
        path_b()
    }
}

#[inline(never)]
fn parent_a() {
    black_box(shared(black_box(true)));
}

#[inline(never)]
fn parent_b() {
    black_box(shared(black_box(false)));
}

#[codspeed_divan_compat::bench]
fn branching_bench() {
    parent_a();
    parent_b();
    codspeed_divan_compat::black_box(())
}

fn main() {
    codspeed_divan_compat::main();
}
