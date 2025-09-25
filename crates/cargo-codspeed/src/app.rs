use crate::{measurement_mode::MeasurementMode, prelude::*, run::run_benches};
use cargo_metadata::MetadataCommand;
use clap::{Args, Parser, Subcommand};
use std::{ffi::OsString, process::exit};

use crate::build::{build_benches, BuildConfig};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Do not print cargo log messages
    #[arg(short, long, global = true)]
    quiet: bool,

    /// The measurement tool to use for measuring performance.
    /// Automatically set to `walltime` on macro runners
    #[arg(short, long, global = true, env = "CODSPEED_RUNNER_MODE")]
    measurement_mode: Option<MeasurementMode>,

    #[command(subcommand)]
    command: Commands,
}

const PACKAGE_HELP: &str = "Package Selection";
#[derive(Args)]
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

#[derive(Args)]
pub(crate) struct BenchTargetFilters {
    /// Select only the specified benchmark target (all benchmark targets by default)
    #[arg(long, help_heading = TARGET_HELP)]
    pub(crate) bench: Option<Vec<String>>,
}

const FEATURE_HELP: &str = "Feature Selection";
const COMPILATION_HELP: &str = "Compilation Options";
const TARGET_HELP: &str = "Target Selection";
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

        #[command(flatten)]
        bench_target_filters: BenchTargetFilters,
    },
    /// Run the previously built benchmarks
    Run {
        /// If specified, only run benches containing this string in their names
        benchname: Option<String>,

        #[command(flatten)]
        package_filters: PackageFilters,

        #[command(flatten)]
        bench_target_filters: BenchTargetFilters,
    },
}

pub fn run(args: impl Iterator<Item = OsString>) -> Result<()> {
    let metadata = MetadataCommand::new().exec()?;
    let cli = Cli::try_parse_from(args)?;

    let measurement_mode = cli.measurement_mode.unwrap_or_default();
    eprintln!("[cargo-codspeed] Measurement mode: {measurement_mode:?}\n");

    let res = match cli.command {
        Commands::Build {
            package_filters,
            bench_target_filters,
            features,
            all_features,
            jobs,
            no_default_features,
            profile,
        } => {
            let passthrough_flags = {
                let mut passthrough_flags = Vec::new();

                if all_features {
                    passthrough_flags.push("--all-features".to_string());
                }

                if no_default_features {
                    passthrough_flags.push("--no-default-features".to_string());
                }

                if let Some(jobs) = jobs {
                    passthrough_flags.push(format!("--jobs={jobs}"));
                }

                passthrough_flags
            };
            let features =
                features.map(|f| f.split([' ', ',']).map(|s| s.to_string()).collect_vec());
            build_benches(
                &metadata,
                BuildConfig {
                    package_filters,
                    bench_target_filters,
                    features,
                    profile,
                    quiet: cli.quiet,
                    measurement_mode,
                    passthrough_flags,
                },
            )
        }
        Commands::Run {
            benchname,
            package_filters,
            bench_target_filters,
        } => run_benches(
            &metadata,
            benchname,
            package_filters,
            bench_target_filters,
            measurement_mode,
        ),
    };

    if let Err(e) = res {
        eprintln!("Error: {e}");
        exit(1);
    }

    Ok(())
}
