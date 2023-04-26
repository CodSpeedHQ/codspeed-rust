use std::{ffi::OsString, process::exit};

use crate::{prelude::*, run::run_benches};

use cargo::util::important_paths::find_root_manifest_for_wd;
use cargo::Config;
use clap::{Parser, Subcommand};
use termcolor::Color;

use crate::build::build_benches;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build the benchmarks
    Build {
        /// Optional list of benchmarks to build (builds all benchmarks by default)
        benches: Option<Vec<String>>,
        /// Package to build benchmarks for (if using a workspace)
        #[arg(short, long)]
        package: Option<String>,
        /// Space or comma separated list of features to activate
        #[arg(short = 'F', long)]
        features: Option<String>,
    },
    /// Run the previously built benchmarks
    Run {
        /// Optional list of benchmarks to run (run all found benchmarks by default)
        benches: Option<Vec<String>>,
        /// Package to build benchmarks for (if using a workspace)
        #[arg(short, long)]
        package: Option<String>,
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
    let workspace = Workspace::new(&manifest_path, &cargo_config)?;

    let res = match cli.command {
        Commands::Build {
            benches,
            package,
            features,
        } => {
            let features = features.map(|f| {
                f.split(|c| c == ' ' || c == ',')
                    .map(|s| s.to_string())
                    .collect_vec()
            });
            build_benches(&workspace, benches, package, features)
        }
        Commands::Run { benches, package } => run_benches(&workspace, benches, package),
    };

    if let Err(e) = res {
        workspace
            .config()
            .shell()
            .status_with_color("Error", e.to_string(), Color::Red)?;
        exit(1);
    }

    Ok(())
}
