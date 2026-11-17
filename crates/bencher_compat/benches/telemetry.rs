//! Benchmarks emitting telemetry in different ways: `tracing` spans and events, records of the
//! `log` crate, or nothing at all.

use codspeed::codspeed::black_box;
use codspeed_bencher_compat::{benchmark_group, benchmark_main, Bencher};
use tracing::{debug, debug_span, info, info_span, warn};

fn parse(input: &str) -> Vec<u32> {
    let _span = info_span!("parse", input_len = input.len()).entered();
    let values: Vec<u32> = input
        .split(',')
        .filter_map(|field| field.parse().ok())
        .collect();
    info!(count = values.len(), "parsed values");
    values
}

fn sum(values: &[u32]) -> u32 {
    let _span = info_span!("sum").entered();
    let _filtered_out = debug_span!("sum_inner").entered();
    debug!("filtered out by the default filter");
    values.iter().sum()
}

/// Nested `tracing` spans, with events in them.
pub fn parse_and_sum(bench: &mut Bencher) {
    bench.iter(|| {
        let _span = info_span!("parse_and_sum").entered();
        let values = parse("1,2,x,3");
        if values.len() < 4 {
            warn!(skipped = 4 - values.len(), "skipped invalid fields");
        }
        sum(&values)
    })
}

/// A `tracing` event outside of any span.
pub fn event_without_span(bench: &mut Bencher) {
    bench.iter(|| {
        let values = black_box([1, 2, 3]);
        info!(count = values.len(), "summed without a span");
        values.iter().sum::<u32>()
    })
}

/// Records of the `log` crate, without `tracing`.
pub fn log_records(bench: &mut Bencher) {
    bench.iter(|| {
        let values = black_box([1, 2, 3]);
        log::info!("summing {} values, logged with the log crate", values.len());
        log::debug!("filtered out by the default filter");
        values.iter().sum::<u32>()
    })
}

/// No telemetry at all.
pub fn no_telemetry(bench: &mut Bencher) {
    bench.iter(|| black_box([1, 2, 3]).iter().sum::<u32>())
}

benchmark_group!(
    benches,
    parse_and_sum,
    event_without_span,
    log_records,
    no_telemetry
);
benchmark_main!(benches);
