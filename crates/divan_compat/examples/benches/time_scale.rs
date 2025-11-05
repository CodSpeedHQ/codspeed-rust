fn main() {
    divan::main();
}

fn busy_sleeping(duration: std::time::Duration) {
    let start = std::time::Instant::now();
    while start.elapsed() < duration {}
}

#[divan::bench]
fn sleep_1ns() {
    busy_sleeping(std::time::Duration::from_nanos(1));
}

#[divan::bench]
fn sleep_100ns() {
    busy_sleeping(std::time::Duration::from_nanos(100));
}

#[divan::bench]
fn sleep_1us() {
    busy_sleeping(std::time::Duration::from_micros(1));
}

#[divan::bench]
fn sleep_100us() {
    busy_sleeping(std::time::Duration::from_micros(100));
}

#[divan::bench]
fn sleep_1ms() {
    busy_sleeping(std::time::Duration::from_millis(1));
}
