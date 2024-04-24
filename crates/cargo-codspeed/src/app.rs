use std::{ffi::OsString, process::exit};

use crate::helpers::style;
use crate::{prelude::*, run::run_benches};

use cargo::Config;
use cargo::{ops::Packages, util::important_paths::find_root_manifest_for_wd};
use clap::{Args, Parser, Subcommand};

use crate::build::build_benches;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Package selection flags
#[derive(Args)]
struct PackageSelection {
    /// Select all packages in the workspace
    #[arg(long)]
    workspace: bool,
    /// Exclude packages
    #[arg(long)]
    exclude: Vec<String>,
    /// Package to select
    #[arg(short, long)]
    package: Vec<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Build the benchmarks
    Build {
        /// Optional list of benchmarks to build (builds all benchmarks by default)
        benches: Option<Vec<String>>,

        #[command(flatten)]
        package_selection: PackageSelection,

        /// Space or comma separated list of features to activate
        #[arg(short = 'F', long)]
        features: Option<String>,
    },
    /// Run the previously built benchmarks
    Run {
        /// Optional list of benchmarks to run (run all found benchmarks by default)
        benches: Option<Vec<String>>,

        #[command(flatten)]
        package_selection: PackageSelection,
    },
}

pub fn get_cargo_config() -> Result<Config> {
    let mut rustflags = std::env::var("RUSTFLAGS").unwrap_or_else(|_| "".into());
    rustflags.push_str(" -g --cfg codspeed");
    std::env::set_var("RUSTFLAGS", &rustflags);
    Config::default()
}

pub fn run(args: impl Iterator<Item = OsString>) -> Result<()> {
    let cli = Cli::try_parse_from(args)?;
    let cargo_config = get_cargo_config()?;
    let manifest_path = find_root_manifest_for_wd(cargo_config.cwd())?;
    let ws = Workspace::new(&manifest_path, &cargo_config)?;

    let res = match cli.command {
        Commands::Build {
            benches,
            package_selection,
            features,
        } => {
            let features = features.map(|f| {
                f.split(|c| c == ' ' || c == ',')
                    .map(|s| s.to_string())
                    .collect_vec()
            });
            let packages = Packages::from_flags(
                package_selection.workspace,
                package_selection.exclude,
                package_selection.package,
            )?;
            build_benches(&ws, benches, packages, features)
        }
        Commands::Run {
            benches,
            package_selection,
        } => {
            let packages = Packages::from_flags(
                package_selection.workspace,
                package_selection.exclude,
                package_selection.package,
            )?;
            run_benches(&ws, benches, packages)
        }
    };

    if let Err(e) = res {
        ws.config()
            .shell()
            .status_with_color("Error", e.to_string(), &style::ERROR)?;
        exit(1);
    }

    Ok(())
}
