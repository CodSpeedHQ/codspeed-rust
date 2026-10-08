use divan::Bencher;
use std::sync::LazyLock;
use tokio::runtime::Runtime;

/// Started in `main`, before any benchmark, so its worker threads belong to
/// the process that runs the harness.
static RUNTIME: LazyLock<Runtime> =
    LazyLock::new(|| Runtime::new().expect("failed to start the Tokio runtime"));

#[divan::bench]
fn first(bencher: Bencher) {
    bencher.bench(|| divan::black_box(1));
}

#[divan::bench]
fn second(bencher: Bencher) {
    bencher.bench(|| divan::black_box(2));
}

#[divan::bench]
fn spawn_on_main_runtime(bencher: Bencher) {
    bencher.bench(|| RUNTIME.block_on(async { tokio::spawn(async { 3 }).await.unwrap() }));
}

fn main() {
    LazyLock::force(&RUNTIME);
    divan::main();
}
