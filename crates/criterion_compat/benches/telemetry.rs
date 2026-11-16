//! Benchmarks emitting telemetry in different ways: `tracing` spans and events, records of the
//! `log` crate, or nothing at all.

use codspeed_criterion_compat::{black_box, criterion_group, criterion_main, Criterion};
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
fn parse_and_sum() -> u32 {
    let _span = info_span!("parse_and_sum").entered();
    let values = parse("1,2,x,3");
    if values.len() < 4 {
        warn!(skipped = 4 - values.len(), "skipped invalid fields");
    }
    sum(&values)
}

/// A `tracing` event outside of any span.
fn event_without_span() -> u32 {
    let values = black_box([1, 2, 3]);
    info!(count = values.len(), "summed without a span");
    values.iter().sum()
}

/// Records of the `log` crate, without `tracing`.
fn log_records() -> u32 {
    let values = black_box([1, 2, 3]);
    log::info!("summing {} values, logged with the log crate", values.len());
    log::debug!("filtered out by the default filter");
    values.iter().sum()
}

/// No telemetry at all.
fn no_telemetry() -> u32 {
    black_box([1, 2, 3]).iter().sum()
}

fn telemetry(c: &mut Criterion) {
    c.bench_function("parse_and_sum", |b| b.iter(parse_and_sum));
    c.bench_function("event_without_span", |b| b.iter(event_without_span));
    c.bench_function("log_records", |b| b.iter(log_records));
    c.bench_function("no_telemetry", |b| b.iter(no_telemetry));
}

criterion_group!(benches, telemetry);
criterion_main!(benches);
