use crate::{
    measurement_mode::{BuildMode, MeasurementMode},
    prelude::*,
    run::run_benches,
};
use cargo_metadata::MetadataCommand;
use clap::{ArgAction, Args, Parser, Subcommand};
use std::{ffi::OsString, process::exit};

use crate::build::{build_benches, BuildConfig};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Do not print cargo log messages
    #[arg(short, long, global = true)]
    quiet: bool,

    #[command(subcommand)]
    command: Commands,
}

impl Cli {
    pub fn run(self) -> Result<()> {
        let metadata = MetadataCommand::new().exec()?;
        match self.command {
            Commands::Build {
                package_filters,
                bench_target_filters,
                features,
                all_features,
                jobs,
                no_default_features,
                profile,
                locked,
                offline,
                frozen,
                measurement_mode,
            } => {
                let passthrough_flags = {
                    let mut passthrough_flags = Vec::new();
                    if all_features {
                        passthrough_flags.push("--all-features".to_string());
                    }
                    if no_default_features {
                        passthrough_flags.push("--no-default-features".to_string());
                    }
                    if locked {
                        passthrough_flags.push("--locked".to_string());
                    }
                    if offline {
                        passthrough_flags.push("--offline".to_string());
                    }
                    if frozen {
                        passthrough_flags.push("--frozen".to_string());
                    }
                    if let Some(jobs) = jobs {
                        passthrough_flags.push(format!("--jobs={jobs}"));
                    }
                    passthrough_flags
                };
                let features =
                    features.map(|f| f.split([' ', ',']).map(|s| s.to_string()).collect_vec());

                let modes = measurement_mode.iter().map(|m| m.to_string()).join(", ");
                eprintln!(
                    "[cargo-codspeed] Measurement mode{}: {modes}\n",
                    if measurement_mode.len() > 1 { "s" } else { "" }
                );

                let build_modes: Vec<BuildMode> = measurement_mode
                    .into_iter()
                    .map(BuildMode::from)
                    .unique()
                    .collect();
                let build_modes = if build_modes.is_empty() {
                    vec![BuildMode::default()]
                } else {
                    build_modes
                };

                for build_mode in build_modes {
                    build_benches(
                        &metadata,
                        BuildConfig {
                            package_filters: package_filters.clone(),
                            bench_target_filters: bench_target_filters.clone(),
                            features: features.clone(),
                            profile: profile.clone(),
                            quiet: self.quiet,
                            build_mode,
                            passthrough_flags: passthrough_flags.clone(),
                        },
                    )?;
                }
                Ok(())
            }
            Commands::Run {
                benchname,
                package_filters,
                bench_target_filters,
                measurement_mode,
            } => {
                let mode = measurement_mode.unwrap_or_default();
                eprintln!("[cargo-codspeed] Measurement mode: {mode:?}\n");
                run_benches(
                    &metadata,
                    benchname,
                    package_filters,
                    bench_target_filters,
                    mode,
                )
            }
        }
    }
}

const PACKAGE_HELP: &str = "Package Selection";
#[derive(Args, Clone)]
pub(crate) struct PackageFilters {
    /// Select all packages in the workspace
    #[arg(long, help_heading = PACKAGE_HELP)]
    pub(crate) workspace: bool,
    /// Package to exclude
    #[arg(long, value_name = "SPEC", help_heading = PACKAGE_HELP)]
    pub(crate) exclude: Vec<String>,
    /// Package to select
    #[arg(short, long, value_name= "SPEC", help_heading = PACKAGE_HELP)]
    pub(crate) package: Vec<String>,
}

#[derive(Args, Clone)]
pub(crate) struct BenchTargetFilters {
    /// Select only the specified benchmark target (all benchmark targets by default)
    #[arg(long, help_heading = TARGET_HELP)]
    pub(crate) bench: Option<Vec<String>>,
}

// Help headings, should mostly match the headers from cargo build --help
const FEATURE_HELP: &str = "Feature Selection";
const COMPILATION_HELP: &str = "Compilation Options";
const TARGET_HELP: &str = "Target Selection";
const MANIFEST_HELP: &str = "Manifest Options";

#[derive(Subcommand)]
enum Commands {
    /// Build the benchmarks
    Build {
        #[command(flatten)]
        package_filters: PackageFilters,

        /// Space or comma separated list of features to activate
        #[arg(short = 'F', long, help_heading = FEATURE_HELP)]
        features: Option<String>,

        /// Activate all available features of all selected packages.
        #[arg(long, help_heading = FEATURE_HELP)]
        all_features: bool,

        /// Do not activate the `default` feature of the selected packages.
        #[arg(long, help_heading = FEATURE_HELP)]
        no_default_features: bool,

        /// Number of parallel jobs, defaults to # of CPUs.
        #[arg(short, long, help_heading = COMPILATION_HELP)]
        jobs: Option<u32>,

        /// Build the benchmarks with the specified profile
        #[arg(long, default_value = "bench", help_heading = COMPILATION_HELP)]
        profile: String,

        /// Assert that `Cargo.lock` will remain unchanged
        #[arg(long, help_heading = MANIFEST_HELP)]
        locked: bool,

        /// Run without accessing the network
        #[arg(long, help_heading = MANIFEST_HELP)]
        offline: bool,

        /// Equivalent to specifying both --locked and --offline
        #[arg(long, help_heading = MANIFEST_HELP)]
        frozen: bool,

        #[command(flatten)]
        bench_target_filters: BenchTargetFilters,

        /// The measurement tool(s) to use for measuring performance.
        /// Can be specified multiple times or comma-separated.
        #[arg(
            short = 'm',
            long = "measurement-mode",
            value_delimiter = ',',
            action = ArgAction::Append,
            help_heading = COMPILATION_HELP,
            env = "CODSPEED_RUNNER_MODE"
        )]
        measurement_mode: Vec<MeasurementMode>,
    },
    /// Run the previously built benchmarks
    Run {
        /// If specified, only run benches containing this string in their names
        benchname: Option<String>,

        #[command(flatten)]
        package_filters: PackageFilters,

        #[command(flatten)]
        bench_target_filters: BenchTargetFilters,

        /// The measurement tool to use for measuring performance.
        /// Automatically set to `walltime` on macro runners
        #[arg(short = 'm', long = "measurement-mode", env = "CODSPEED_RUNNER_MODE")]
        measurement_mode: Option<MeasurementMode>,
    },
}

pub fn run(args: impl Iterator<Item = OsString>) -> Result<()> {
    let cli = Cli::try_parse_from(args)?;
    if let Err(e) = cli.run() {
        eprintln!("Error: {e}");
        exit(1);
    }

    Ok(())
}
