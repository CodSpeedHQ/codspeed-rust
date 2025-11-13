use clap::ValueEnum;
use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, Copy, ValueEnum, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum MeasurementMode {
    #[default]
    #[value(alias = "instrumentation")]
    Simulation,
    Walltime,
}

impl fmt::Display for MeasurementMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                MeasurementMode::Simulation => "simulation",
                MeasurementMode::Walltime => "walltime",
            }
        )
    }
}
