#[divan::bench]
fn panics() {
    panic!("benchmark failed");
}

fn main() {
    divan::main();
}
