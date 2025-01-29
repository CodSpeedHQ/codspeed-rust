use clap::ValueEnum;
use serde::Serialize;
use std::env;

#[derive(Debug, Clone, ValueEnum, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MeasurementMode {
    Walltime,
    Instrumentation,
}

impl Default for MeasurementMode {
    fn default() -> Self {
        if env::var("CODSPEED_ENV").is_ok() {
            MeasurementMode::Instrumentation
        } else {
            MeasurementMode::Walltime
        }
    }
}
