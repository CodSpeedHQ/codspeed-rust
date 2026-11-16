pub use codspeed::codspeed_uri;
#[cfg(feature = "telemetry")]
pub use codspeed::telemetry;

#[cfg(not(codspeed))]
mod compat_criterion {
    pub use codspeed::abs_file;
    pub use criterion::*;
}

#[cfg(codspeed)]
#[path = "."]
mod compat_criterion {
    pub use codspeed::abs_file;

    mod compat;
    pub use compat::*;

    pub use criterion::{
        async_executor, black_box, measurement, profiler, AxisScale, Baseline, BatchSize,
        IterManualOptions, PlotConfiguration, PlottingBackend, SamplingMode, Throughput,
    };
}

pub use compat_criterion::*;

/// Installs the telemetry export for the `main` of `criterion_main!`, with the `telemetry`
/// feature. The macro expands in the user's crate, so it can't check the feature itself.
///
/// Keep the returned guard alive until the benchmarks are done.
#[doc(hidden)]
#[cfg(feature = "telemetry")]
pub fn __init_telemetry() -> Option<telemetry::TelemetryGuard> {
    telemetry::init()
}

#[doc(hidden)]
#[cfg(not(feature = "telemetry"))]
pub fn __init_telemetry() -> Option<()> {
    None
}
