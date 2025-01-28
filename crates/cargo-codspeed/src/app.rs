use crate::{prelude::*, run::run_benches};
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

    let res = match cli.command {
        Commands::Build {
            filters,
            features,
            profile,
        } => {
            let features =
                features.map(|f| f.split([' ', ',']).map(|s| s.to_string()).collect_vec());
            build_benches(&metadata, filters, features, profile, cli.quiet)
        }
        Commands::Run { filters } => run_benches(&metadata, filters),
    };

    if let Err(e) = res {
        eprintln!("Error: {e}");
        exit(1);
    }

    Ok(())
}
