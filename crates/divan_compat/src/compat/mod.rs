// Used by generated code. Not public API and thus not subject to SemVer.
#[doc(hidden)]
#[path = "private.rs"]
pub mod __private;

mod bench;
mod entry;

pub fn main() {
    // 1. Get registered entries
    // TODO: Manage bench groups
    let bench_entries = &entry::BENCH_ENTRIES;

    // 2. Build an execution tree
    // TODO:

    // 3. Filter the tree then sort it (drop sort?)
    // TODO:

    // 4. Scan the tree and execute benchmarks
    // TODO:
    for entry in bench_entries.iter() {
        match entry.bench {
            entry::BenchEntryRunner::Plain(bench_fn) => {
                bench_fn(bench::Bencher::new(format!(
                    "{}:{}::{}::{}",
                    entry.meta.location.file,
                    entry.meta.location.line,
                    entry.meta.module_path,
                    entry.meta.display_name
                )));
            }
        }
    }
}
