use clap::ValueEnum;
use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildMode {
    Analysis,
    Walltime,
}

impl fmt::Display for BuildMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BuildMode::Analysis => "analysis",
                BuildMode::Walltime => "walltime",
            }
        )
    }
}

#[derive(Debug, Clone, Copy, ValueEnum, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum MeasurementMode {
    #[default]
    #[value(alias = "instrumentation")]
    Simulation,
    Walltime,
    Memory,
}

impl From<MeasurementMode> for BuildMode {
    fn from(measurement_mode: MeasurementMode) -> Self {
        match measurement_mode {
            MeasurementMode::Simulation | MeasurementMode::Memory => BuildMode::Analysis,
            MeasurementMode::Walltime => BuildMode::Walltime,
        }
    }
}

impl fmt::Display for MeasurementMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                MeasurementMode::Simulation => "simulation",
                MeasurementMode::Walltime => "walltime",
                MeasurementMode::Memory => "memory",
            }
        )
    }
}
