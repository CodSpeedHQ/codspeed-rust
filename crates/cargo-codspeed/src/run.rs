use crate::{
    app::{BenchTargetFilters, PackageFilters},
    helpers::get_codspeed_target_dir,
    measurement_mode::MeasurementMode,
    prelude::*,
};
use anyhow::Context;
use cargo_metadata::{Metadata, Package};
use codspeed::walltime_results::WalltimeResults;
use std::{
    io::{self, Write},
    path::{Path, PathBuf},
    process::Stdio,
};

#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;

struct BenchToRun {
    bench_path: PathBuf,
    bench_target_name: String,
    working_directory: PathBuf,
    package_name: String,
}

impl PackageFilters {
    /// Logic taken from [cargo::ops::Packages](https://docs.rs/cargo/0.85.0/src/cargo/ops/cargo_compile/packages.rs.html#34-42)
    pub(crate) fn packages_from_flags<'a>(
        &self,
        metadata: &'a Metadata,
    ) -> Result<Vec<&'a Package>> {
        Ok(
            match (self.workspace, self.exclude.len(), self.package.len()) {
                (false, 0, 0) => metadata.workspace_default_packages(),
                (false, 0, _) => metadata
                    .workspace_packages()
                    .into_iter()
                    .filter(|p| self.package.contains(&p.name))
                    .collect(),
                (false, _, _) => bail!("--exclude can only be used with --workspace"),
                (true, 0, _) => metadata.workspace_packages(),
                (true, _, _) => metadata
                    .workspace_packages()
                    .into_iter()
                    .filter(|p| !self.exclude.contains(&p.name))
                    .collect(),
            },
        )
    }

    fn benches_to_run(
        &self,
        metadata: &Metadata,
        bench_target_filters: BenchTargetFilters,
        codspeed_target_dir: PathBuf,
    ) -> Result<Vec<BenchToRun>> {
        let packages = self.packages_from_flags(metadata)?;

        let mut benches = vec![];
        for package in packages {
            let package_name = &package.name;
            let package_target_dir = codspeed_target_dir.join(package_name);
            let working_directory = package.manifest_path.parent().ok_or_else(|| {
                Error::msg(format!("Failed to get root dir for package {package_name}"))
            })?;
            if let io::Result::Ok(read_dir) = std::fs::read_dir(&package_target_dir) {
                for entry in read_dir {
                    let entry = entry?;
                    let bench_path = entry.path();
                    let bench_target_name = bench_path
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string();
                    if let Some(bench_target_filter) = &bench_target_filters.bench {
                        if !bench_target_filter.contains(&bench_target_name) {
                            continue;
                        }
                    }
                    if !bench_path.is_dir() {
                        benches.push(BenchToRun {
                            package_name: package_name.clone(),
                            working_directory: working_directory.into(),
                            bench_path,
                            bench_target_name,
                        });
                    }
                }
            }
        }

        Ok(benches)
    }
}

