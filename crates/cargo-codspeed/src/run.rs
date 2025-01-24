use crate::{
    app::{Filters, PackageFilters},
    helpers::get_codspeed_target_dir,
    prelude::*,
};
use cargo_metadata::{Metadata, Package};
use std::{io, path::PathBuf};

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

pub fn run_benches(metadata: &Metadata, filters: Filters) -> Result<()> {
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
    eprintln!("Finished running {} benchmark suite(s)", to_run.len());
    Ok(())
}
