use crate::{
    app::{Filters, PackageFilters},
    helpers::{clear_dir, get_codspeed_target_dir},
    measurement_mode::MeasurementMode,
    prelude::*,
};
use cargo_metadata::{camino::Utf8PathBuf, Message, Metadata, TargetKind};
use std::process::{exit, Command, Stdio};

struct BuildOptions<'a> {
    filters: Filters,
    features: &'a Option<Vec<String>>,
    profile: &'a str,
    passthrough_flags: &'a Vec<String>,
}

struct BuiltBench {
    package: String,
    bench: String,
    executable_path: Utf8PathBuf,
}

impl BuildOptions<'_> {
    /// Builds the benchmarks by invoking cargo
    /// Returns a list of built benchmarks, with path to associated executables
    fn build(
        &self,
        metadata: &Metadata,
        quiet: bool,
        measurement_mode: MeasurementMode,
    ) -> Result<Vec<BuiltBench>> {
        let workspace_packages = metadata.workspace_packages();

        let mut cargo = self.build_command(measurement_mode);
        if quiet {
            cargo.arg("--quiet");
        }
        cargo.args(["--message-format", "json"]);

        let mut cargo = cargo
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to spawn cargo command");
        let reader = std::io::BufReader::new(
            cargo
                .stdout
                .take()
                .expect("Unable to get stdout for child process"),
        );

        let mut built_benches = Vec::new();
        for message in Message::parse_stream(reader) {
            match message.expect("Failed to parse message") {
                // Those messages will include build errors and warnings even if stderr also contain some of them
                Message::CompilerMessage(msg) => {
                    println!("{}", &msg.message);
                }
                Message::TextLine(line) => {
                    println!("{line}");
                }
                Message::CompilerArtifact(artifact)
                    if artifact.target.is_kind(TargetKind::Bench) =>
                {
                    let package = workspace_packages
                        .iter()
                        .find(|p| p.id == artifact.package_id)
                        .expect("Could not find package");

                    let bench_name = artifact.target.name;

                    let add_bench_to_codspeed_dir = match &self.filters.bench {
                        Some(allowed_bench_names) => allowed_bench_names
                            .iter()
                            .any(|allowed_bench_name| bench_name.contains(allowed_bench_name)),
                        None => true,
                    };

                    if add_bench_to_codspeed_dir {
                        built_benches.push(BuiltBench {
                            package: package.name.clone(),
                            bench: bench_name,
                            executable_path: artifact
                                .executable
                                .expect("Unexpected missing executable path"),
                        });
                    }
                }
                _ => {}
            }
        }

        let status = cargo.wait().expect("Could not get cargo's exist status");

        if !status.success() {
            exit(status.code().expect("Could not get exit code"));
        }

        for built_bench in &built_benches {
            eprintln!(
                "Built benchmark `{}` in package `{}`",
                built_bench.bench, built_bench.package
            );
        }

        Ok(built_benches)
    }

    /// Adds debug flags and codspeed compilation
    ///
    /// If the user has set `RUSTFLAGS`, it will append the flags to it.
    /// Else, and if the cargo version allows it, it will set the cargo config through
    /// `--config 'build.rustflags=[ ... ]'`
    ///
    /// # Why we do this
    /// As tracked in [https://github.com/rust-lang/cargo/issues/5376], setting `RUSTFLAGS`
    /// completely overrides rustflags from cargo config
    /// We use the cargo built-in config mechanism to set the flags if the user has not set
    /// `RUSTFLAGS`.
    fn add_rust_flags(&self, cargo: &mut Command, measurement_mode: MeasurementMode) {
        let mut flags = vec![
            // Add debug info (equivalent to -g)
            "-Cdebuginfo=2".to_owned(),
            // Prevent debug info stripping
            // https://doc.rust-lang.org/cargo/reference/profiles.html#release
            // According to cargo docs, for release profile which we default to:
            // `strip = "none"` and `debug = false`.
            // In practice, if we set debug info through RUSTFLAGS, cargo still strips them, most
            // likely because debug = false in the release profile.
            // We also need to disable stripping through rust flags.
            "-Cstrip=none".to_owned(),
        ];

        // Add the codspeed cfg flag if instrumentation mode is enabled
        if measurement_mode == MeasurementMode::Instrumentation {
            flags.push("--cfg=codspeed".to_owned());
        }

        match std::env::var("RUSTFLAGS") {
            Result::Ok(existing_rustflags) => {
                // Expand already existing RUSTFLAGS env var
                let flags_str = flags.join(" ");
                cargo.env("RUSTFLAGS", format!("{existing_rustflags} {flags_str}"));
            }
            Err(_) => {
                // Use --config to set rustflags
                // Our rust integration has an msrv of 1.74, --config is available since 1.63
                // https://doc.rust-lang.org/nightly/cargo/CHANGELOG.html#cargo-163-2022-08-11
                // Note: We have to use `target.cfg(all())` since `build` has a lower precedence.
                let config_value = format!(
                    "target.'cfg(all())'.rustflags=[{}]",
                    flags.into_iter().map(|f| format!("\"{f}\"")).join(",")
                );
                cargo.arg("--config").arg(config_value);
            }
        }
    }

    /// Generates a subcommand to build the benchmarks by invoking cargo and forwarding the filters
    /// This command explicitly ignores the `self.benches`: all benches are built
    fn build_command(&self, measurement_mode: MeasurementMode) -> Command {
        let mut cargo = Command::new("cargo");
        cargo.args(["build", "--benches"]);

        self.add_rust_flags(&mut cargo, measurement_mode);

        if let Some(features) = self.features {
            cargo.arg("--features").arg(features.join(","));
        }

        cargo.args(self.passthrough_flags);

        cargo.arg("--profile").arg(self.profile);

        self.filters.package.add_cargo_args(&mut cargo);

        cargo
    }
}

