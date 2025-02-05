use clap::ValueEnum;
use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, Copy, ValueEnum, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum MeasurementMode {
    #[default]
    Instrumentation,
    Walltime,
}

impl fmt::Display for MeasurementMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                MeasurementMode::Instrumentation => "instrumentation",
                MeasurementMode::Walltime => "walltime",
            }
        )
    }
}
