#[codspeed_divan_compat::bench]
fn sleep_1ms() {
    std::thread::sleep(std::time::Duration::from_millis(1));
}

#[codspeed_divan_compat::bench]
fn sleep_10ms() {
    std::thread::sleep(std::time::Duration::from_millis(10));
}

#[codspeed_divan_compat::bench]
fn sleep_50ms() {
    std::thread::sleep(std::time::Duration::from_millis(50));
}

#[codspeed_divan_compat::bench]
fn sleep_100ms() {
    std::thread::sleep(std::time::Duration::from_millis(100));
}

// Tests COD-1044, do not modify the sample size or count!
#[codspeed_divan_compat::bench(sample_size = 3, sample_count = 6)]
fn sleep_100ms_with_custom_sample() {
    std::thread::sleep(std::time::Duration::from_millis(100));
}

fn main() {
    codspeed_divan_compat::main();
}
