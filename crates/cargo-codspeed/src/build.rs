use crate::{
    app::{BenchTargetFilters, PackageFilters},
    helpers::{clear_dir, get_codspeed_target_dir},
    measurement_mode::{BuildMode, MeasurementMode},
    prelude::*,
};
use anyhow::Context;
use cargo_metadata::{camino::Utf8PathBuf, Message, Metadata, TargetKind};
use std::collections::HashMap;
use std::process::{exit, Command, Stdio};

struct BuildOptions<'a> {
    bench_target_filters: BenchTargetFilters,
    package_filters: PackageFilters,
    features: &'a Option<Vec<String>>,
    profile: &'a str,
    passthrough_flags: &'a Vec<String>,
}

struct BuiltBench {
    package: String,
    bench: String,
    executable_path: Utf8PathBuf,
}

pub struct BuildConfig {
    pub package_filters: PackageFilters,
    pub bench_target_filters: BenchTargetFilters,
    pub features: Option<Vec<String>>,
    pub profile: String,
    pub quiet: bool,
    pub measurement_mode: MeasurementMode,
    pub passthrough_flags: Vec<String>,
}

fn get_bench_harness_value(
    manifest_path: &Utf8PathBuf,
    bench_name: &str,
    cache: &mut HashMap<Utf8PathBuf, toml::Table>,
) -> Result<bool> {
    let manifest_table = if let Some(table) = cache.get(manifest_path) {
        table
    } else {
        // Read and parse the Cargo.toml file
        let manifest_content = std::fs::read_to_string(manifest_path)
            .with_context(|| format!("Failed to read manifest at {manifest_path}"))?;
        let table: toml::Table = toml::from_str(&manifest_content)
            .with_context(|| format!("Failed to parse TOML in {manifest_path}"))?;
        cache.insert(manifest_path.clone(), table);
        cache.get(manifest_path).unwrap()
    };

    // Look for [[bench]] sections
    let Some(benches) = manifest_table.get("bench").and_then(|v| v.as_array()) else {
        // If no [[bench]] sections, it's not an error, benches present in <root>/benches/<name>.rs
        // are still collected with harness = true
        return Ok(true);
    };

    // Find the bench entry with matching name
    let matching_bench = benches
        .iter()
        .filter_map(|bench| bench.as_table())
        .find(|bench_table| {
            bench_table
                .get("name")
                .and_then(|v| v.as_str())
                .is_some_and(|name| name == bench_name)
        });

    // Check if harness is enabled (defaults to true)
    let harness_enabled = matching_bench
        .and_then(|bench_table| bench_table.get("harness"))
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    Ok(harness_enabled)
}

impl BuildOptions<'_> {
    /// Builds the benchmarks by invoking cargo
    /// Returns a list of built benchmarks, with path to associated executables
    fn build(
        &self,
        metadata: &Metadata,
        quiet: bool,
        build_mode: BuildMode,
    ) -> Result<Vec<BuiltBench>> {
        let workspace_packages = metadata.workspace_packages();

        let mut cargo = self.build_command(build_mode);
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
        let mut bench_targets_with_default_harness = Vec::new();
        let mut manifest_cache = HashMap::new();

        let package_names = self
            .package_filters
            .packages_from_flags(metadata)
            .map_err(|e| {
                // Avoid leaving an orphan cargo process, even if something went wrong
                cargo.wait().expect("Could not get cargo's exist status");
                e
            })?
            .into_iter()
            .map(|t| t.name.clone())
            .collect::<Vec<_>>();

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

                    let bench_target_name = artifact.target.name;

                    let add_bench_to_codspeed_dir = package_names.iter().contains(&package.name);

                    if add_bench_to_codspeed_dir {
                        if get_bench_harness_value(
                            &package.manifest_path,
                            &bench_target_name,
                            &mut manifest_cache,
                        )? {
                            bench_targets_with_default_harness
                                .push((package.name.to_string(), bench_target_name.clone()));
                        }

                        built_benches.push(BuiltBench {
                            package: package.name.to_string(),
                            bench: bench_target_name,
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

        if !bench_targets_with_default_harness.is_empty() {
            let targets_list = bench_targets_with_default_harness
                .into_iter()
                .map(|(package, bench)| format!("  - `{bench}` in package `{package}`"))
                .join("\n");

            bail!("\
CodSpeed will not work with the following benchmark targets:
{targets_list}

CodSpeed requires benchmark targets to disable the default test harness because benchmark frameworks handle harnessing themselves.

Either disable the default harness by adding `harness = false` to the corresponding \
`[[bench]]` section in the Cargo.toml, or specify which targets to build by using \
`cargo codspeed build -p package_name --bench first_target --bench second_target`.

See `cargo codspeed build --help` for more information.");
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
    fn add_rust_flags(&self, cargo: &mut Command, build_mode: BuildMode) {
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

        // Add the codspeed cfg flag if the benchmark should only run once
        if build_mode == BuildMode::Analysis {
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
    fn build_command(&self, build_mode: BuildMode) -> Command {
        let mut cargo = Command::new("cargo");
        cargo.arg("build");

        if let Some(bench_target_filters) = &self.bench_target_filters.bench {
            for bench_target_filter in bench_target_filters {
                cargo.args(["--bench", bench_target_filter]);
            }
        } else {
            cargo.args(["--benches"]);
        }

        self.add_rust_flags(&mut cargo, build_mode);

        if let Some(features) = self.features {
            cargo.arg("--features").arg(features.join(","));
        }

        cargo.args(self.passthrough_flags);

        cargo.arg("--profile").arg(self.profile);

        self.package_filters.add_cargo_args(&mut cargo);

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

pub fn build_benches(metadata: &Metadata, config: BuildConfig) -> Result<()> {
    let build_mode = config.measurement_mode.into();
    let built_benches = BuildOptions {
        bench_target_filters: config.bench_target_filters,
        package_filters: config.package_filters,
        features: &config.features,
        profile: &config.profile,
        passthrough_flags: &config.passthrough_flags,
    }
    .build(metadata, config.quiet, build_mode)?;

    if built_benches.is_empty() {
        bail!(
            "No benchmark target found. \
            Please add a benchmark target to your Cargo.toml"
        );
    }

    let codspeed_target_dir = get_codspeed_target_dir(metadata, build_mode);
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
