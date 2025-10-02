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
mod cli;
mod config;
mod entry;
mod uri;
mod util;

pub use bench::*;
use codspeed::codspeed::CodSpeed;
use config::Filter;
use entry::AnyBenchEntry;
use regex::Regex;
use std::{cell::RefCell, rc::Rc};

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
    let should_run_benchmark_from_filters = {
        let mut command = cli::command();
        let matches = command.get_matches_mut();
        let is_exact = matches.get_flag("exact");

        let parse_filter = |filter: &String| {
            if is_exact {
                Filter::Exact(filter.to_owned())
            } else {
                match Regex::new(filter) {
                    Ok(r) => Filter::Regex(r),
                    Err(error) => {
                        let kind = clap::error::ErrorKind::ValueValidation;
                        command.error(kind, error).exit();
                    }
                }
            }
        };

        let filters: Option<Vec<Filter>> = matches
            .get_many::<String>("filter")
            .map(|arg_filters| arg_filters.map(parse_filter).collect());

        move |uri: &str| {
            if let Some(filters) = filters.as_ref() {
                filters.iter().any(|filter| filter.is_match(uri))
            } else {
                true
            }
        }
    };

    // 4. Scan the tree and execute benchmarks
    let codspeed = Rc::new(RefCell::new(CodSpeed::new()));
    for entry in bench_entries {
        let runner = entry.bench_runner();
        let meta = entry.meta();

        if let Some(options) = &meta.bench_options {
            if let Some(true) = options.ignore {
                let uri = uri::generate(&entry, entry.display_name());
                println!("Skipped: {uri}");
                continue;
            }
        }
        match runner {
            entry::BenchEntryRunner::Plain(bench_fn) => {
                let uri = uri::generate(&entry, entry.display_name());

                if !should_run_benchmark_from_filters(&uri) {
                    continue;
                }

                bench_fn(bench::Bencher::new(&codspeed, uri));
            }
            entry::BenchEntryRunner::Args(bench_runner) => {
                let bench_runner = bench_runner();

                for (arg_index, arg_name) in bench_runner.arg_names().iter().enumerate() {
                    let uri = uri::generate(&entry, arg_name);

                    if !should_run_benchmark_from_filters(&uri) {
                        continue;
                    }

                    let bencher = bench::Bencher::new(&codspeed, uri);

                    bench_runner.bench(bencher, arg_index);
                }
            }
        }
    }
}
