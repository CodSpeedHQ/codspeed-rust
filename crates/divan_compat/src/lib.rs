pub use codspeed::codspeed_uri;
#[cfg(feature = "otel")]
pub use codspeed::otel;

#[cfg(not(codspeed))]
mod compat_divan {
    pub use divan::*;
}

#[cfg(codspeed)]
#[path = "."]
mod compat_divan {
    pub use divan::{black_box, black_box_drop, AllocProfiler};

    pub use codspeed_divan_compat_macros::bench_compat as bench;
    // Important: Keep in sync with the name used in the compat macro
    pub use divan::bench as bench_original;

    mod compat;
    pub use compat::*;
}

pub use compat_divan::*;

/// Runs all registered benchmarks.
///
/// With the `otel` feature, spans and events are exported for the duration of the run.
pub fn main() {
    #[cfg(feature = "otel")]
    let _otel_guard = otel::init();

    compat_divan::main();
}
