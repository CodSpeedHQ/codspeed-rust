use crate::{
    app::{Filters, PackageFilters},
    helpers::get_codspeed_target_dir,
    measurement_mode::MeasurementMode,
    prelude::*,
    walltime::{parse_sample_json, Results, WalltimeBenchmark},
};
use anyhow::Context;
use cargo_metadata::{Metadata, Package};
use glob::glob;
use std::{
    io::{self, Read},
    path::PathBuf,
    process::Stdio,
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
    let codspeed_target_dir = get_codspeed_target_dir(metadata);
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
    match measurement_mode {
        MeasurementMode::Instrumentation => run_instrumentation_benches(metadata, &to_run)?,
        MeasurementMode::Walltime => run_walltime_benches(metadata, &to_run)?,
    }
    eprintln!("Finished running {} benchmark suite(s)", to_run.len());
    Ok(())
}

fn run_instrumentation_benches(metadata: &Metadata, to_run: &[&BenchToRun]) -> Result<()> {
    for bench in to_run.iter() {
        let bench_name = &bench.bench_name;
        // workspace_root is needed since file! returns the path relatively to the workspace root
        // while CARGO_MANIFEST_DIR returns the path to the sub package
        let workspace_root = metadata.workspace_root.clone();
        eprintln!("Running {} {}", &bench.package_name, bench_name);
        std::process::Command::new(&bench.bench_path)
            .env("CODSPEED_CARGO_WORKSPACE_ROOT", workspace_root)
            .current_dir(&bench.working_directory)
            .status()
            .map_err(|e| anyhow!("failed to execute the benchmark process: {}", e))
            .and_then(|status| {
                if status.success() {
                    Ok(())
                } else {
                    Err(anyhow!(
                        "failed to execute the benchmark process, exit code: {}",
                        status.code().unwrap_or(1)
                    ))
                }
            })?;
        eprintln!("Done running {}", bench_name);
    }
    Ok(())
}

fn run_walltime_benches(metadata: &Metadata, to_run: &[&BenchToRun]) -> Result<()> {
    let workspace_root = metadata.workspace_root.clone();
    let criterion_output_directory_base = workspace_root.join("target/codspeed/criterion");
    // clean the directory before running the benchmarks
    std::fs::remove_dir_all(&criterion_output_directory_base).ok();

    for bench in to_run.iter() {
        let bench_name = &bench.bench_name;
        eprintln!("Running {} {}", &bench.package_name, bench_name);

        let output_directory = criterion_output_directory_base.join(bench_name);

        let mut bench_process = std::process::Command::new(&bench.bench_path)
            .env("CODSPEED_CARGO_WORKSPACE_ROOT", &workspace_root)
            .arg("--bench")
            .current_dir(&bench.working_directory)
            .current_dir(workspace_root.clone())
            // .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .env("CRITERION_HOME", output_directory)
            .spawn()
            .context("Failed to spawn benchmark process")?;

        // let mut stdout = bench_process
        //     .stdout
        //     .take()
        //     .context("Failed to take stdout")?;
        // let mut stdout_buffer = [0; 1024];
        // let mut stdout_string = String::new();
        // loop {
        //     let bytes_read = stdout
        //         .read(&mut stdout_buffer)
        //         .context("Failed to read stdout")?;

        //     let read_string = String::from_utf8_lossy(&stdout_buffer[..bytes_read]);
        //     eprint!("stdout: {}", read_string);
        //     stdout_string.push_str(&read_string);

        //     if let io::Result::Ok(Some(exit_status)) = bench_process.try_wait() {
        //         if exit_status.success() {
        //             break;
        //         } else {
        //             bail!(
        //                 "Failed to execute the benchmark process, exit code: {}",
        //                 exit_status.code().unwrap_or(1)
        //             );
        //         }
        //     }
        // }
        // eprintln!("stdout str: {}\nend of stdout", stdout_string);

        let mut stderr = bench_process
            .stderr
            .take()
            .context("Failed to take stderr")?;
        let mut stderr_buffer = [0; 1024];
        let mut stderr_string = String::new();
        loop {
            let bytes_read = stderr
                .read(&mut stderr_buffer)
                .context("Failed to read stderr")?;

            let read_string = String::from_utf8_lossy(&stderr_buffer[..bytes_read]);
            // eprintln!("stderr: {}. done", read_string);
            dbg!(&read_string);
            stderr_string.push_str(&read_string);

            if let io::Result::Ok(Some(exit_status)) = bench_process.try_wait() {
                if exit_status.success() {
                    break;
                } else {
                    bail!(
                        "Failed to execute the benchmark process, exit code: {}",
                        exit_status.code().unwrap_or(1)
                    );
                }
            }
        }
        dbg!(&stderr_string);
        eprintln!("stderr str: {}\nend of stderr", stderr_string);
    }

    let mut walltime_benchmarks = vec![];

    // retrieve data from `workspace_root/target/criterion/{sub_bench_name}/sample.json
    for sample in glob(
        criterion_output_directory_base
            .join("*/*/new/sample.json")
            .as_ref(),
    )? {
        let sample = sample?;
        let sample_path = sample.display().to_string();
        let benchmark_name = sample
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();
        eprintln!("Found sample: {}, name: {}", sample_path, benchmark_name);

        let walltime_benchmark =
            WalltimeBenchmark::from((benchmark_name.to_string(), parse_sample_json(&sample)?));
        walltime_benchmarks.push(walltime_benchmark);
    }

    // TODO: move this in the app module?
    let profile_folder = std::env::var("CODSPEED_PROFILE_FOLDER")
        .map(PathBuf::from)
        .unwrap_or_else(|_| workspace_root.join("target/codspeed/profiles").into());

    let results = Results::new(walltime_benchmarks);

    let results_folder = profile_folder.join("results");
    std::fs::create_dir_all(&results_folder).context("Failed to create results folder")?;
    let results_path = results_folder.join(format!("{}.json", std::process::id()));
    let results_file =
        std::fs::File::create(&results_path).context("Failed to create results file")?;
    serde_json::to_writer(results_file, &results)?;

    Ok(())
}
