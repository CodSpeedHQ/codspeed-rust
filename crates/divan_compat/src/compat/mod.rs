// Used by generated code. Not public API and thus not subject to SemVer.
#[doc(hidden)]
#[path = "private.rs"]
pub mod __private;

mod bench;
mod entry;
mod util;

pub use bench::*;

pub fn main() {
    // Outlined steps of original divan::main and their equivalent in codspeed instrumented mode
    // 1. Get registered entries
    // TODO: Manage bench groups
    let bench_entries = &entry::BENCH_ENTRIES;

    // 2. Build an execution tree
    // No need, we do not manage detailed tree printing like original divan, and we extract
    // codspeed URI from entry metadata directly.

    // 3. Filtering
    // No need, filtering is managed at a higher level in the `cargo-codspeed` wrapper.

    // 4. Scan the tree and execute benchmarks
    for entry in bench_entries.iter() {
        let entry_uri = format!(
            "{}:{}::{}::{}",
            entry.meta.location.file,
            entry.meta.location.line,
            entry.meta.module_path,
            entry.meta.display_name,
        );

        if let Some(options) = &entry.meta.bench_options.as_ref() {
            if let Some(true) = options.ignore {
                println!("Skipped: {}", entry_uri);
                continue;
            }
        }
        match entry.bench {
            entry::BenchEntryRunner::Plain(bench_fn) => {
                bench_fn(bench::Bencher::new(entry_uri));
            }
            entry::BenchEntryRunner::Args(bench_runner) => {
                let bench_runner = bench_runner();

                for (arg_index, arg_name) in bench_runner.arg_names().iter().enumerate() {
                    let entry_name_with_arg = format!("{}::{}", entry_uri, arg_name);
                    let bencher = bench::Bencher::new(entry_name_with_arg);

                    bench_runner.bench(bencher, arg_index);
                }
            }
        }
    }
}
