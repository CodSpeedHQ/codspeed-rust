//! Export of the `tracing` spans and events emitted by benchmarks over OTLP/HTTP, as traces and
//! logs respectively.
//!
//! Everything is a no-op unless `CODSPEED_OTEL_ENDPOINT` is set.
//!
//! Ended spans and log records are exported in batches from a background thread. Telemetry is
//! collected in a run of its own, whose performance is not measured, so exporting while a
//! benchmark runs doesn't matter.
//!
//! Spans and log records keep the wall clock times of the SDK. With instrument hooks, they also
//! carry their times on the clock of the benchmark markers: spans as their
//! `codspeed.monotonic_start_ns` and `codspeed.monotonic_end_ns` attributes, log records as their
//! `codspeed.monotonic_time_ns` attribute.
//!
//! Only the telemetry builds of `cargo-codspeed` compile the export: the others get a stand-in
//! exporting nothing.

#[cfg(codspeed_telemetry)]
mod export;
#[cfg(codspeed_telemetry)]
mod monotonic;
#[cfg(all(test, codspeed_telemetry))]
mod tests;
#[cfg(codspeed_telemetry)]
mod warmup;
#[cfg(codspeed_telemetry)]
pub use export::{init, layer, TelemetryGuard};

#[cfg(not(codspeed_telemetry))]
mod disabled;
#[cfg(not(codspeed_telemetry))]
pub use disabled::{init, layer, TelemetryGuard};
