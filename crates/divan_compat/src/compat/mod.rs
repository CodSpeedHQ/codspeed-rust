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
mod uri;
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
    // We do not support finer filtering that bench targets for now, do nothing here, bench
    // filtering is managed by the `cargo-codspeed` wrappers before we reach this point.

    // 4. Scan the tree and execute benchmarks
    for entry in bench_entries.iter() {
        let entry_uri = uri::generate(entry.meta.display_name, &entry.meta);

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
