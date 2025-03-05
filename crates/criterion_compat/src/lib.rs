pub use codspeed::codspeed_uri;

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
        PlotConfiguration, PlottingBackend, SamplingMode, Throughput,
    };
}

pub use compat_criterion::*;