impl PackageFilters {
    fn add_cargo_args(&self, cargo: &mut Command) {
        if self.workspace {
            cargo.arg("--workspace");
        }

        if !self.package.is_empty() {
            self.package.iter().for_each(|p| {
                cargo.arg("--package").arg(p);
            });
        }

        if !self.exclude.is_empty() {
            self.exclude.iter().for_each(|p| {
                cargo.arg("--exclude").arg(p);
            });
        }
    }
}

pub fn build_benches(
    metadata: &Metadata,
    filters: Filters,
    features: Option<Vec<String>>,
    profile: String,
    quiet: bool,
    measurement_mode: MeasurementMode,
    passthrough_flags: Vec<String>,
) -> Result<()> {
    let built_benches = BuildOptions {
        filters,
        features: &features,
        profile: &profile,
        passthrough_flags: &passthrough_flags,
    }
    .build(metadata, quiet, measurement_mode)?;

    if built_benches.is_empty() {
        bail!(
            "No benchmark target found. \
            Please add a benchmark target to your Cargo.toml"
        );
    }

    let codspeed_target_dir = get_codspeed_target_dir(metadata, measurement_mode);
    let built_bench_count = built_benches.len();

    // Create and clear packages codspeed target directories
    let target_dir_to_clear = built_benches
        .iter()
        .unique_by(|bench| &bench.package)
        .map(|bench| codspeed_target_dir.clone().join(&bench.package));
    for target_dir in target_dir_to_clear {
        std::fs::create_dir_all(&target_dir)?;
        clear_dir(&target_dir)?;
    }

    // Copy built artifacts to codspeed target directory
    for built_bench in built_benches {
        let codspeed_target_package_dir = codspeed_target_dir.clone().join(&built_bench.package);

        std::fs::copy(
            built_bench.executable_path,
            codspeed_target_package_dir.join(built_bench.bench),
        )?;
    }

    eprintln!("Built {built_bench_count} benchmark suite(s)");

    Ok(())
}
