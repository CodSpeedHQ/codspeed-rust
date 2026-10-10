//! Identical benchmarks whose cost only depends on the allocator state left
//! behind by the benchmarks that ran before them in the same process.
//!
//! glibc serves `calloc` requests above `M_MMAP_THRESHOLD` (128 KiB by default)
//! with a fresh `mmap`, which the kernel already zeroed. Freeing such a chunk
//! raises the threshold to the chunk size, so the next allocation of the same
//! size comes from the heap and has to be cleared with `memset`.
//!
//! Simulation mode measures a single run per benchmark. With the default
//! process isolation, every benchmark runs in a fresh process and all of them
//! report the same cost. With `CODSPEED_PROCESS_ISOLATION=false`, they share
//! a process and the benchmarks after the first one pay for clearing the
//! buffer. Walltime mode repeats the routine, so its first iteration already
//! raises the threshold and later samples are similar either way.
//!
//! `spawn_on_parent_runtime` uses a multi-threaded Tokio runtime started in
//! `main`, before the harness runs any benchmark. Each isolated benchmark
//! process starts its own runtime, so awaiting a spawned task completes.

use std::sync::LazyLock;
use tokio::runtime::Runtime;

const BUFFER_SIZE: usize = 4 << 20;

#[codspeed_divan_compat::bench(args = [1, 2, 3])]
fn zeroed_buffer(_copy: usize) -> u8 {
    let buffer = vec![0u8; BUFFER_SIZE];
    codspeed_divan_compat::black_box(&buffer)[BUFFER_SIZE / 2]
}

/// Started in `main`, so its workers only exist in the parent process.
static RUNTIME: LazyLock<Runtime> =
    LazyLock::new(|| Runtime::new().expect("failed to start the Tokio runtime"));

#[codspeed_divan_compat::bench]
fn spawn_on_parent_runtime() -> u32 {
    RUNTIME.block_on(async { tokio::spawn(async { 1 }).await.unwrap() })
}

fn main() {
    LazyLock::force(&RUNTIME);
    codspeed_divan_compat::main();
}