pub fn run_benches(
    metadata: &Metadata,
    bench_name_filter: Option<String>,
    package_filters: PackageFilters,
    bench_target_filters: BenchTargetFilters,
    measurement_mode: MeasurementMode,
    show_details: bool,
) -> Result<()> {
    let codspeed_target_dir = get_codspeed_target_dir(metadata, measurement_mode);
    let workspace_root = metadata.workspace_root.as_std_path();
    if measurement_mode == MeasurementMode::Walltime {
        WalltimeResults::clear(workspace_root)?;
    }
    let benches =
        package_filters.benches_to_run(metadata, bench_target_filters, codspeed_target_dir)?;
    if benches.is_empty() {
        bail!("No benchmarks found. Run `cargo codspeed build` first.");
    }

    eprintln!("Collected {} benchmark suite(s) to run", benches.len());

    let mut total_benchmark_count = 0;

    for bench in benches.iter() {
        let bench_target_name = &bench.bench_target_name;
        // workspace_root is needed since file! returns the path relatively to the workspace root
        // while CARGO_MANIFEST_DIR returns the path to the sub package
        let workspace_root = metadata.workspace_root.clone();
        eprintln!("Running {} {}", &bench.package_name, bench_target_name);
        let mut command = std::process::Command::new(&bench.bench_path);
        command
            .env("CODSPEED_CARGO_WORKSPACE_ROOT", workspace_root)
            .current_dir(&bench.working_directory);

        if show_details {
            command.env("CODSPEED_SHOW_DETAILS", "1");
            command.stdout(Stdio::piped()).stderr(Stdio::inherit());
        }

        if measurement_mode == MeasurementMode::Walltime {
            command.arg("--bench"); // Walltime targets need this additional argument (inherited from running them with `cargo bench`)
        }

        if let Some(bench_name_filter) = bench_name_filter.as_ref() {
            command.arg(bench_name_filter);
        }

        if show_details {
            // Only capture and process output when details are requested
            let output = command
                .output()
                .map_err(|e| anyhow!("failed to execute the benchmark process: {}", e))?;

            // Count benchmarks by looking for "Measured:" or "Checked:" lines
            let stdout = String::from_utf8_lossy(&output.stdout);
            let benchmark_count = stdout
                .lines()
                .filter(|line| {
                    line.trim_start().starts_with("Measured:")
                        || line.trim_start().starts_with("Checked:")
                        || line.trim_start().starts_with("  Checked:")
                        || line.trim_start().starts_with("  Measured:")
                })
                .count();
            total_benchmark_count += benchmark_count;

            // Print captured output
            print!("{stdout}");
            io::stdout().flush().unwrap();

            if !output.status.success() {
                #[cfg(unix)]
                {
                    let code = output
                        .status
                        .code()
                        .or(output.status.signal().map(|s| 128 + s)) // 128+N indicates that a command was interrupted by signal N (see: https://tldp.org/LDP/abs/html/exitcodes.html)
                        .unwrap_or(1);

                    eprintln!("failed to execute the benchmark process, exit code: {code}");

                    std::process::exit(code);
                }
                #[cfg(not(unix))]
                {
                    bail!("failed to execute the benchmark process: {}", output.status)
                }
            }

            if benchmark_count == 0 && !stdout.is_empty() {
                eprintln!("Warning: No benchmarks detected in output for {bench_target_name}");
            }
            eprintln!("Done running {bench_target_name} ({benchmark_count} benchmarks)");
        } else {
            // Fast path: don't capture output when details aren't needed
            command
                .status()
                .map_err(|e| anyhow!("failed to execute the benchmark process: {}", e))
                .and_then(|status| {
                    if status.success() {
                        Ok(())
                    } else {
                        #[cfg(unix)]
                        {
                            let code = status
                                .code()
                                .or(status.signal().map(|s| 128 + s)) // 128+N indicates that a command was interrupted by signal N (see: https://tldp.org/LDP/abs/html/exitcodes.html)
                                .unwrap_or(1);

                            eprintln!("failed to execute the benchmark process, exit code: {code}");

                            std::process::exit(code);
                        }
                        #[cfg(not(unix))]
                        {
                            bail!("failed to execute the benchmark process: {}", status)
                        }
                    }
                })?;
            eprintln!("Done running {bench_target_name}");
        }
    }
    if show_details {
        eprintln!(
            "Finished running {} benchmark suite(s) ({total_benchmark_count} benchmarks total)",
            benches.len()
        );
    } else {
        eprintln!("Finished running {} benchmark suite(s)", benches.len());
    }

    if measurement_mode == MeasurementMode::Walltime {
        aggregate_raw_walltime_data(workspace_root)?;
    }

    Ok(())
}

fn aggregate_raw_walltime_data(workspace_root: &Path) -> Result<()> {
    let results = WalltimeResults::collect_walltime_results(workspace_root)
        .with_context(|| {
            format!(
                "Failed to collect walltime results. This may be due to version incompatibility. \
                Ensure that your compat layer (codspeed-criterion-compat, codspeed-bencher-compat, or codspeed-divan-compat) \
                has the same major version as cargo-codspeed (currently v{}).",
                env!("CARGO_PKG_VERSION")
            )
        })?;

    if results.benchmarks().is_empty() {
        eprintln!("No walltime benchmarks found");
        return Ok(());
    }

    for bench in results.benchmarks() {
        if bench.is_invalid() {
            eprintln!(
                "Warning: Benchmark {} was possibly optimized away",
                bench.name()
            );
        }
    }

    let results_folder = std::env::var("CODSPEED_PROFILE_FOLDER")
        .map(PathBuf::from)
        .unwrap_or_else(|_| workspace_root.join("target/codspeed/profiles"))
        .join("results");
    std::fs::create_dir_all(&results_folder).context("Failed to create results folder")?;

    let results_path = results_folder.join(format!("{}.json", std::process::id()));
    let mut results_file =
        std::fs::File::create(&results_path).context("Failed to create results file")?;
    serde_json::to_writer_pretty(&results_file, &results)?;
    results_file
        .flush()
        .context("Failed to flush results file")?;
    Ok(())
}
