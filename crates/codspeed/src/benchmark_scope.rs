//! The benchmark being run, which the exported telemetry is attributed to.
//!
//! Without the `otel` feature, entering and exiting a scope does nothing.

/// Marks the start of the benchmark `uri`. Spans started until [`exit`] carry its URI.
///
/// Call it outside the measured section.
pub fn enter(uri: &str) {
    #[cfg(feature = "otel")]
    crate::otel::enter_benchmark(uri);
    #[cfg(not(feature = "otel"))]
    let _ = uri;
}

/// Marks the end of the current benchmark.
///
/// Call it outside the measured section.
pub fn exit() {
    #[cfg(feature = "otel")]
    crate::otel::exit_benchmark();
}
