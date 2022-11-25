use anyhow::anyhow;
use termcolor::Color;

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
        Color::White,
    )?;
    for bench in to_run.iter() {
        let bench_name = bench.file_name().unwrap().to_str().unwrap();
        ws.config()
            .shell()
            .status_with_color("Running", bench_name, Color::Yellow)?;
        std::process::Command::new(bench)
            .status()
            .map_err(|_| anyhow!("failed to execute the benchmark process"))?;
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
