use std::{io, path::PathBuf};

use anyhow::anyhow;
use cargo::ops::Packages;

use crate::{
    helpers::{get_codspeed_target_dir, style},
    prelude::*,
};

struct BenchToRun {
    bench_path: PathBuf,
    bench_name: String,
    working_directory: PathBuf,
    package_name: String,
}

pub fn run_benches(
    ws: &Workspace,
    selected_bench_names: Option<Vec<String>>,
    packages: Packages,
) -> Result<()> {
    let codspeed_target_dir = get_codspeed_target_dir(ws);
    let packages_to_run = packages.get_packages(ws)?;
    let mut benches: Vec<BenchToRun> = vec![];
    for p in packages_to_run {
        let package_name = p.manifest().name().to_string();
        let package_target_dir = codspeed_target_dir.join(&package_name);
        let working_directory = p.root().to_path_buf();
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
                        bench_path,
                        bench_name,
                        working_directory: working_directory.clone(),
                    });
                }
            }
        }
    }

    if benches.is_empty() {
        bail!("No benchmarks found. Run `cargo codspeed build` first.");
    }

    let mut to_run = vec![];
    if let Some(selected_bench_names) = selected_bench_names {
        // Make sure all benchmarks are found
        let mut not_found = vec![];
        for bench_name in selected_bench_names.iter() {
            let bench = benches.iter().find(|b| &b.bench_name == bench_name);

            if let Some(bench) = bench {
                to_run.push(bench);
            } else {
                not_found.push(bench_name);
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
    ws.gctx().shell().status_with_color(
        "Collected",
        format!("{} benchmark suite(s) to run", to_run.len()),
        &style::TITLE,
    )?;
    for bench in to_run.iter() {
        let bench_name = &bench.bench_name;
        // workspace_root is needed since file! returns the path relatively to the workspace root
        // while CARGO_MANIFEST_DIR returns the path to the sub package
        let workspace_root = ws.root().to_string_lossy();
        ws.gctx().shell().status_with_color(
            "Running",
            format!("{} {}", &bench.package_name, bench_name),
            &style::ACTIVE,
        )?;
        std::process::Command::new(&bench.bench_path)
            .env("CODSPEED_CARGO_WORKSPACE_ROOT", workspace_root.as_ref())
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
        ws.gctx().shell().status_with_color(
            "Done",
            format!("running {}", bench_name),
            &style::SUCCESS,
        )?;
    }
    ws.gctx().shell().status_with_color(
        "Finished",
        format!("running {} benchmark suite(s)", to_run.len()),
        &style::SUCCESS,
    )?;
    Ok(())
}
