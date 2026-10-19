use clap::ValueEnum;
use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BuildMode {
    #[default]
    Analysis,
    Walltime,
    /// Runs each benchmark like [`BuildMode::Analysis`], to collect the telemetry it emits rather
    /// than measure it
    Telemetry,
}

impl BuildMode {
    /// Whether the benchmarks run once, behind `cfg(codspeed)`, rather than in a walltime loop
    pub fn runs_once(self) -> bool {
        matches!(self, BuildMode::Analysis | BuildMode::Telemetry)
    }
}

impl fmt::Display for BuildMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BuildMode::Analysis => "analysis",
                BuildMode::Walltime => "walltime",
                BuildMode::Telemetry => "telemetry",
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
    /// Collects the telemetry the benchmarks emit instead of measuring them
    Telemetry,
}

impl From<MeasurementMode> for BuildMode {
    fn from(measurement_mode: MeasurementMode) -> Self {
        match measurement_mode {
            MeasurementMode::Simulation | MeasurementMode::Memory => BuildMode::Analysis,
            MeasurementMode::Walltime => BuildMode::Walltime,
            MeasurementMode::Telemetry => BuildMode::Telemetry,
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
                MeasurementMode::Telemetry => "telemetry",
            }
        )
    }
}
