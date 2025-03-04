// Used by generated code from the macro `codspeed_divan_compat_macros::bench_compat`
#[doc(hidden)]
pub mod __private {
    pub use super::{
        bench::{BenchArgs, BenchOptions},
        entry::{
            BenchEntry, BenchEntryRunner, EntryConst, EntryList, EntryLocation, EntryMeta,
            EntryType, GenericBenchEntry, GroupEntry, BENCH_ENTRIES, GROUP_ENTRIES,
        },
    };

    pub use divan::__private::{shrink_array, Arg, ToStringHelper};
}

mod bench;
mod entry;
mod uri;
mod util;

use std::{cell::RefCell, rc::Rc};

pub use bench::*;
use codspeed::codspeed::CodSpeed;
use entry::AnyBenchEntry;

pub fn main() {
    // Outlined steps of original divan::main and their equivalent in codspeed instrumented mode
    // 1. Get registered entries
    let group_entries = &entry::GROUP_ENTRIES;

    let generic_bench_entries = group_entries.iter().flat_map(|group| {
        group
            .generic_benches_iter()
            .map(AnyBenchEntry::GenericBench)
    });

    let bench_entries = entry::BENCH_ENTRIES
        .iter()
        .map(AnyBenchEntry::Bench)
        .chain(generic_bench_entries);

    // TODO: Manage non generic bench groups

    // 2. Build an execution tree
    // No need, we do not manage detailed tree printing like original divan, and we extract
    // codspeed URI from entry metadata directly.

    // 3. Filtering
    // We do not support finer filtering that bench targets for now, do nothing here, bench
    // filtering is managed by the `cargo-codspeed` wrappers before we reach this point.

    // 4. Scan the tree and execute benchmarks
    let codspeed = Rc::new(RefCell::new(CodSpeed::new()));
    for entry in bench_entries {
        let runner = entry.bench_runner();
        let meta = entry.meta();

        if let Some(options) = &meta.bench_options {
            if let Some(true) = options.ignore {
                let uri = uri::generate(&entry, entry.display_name());
                println!("Skipped: {}", uri);
                continue;
            }
        }
        match runner {
            entry::BenchEntryRunner::Plain(bench_fn) => {
                let uri = uri::generate(&entry, entry.display_name());

                bench_fn(bench::Bencher::new(&codspeed, uri));
            }
            entry::BenchEntryRunner::Args(bench_runner) => {
                let bench_runner = bench_runner();

                for (arg_index, arg_name) in bench_runner.arg_names().iter().enumerate() {
                    let uri = uri::generate(&entry, arg_name);

                    let bencher = bench::Bencher::new(&codspeed, uri);

                    bench_runner.bench(bencher, arg_index);
                }
            }
        }
    }
}
