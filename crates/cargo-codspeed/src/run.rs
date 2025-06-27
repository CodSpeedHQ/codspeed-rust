use crate::{
    app::{Filters, PackageFilters},
    helpers::get_codspeed_target_dir,
    measurement_mode::MeasurementMode,
    prelude::*,
};
use anyhow::Context;
use cargo_metadata::{Metadata, Package};
use codspeed::{
    walltime::get_raw_result_dir_from_workspace_root,
    walltime_results::{WalltimeBenchmark, WalltimeResults},
};
use glob::glob;
use std::{
    io::{self, Write},
    os::unix::process::ExitStatusExt,
    path::{Path, PathBuf},
};

struct BenchToRun {
    bench_path: PathBuf,
    bench_name: String,
    working_directory: PathBuf,
    package_name: String,
}

impl Filters {
    fn benches_to_run(
        &self,
        codspeed_target_dir: PathBuf,
        metadata: &Metadata,
    ) -> Result<Vec<BenchToRun>> {
        let packages = self.package.packages(metadata)?;

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
                    let bench_name = bench_path
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string();
                    if !bench_path.is_dir() {
                        benches.push(BenchToRun {
                            package_name: package_name.clone(),
                            working_directory: working_directory.into(),
                            bench_path,
                            bench_name,
                        });
                    }
                }
            }
        }

        Ok(benches)
    }
}

impl PackageFilters {
    /// Logic taken from [cargo::ops::Packages](https://docs.rs/cargo/0.85.0/src/cargo/ops/cargo_compile/packages.rs.html#34-42)
    fn packages<'a>(&self, metadata: &'a Metadata) -> Result<Vec<&'a Package>> {
        Ok(
            match (self.workspace, self.exclude.len(), self.package.len()) {
                (false, 0, 0) => metadata.workspace_default_packages(),
                (false, 0, _) => metadata
                    .workspace_packages()
                    .into_iter()
                    .filter(|p| {
                        self.package
                            .iter()
                            .any(|allowed_package| p.name.contains(allowed_package))
                    })
                    .collect(),
                (false, _, _) => bail!("--exclude can only be used with --workspace"),
                (true, 0, _) => metadata.workspace_packages(),
                (true, _, _) => metadata
                    .workspace_packages()
                    .into_iter()
                    .filter(|p| {
                        !self
                            .exclude
                            .iter()
                            .any(|denied_package| p.name.contains(denied_package))
                    })
                    .collect(),
            },
        )
    }
}

pub fn run_benches(
    metadata: &Metadata,
    filters: Filters,
    measurement_mode: MeasurementMode,
) -> Result<()> {
    let codspeed_target_dir = get_codspeed_target_dir(metadata, measurement_mode);
    let workspace_root = metadata.workspace_root.as_std_path();
    if measurement_mode == MeasurementMode::Walltime {
        clear_raw_walltime_data(workspace_root)?;
    }
    let benches = filters.benches_to_run(codspeed_target_dir, metadata)?;
    if benches.is_empty() {
        bail!("No benchmarks found. Run `cargo codspeed build` first.");
    }

    let mut to_run = vec![];
    if let Some(allowed_bench_names) = filters.bench {
        // Make sure all benchmarks are found
        let mut not_found = vec![];
        for allowed_bench_name in allowed_bench_names.iter() {
            let bench = benches
                .iter()
                .find(|b| b.bench_name.contains(allowed_bench_name));

            if let Some(bench) = bench {
                to_run.push(bench);
            } else {
                not_found.push(allowed_bench_name);
            }
        }

        if !not_found.is_empty() {
            bail!(
                "The following benchmarks to run were not found: {}",
                not_found.iter().join(", ")
            );
        }
    } else {
        to_run = benches.iter().collect();
    }
    eprintln!("Collected {} benchmark suite(s) to run", to_run.len());
    for bench in to_run.iter() {
        let bench_name = &bench.bench_name;
        // workspace_root is needed since file! returns the path relatively to the workspace root
        // while CARGO_MANIFEST_DIR returns the path to the sub package
        let workspace_root = metadata.workspace_root.clone();
        eprintln!("Running {} {}", &bench.package_name, bench_name);
        let mut command = std::process::Command::new(&bench.bench_path);
        command
            .env("CODSPEED_CARGO_WORKSPACE_ROOT", workspace_root)
            .current_dir(&bench.working_directory);

        if measurement_mode == MeasurementMode::Walltime {
            command.arg("--bench"); // Walltime targets need this additional argument (inherited from running them with `cargo bench`)
        }

        command
            .status()
            .map_err(|e| anyhow!("failed to execute the benchmark process: {}", e))
            .and_then(|status| {
                if status.success() {
                    Ok(())
                } else {
                    let code = status
                        .code()
                        .or(status.signal().map(|s| 128 + s)) // 128+N indicates that a command was interrupted by signal N (see: https://tldp.org/LDP/abs/html/exitcodes.html)
                        .unwrap_or(1);

                    eprintln!(
                        "failed to execute the benchmark process, exit code: {}",
                        code
                    );
                    std::process::exit(code);
                }
            })?;
        eprintln!("Done running {}", bench_name);
    }
    eprintln!("Finished running {} benchmark suite(s)", to_run.len());

    if measurement_mode == MeasurementMode::Walltime {
        aggregate_raw_walltime_data(workspace_root)?;
    }

    Ok(())
}

fn clear_raw_walltime_data(workspace_root: &Path) -> Result<()> {
    let raw_results_dir = get_raw_result_dir_from_workspace_root(workspace_root);
    std::fs::remove_dir_all(&raw_results_dir).ok(); // ignore errors when the directory does not exist
    std::fs::create_dir_all(&raw_results_dir).context("Failed to create raw_results directory")?;
    Ok(())
}

fn aggregate_raw_walltime_data(workspace_root: &Path) -> Result<()> {
    // retrieve data from `{workspace_root}/target/codspeed/raw_results/{scope}/*.json
    let walltime_benchmarks = glob(&format!(
        "{}/**/*.json",
        get_raw_result_dir_from_workspace_root(workspace_root)
            .to_str()
            .unwrap(),
    ))?
    .map(|sample| {
        let sample = sample?;
        let raw_walltime_data: codspeed::walltime::RawWallTimeData =
            serde_json::from_reader(std::fs::File::open(&sample)?)?;
        Ok(WalltimeBenchmark::from(raw_walltime_data))
    })
    .collect::<Result<Vec<_>>>()?;

    if walltime_benchmarks.is_empty() {
        eprintln!("No walltime benchmarks found");
        return Ok(());
    }

    for bench in &walltime_benchmarks {
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

    let results = WalltimeResults::from_benchmarks(walltime_benchmarks);
    let results_path = results_folder.join(format!("{}.json", std::process::id()));
    let mut results_file =
        std::fs::File::create(&results_path).context("Failed to create results file")?;
    serde_json::to_writer_pretty(&results_file, &results)?;
    results_file
        .flush()
        .context("Failed to flush results file")?;
    Ok(())
}
