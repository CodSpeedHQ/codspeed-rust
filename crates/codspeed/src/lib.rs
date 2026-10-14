pub mod codspeed;

pub mod instrument_hooks;

mod macros;
mod measurement;
#[cfg(feature = "otel")]
pub mod otel;
mod request;
pub mod utils;
pub mod walltime_results;
