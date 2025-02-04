// Used by generated code from the macro `codspeed_divan_compat_macros::bench_compat`
#[doc(hidden)]
pub mod __private {
    pub use super::{
        bench::{BenchArgs, BenchOptions},
        entry::{BenchEntry, BenchEntryRunner, EntryList, EntryLocation, EntryMeta, BENCH_ENTRIES},
    };

    pub use divan::__private::{Arg, ToStringHelper};
}

mod bench;
mod entry;

pub fn main() {
    // 1. Get registered entries
    // TODO: Manage bench groups

    // TODO: remove when releasing divan with instrumentation mode
    todo!("Instrumentation mode with divan is not yet available.");
    #[allow(unreachable_code)]
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
