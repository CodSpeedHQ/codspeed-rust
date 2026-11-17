#[cfg(feature = "telemetry")]
pub use codspeed::telemetry;

#[cfg(not(codspeed))]
mod compat_bencher {
    pub use bencher::*;
}

#[cfg(codspeed)]
#[path = "."]
mod compat_bencher {
    pub use codspeed::abs_file;

    mod compat;
    pub use compat::*;
}

pub use compat_bencher::*;

/// Installs the telemetry export for the `main` of `benchmark_main!`, with the `telemetry`
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
