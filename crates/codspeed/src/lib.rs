pub mod codspeed;

pub mod instrument_hooks;
pub mod isolation;

mod macros;
mod measurement;
mod request;
#[cfg(all(feature = "telemetry", codspeed_telemetry))]
pub mod telemetry;
#[cfg(all(feature = "telemetry", not(codspeed_telemetry)))]
#[path = "telemetry_disabled.rs"]
pub mod telemetry;
pub mod utils;
pub mod walltime_results;
