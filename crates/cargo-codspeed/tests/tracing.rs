use predicates::str::contains;
use std::io::Write;

mod helpers;
use helpers::*;

const DIR: &str = "tests/tracing.in";

/// Compiles `tracing` and `log` out of the builds that measure, as a project can from its
/// manifest.
const LOGS_OFF_WHEN_MEASURED: &str = r#"
[target.'cfg(all(codspeed_build, not(codspeed_telemetry)))'.dependencies]
tracing = { version = "0.1", features = ["max_level_off"] }
log = { version = "0.4", features = ["max_level_off"] }
"#;

fn build_and_run(
    manifest_addition: Option<&str>,
    build_args: &[&str],
    run_args: &[&str],
    expected_level: &str,
) {
    let dir = setup(DIR, Project::Simple);
    if let Some(manifest_addition) = manifest_addition {
        std::fs::OpenOptions::new()
            .append(true)
            .open(format!("{dir}/Cargo.toml"))
            .unwrap()
            .write_all(manifest_addition.as_bytes())
            .unwrap();
    }
    cargo_codspeed(&dir)
        .arg("build")
        .args(build_args)
        .assert()
        .success();
    cargo_codspeed(&dir)
        .arg("run")
        .args(run_args)
        .assert()
        .success()
        .stderr(contains(format!(
            "tracing static max level: {expected_level}"
        )))
        // `log` prints its levels in uppercase.
        .stderr(contains(format!(
            "log static max level: {}",
            expected_level.to_uppercase()
        )));
    teardown(dir);
}

#[test]
fn test_tracing_kept_for_simulation() {
    build_and_run(None, &[], &[], "trace");
}

#[test]
fn test_tracing_kept_for_walltime() {
    build_and_run(None, &["-m", "walltime"], &["-m", "walltime"], "trace");
}

#[test]
fn test_tracing_kept_for_telemetry() {
    build_and_run(None, &["-m", "telemetry"], &["-m", "telemetry"], "trace");
}

#[test]
fn test_tracing_compiled_out_by_manifest_for_simulation() {
    build_and_run(Some(LOGS_OFF_WHEN_MEASURED), &[], &[], "off");
}

#[test]
fn test_tracing_compiled_out_by_manifest_for_walltime() {
    build_and_run(
        Some(LOGS_OFF_WHEN_MEASURED),
        &["-m", "walltime"],
        &["-m", "walltime"],
        "off",
    );
}

#[test]
fn test_tracing_kept_by_manifest_for_telemetry_built_with_other_modes() {
    build_and_run(
        Some(LOGS_OFF_WHEN_MEASURED),
        &["-m", "walltime,telemetry"],
        &["-m", "telemetry"],
        "trace",
    );
}

#[test]
fn test_telemetry_run_without_build() {
    let dir = setup(DIR, Project::Simple);
    cargo_codspeed(&dir)
        .args(["run", "-m", "telemetry"])
        .assert()
        .failure()
        .stderr(contains(
            "Error: No benchmarks found for the telemetry mode. Run `cargo codspeed build -m telemetry` first.",
        ));
    teardown(dir);
}
