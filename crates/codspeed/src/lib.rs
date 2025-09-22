pub mod codspeed;

#[cfg(unix)]
pub mod instrument_hooks;

mod macros;
mod measurement;
mod request;
pub mod utils;
pub mod walltime_results;
