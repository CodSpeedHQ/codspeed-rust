use crate::{measurement_mode::MeasurementMode, prelude::*, run::run_benches};
use cargo_metadata::MetadataCommand;
use clap::{Args, Parser, Subcommand};
use std::{ffi::OsString, process::exit};

use crate::build::build_benches;

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

#[derive(Args)]
pub(crate) struct PackageFilters {
    /// Select all packages in the workspace
    #[arg(long)]
    pub(crate) workspace: bool,
    /// Exclude packages
    #[arg(long)]
    pub(crate) exclude: Vec<String>,
    /// Package to select (builds all workspace package by default)
    #[arg(short, long)]
    pub(crate) package: Vec<String>,
}

#[derive(Args)]
pub(crate) struct Filters {
    /// Optional list of benchmarks to build (builds all benchmarks by default)
    pub(crate) bench: Option<Vec<String>>,
    #[command(flatten)]
    pub(crate) package: PackageFilters,
}

#[derive(Subcommand)]
enum Commands {
    /// Build the benchmarks
    Build {
        #[command(flatten)]
        filters: Filters,

        /// Space or comma separated list of features to activate
        #[arg(short = 'F', long)]
        features: Option<String>,

        /// Activate all available features of all selected packages.
        #[arg(long)]
        all_features: bool,

        /// Do not activate the `default` feature of the selected packages.
        #[arg(long)]
        no_default_features: bool,

        /// Number of parallel jobs, defaults to # of CPUs.
        #[arg(short, long)]
        jobs: Option<u32>,

        /// Build the benchmarks with the specified profile
        #[arg(long, default_value = "release")]
        profile: String,
    },
    /// Run the previously built benchmarks
    Run {
        #[command(flatten)]
        filters: Filters,
    },
}

pub fn run(args: impl Iterator<Item = OsString>) -> Result<()> {
    let metadata = MetadataCommand::new().exec()?;
    let cli = Cli::try_parse_from(args)?;

    let measurement_mode = cli.measurement_mode.unwrap_or_default();
    eprintln!("[cargo-codspeed] Measurement mode: {measurement_mode:?}\n");

    let res = match cli.command {
        Commands::Build {
            filters,
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
                filters,
                features,
                profile,
                cli.quiet,
                measurement_mode,
                passthrough_flags,
            )
        }
        Commands::Run { filters } => run_benches(&metadata, filters, measurement_mode),
    };

    if let Err(e) = res {
        eprintln!("Error: {e}");
        exit(1);
    }

    Ok(())
}
