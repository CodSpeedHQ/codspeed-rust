fn main() {
    divan::main();
}

#[divan::bench]
fn sleep_1ns() {
    std::thread::sleep(std::time::Duration::from_nanos(1));
}

#[divan::bench]
fn sleep_100ns() {
    std::thread::sleep(std::time::Duration::from_nanos(100));
}

#[divan::bench]
fn sleep_1us() {
    std::thread::sleep(std::time::Duration::from_micros(1));
}

#[divan::bench]
fn sleep_100us() {
    std::thread::sleep(std::time::Duration::from_micros(100));
}

#[divan::bench]
fn sleep_1ms() {
    std::thread::sleep(std::time::Duration::from_millis(1));
}
