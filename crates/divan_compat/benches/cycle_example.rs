use std::hint::black_box;

#[inline(never)]
fn work(seed: u32, iters: u32) -> u32 {
    let mut acc = black_box(seed);
    for i in 0..iters {
        acc = acc.wrapping_mul(1664525).wrapping_add(1013904223 ^ i);
        acc ^= acc.rotate_left(13);
    }
    black_box(acc)
}

#[inline(never)]
fn cycle_a(depth: u32, seed: u32) -> u32 {
    let local = work(seed, 2_000);
    if depth == 0 {
        return local;
    }
    black_box(cycle_b(depth - 1, local).wrapping_add(local))
}

#[inline(never)]
fn cycle_b(depth: u32, seed: u32) -> u32 {
    let local = work(seed, 2_000);
    if depth == 0 {
        return local;
    }
    black_box(cycle_c(depth - 1, local).wrapping_add(local))
}

#[inline(never)]
fn cycle_c(depth: u32, seed: u32) -> u32 {
    let local = work(seed, 2_000);
    if depth == 0 {
        return local;
    }
    black_box(cycle_a(depth - 1, local).wrapping_add(local))
}

const DEPTH: u32 = 299;
#[codspeed_divan_compat::bench]
fn cycle_bench() {
    black_box(cycle_a(black_box(DEPTH), black_box(0xC0DE_5EEDu32)));
    codspeed_divan_compat::black_box(())
}

fn main() {
    codspeed_divan_compat::main();
}
