use std::{io, path::PathBuf};

use anyhow::anyhow;
use termcolor::Color;

use crate::{helpers::get_codspeed_target_dir, prelude::*};

struct BenchToRun {
    bench_path: PathBuf,
    bench_name: String,
    working_directory: PathBuf,
    package_name: String,
}

pub fn run_benches(
    ws: &Workspace,
    selected_bench_names: Option<Vec<String>>,
    package: Option<String>,
) -> Result<()> {
    let codspeed_target_dir = get_codspeed_target_dir(ws);

    let packages_to_run = if let Some(package) = package.as_ref() {
        let p = ws
            .members()
            .find(|m| m.manifest().name().to_string().as_str() == package);
        if let Some(p) = p {
            vec![p]
        } else {
            bail!("Package {} not found", package);
        }
    } else {
        ws.default_members().collect::<Vec<_>>()
    };
    let mut benches: Vec<BenchToRun> = vec![];
    for p in packages_to_run {
        let package_name = p.manifest().name().to_string();
        let is_root_package = p.root() == ws.root();
        let package_target_dir = if is_root_package {
            codspeed_target_dir.clone()
        } else {
            codspeed_target_dir.join(&package_name)
        };
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
        if let Some(package) = package.as_ref() {
            bail!(
                "No benchmarks found. Run `cargo codspeed build -p {}` first.",
                package
            );
        } else {
            bail!("No benchmarks found. Run `cargo codspeed build` first.");
        }
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
    ws.config().shell().status_with_color(
        "Collected",
        format!("{} benchmark suite(s) to run", to_run.len()),
        Color::White,
    )?;
    for bench in to_run.iter() {
        let bench_name = &bench.bench_name;
        // workspace_root is needed since file! returns the path relatively to the workspace root
        // while CARGO_MANIFEST_DIR returns the path to the sub package
        let workspace_root = ws.root().to_string_lossy();
        ws.config().shell().status_with_color(
            "Running",
            format!("{} {}", &bench.package_name, bench_name),
            Color::Yellow,
        )?;
        std::process::Command::new(&bench.bench_path)
            .env("CODSPEED_CARGO_WORKSPACE_ROOT", workspace_root.as_ref())
            .current_dir(&bench.working_directory)
            .status()
            .map_err(|_| anyhow!("failed to execute the benchmark process"))
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
        ws.config().shell().status_with_color(
            "Done",
            format!("running {}", bench_name),
            Color::Green,
        )?;
    }
    ws.config().shell().status_with_color(
        "Finished",
        format!("running {} benchmark suite(s)", to_run.len()),
        Color::Green,
    )?;
    Ok(())
}
