//! Benchmarks emitting telemetry in different ways: `tracing` spans, or nothing at all.

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
#[divan::bench]
fn parse_and_sum() -> u32 {
    let _span = info_span!("parse_and_sum").entered();
    let values = parse("1,2,x,3");
    if values.len() < 4 {
        warn!(skipped = 4 - values.len(), "skipped invalid fields");
    }
    sum(&values)
}

/// No telemetry at all.
#[divan::bench]
fn no_telemetry() -> u32 {
    std::hint::black_box([1, 2, 3]).iter().sum()
}

fn main() {
    divan::main();
}
