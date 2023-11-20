use anstyle::{AnsiColor, Style};
use anyhow::anyhow;

use crate::{
    helpers::{get_codspeed_dir, read_dir_recursive},
    prelude::*,
};

pub fn run_benches(
    ws: &Workspace,
    benches: Option<Vec<String>>,
    package: Option<String>,
) -> Result<()> {
    let mut codspeed_dir = get_codspeed_dir(ws);

    if let Some(package) = package {
        codspeed_dir.push(package.clone());
        if !codspeed_dir.exists() {
            return Err(anyhow!(
                "No benchmarks found. Run `cargo codspeed build -p {}` first.",
                package
            ));
        }
    }
    if !codspeed_dir.exists() {
        return Err(anyhow!(
            "No benchmarks found. Run `cargo codspeed build` first."
        ));
    }

    let found_benches = read_dir_recursive(codspeed_dir)?;

    if found_benches.is_empty() {
        return Err(anyhow!(
            "No benchmark target found. Run `cargo codspeed build` first."
        ));
    }
    let mut to_run = vec![];
    if let Some(benches) = benches {
        // Make sure all benchmarks are found
        let mut not_found = vec![];
        for bench in benches.iter() {
            let bench_path = found_benches
                .iter()
                .find(|b| b.file_name().unwrap().to_str().unwrap() == bench);

            if let Some(bench_path) = bench_path {
                to_run.push(bench_path.clone());
            } else {
                not_found.push(bench);
            }
        }

        if !not_found.is_empty() {
            return Err(anyhow!(
                "The following benchmarks to run were not found: {}",
                not_found.iter().join(", ")
            ));
        }
    } else {
        to_run = found_benches;
    }
    ws.config().shell().status_with_color(
        "Collected",
        format!("{} benchmark suite(s) to run", to_run.len()),
        &Style::new().fg_color(Some(AnsiColor::White.into())),
    )?;
    for bench in to_run.iter() {
        let bench_name = bench.file_name().unwrap().to_str().unwrap();
        // workspace_root is needed since file! returns the path relatively to the workspace root
        // while CARGO_MANIFEST_DIR returns the path to the sub package
        let workspace_root = ws.root().to_string_lossy();
        ws.config().shell().status_with_color(
            "Running",
            bench_name,
            &Style::new().fg_color(Some(AnsiColor::Yellow.into())),
        )?;
        std::process::Command::new(bench)
            .env("CODSPEED_CARGO_WORKSPACE_ROOT", workspace_root.as_ref())
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
            &Style::new().fg_color(Some(AnsiColor::Green.into())),
        )?;
    }
    ws.config().shell().status_with_color(
        "Finished",
        format!("running {} benchmark suite(s)", to_run.len()),
        &Style::new().fg_color(Some(AnsiColor::Green.into())),
    )?;
    Ok(())
}
