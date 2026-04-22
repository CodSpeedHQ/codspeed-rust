use std::sync::atomic::{AtomicBool, Ordering};

#[inline(never)]
fn busy_sleep(ms: u64) {
    let start = std::time::Instant::now();
    while start.elapsed() < std::time::Duration::from_millis(ms) {
        std::hint::spin_loop();
    }
}

#[inline(never)]
fn first_branch() -> u32 {
    busy_sleep(100);
    1
}

#[inline(never)]
fn second_branch() -> u32 {
    busy_sleep(200);
    2
}

#[inline(never)]
fn branch() -> u32 {
    static TOGGLE: AtomicBool = AtomicBool::new(false);
    let use_first = TOGGLE.fetch_xor(true, Ordering::Relaxed);

    if use_first {
        first_branch()
    } else {
        second_branch()
    }
}

#[codspeed_divan_compat::bench]
fn branching_bench() -> u32 {
    let mut result = 0;
    for _ in 0..20 {
        result = codspeed_divan_compat::black_box(branch());
    }
    result
}

fn main() {
    codspeed_divan_compat::main();
}
