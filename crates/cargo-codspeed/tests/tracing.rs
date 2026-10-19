use predicates::str::contains;

mod helpers;
use helpers::*;

const DIR: &str = "tests/tracing.in";

fn build_and_run(build_args: &[&str], run_args: &[&str], expected_level: &str) {
    let dir = setup(DIR, Project::Simple);
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
        )));
    teardown(dir);
}

#[test]
fn test_tracing_compiled_out_for_simulation() {
    build_and_run(&[], &[], "off");
}

#[test]
fn test_tracing_compiled_out_for_walltime() {
    build_and_run(&["-m", "walltime"], &["-m", "walltime"], "off");
}

#[test]
fn test_tracing_kept_with_keep_tracing() {
    build_and_run(&["--keep-tracing"], &[], "trace");
}

#[test]
fn test_tracing_kept_for_telemetry() {
    build_and_run(&["-m", "telemetry"], &["-m", "telemetry"], "trace");
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

#[test]
fn test_tracing_kept_for_telemetry_built_with_other_modes() {
    build_and_run(&["-m", "walltime,telemetry"], &["-m", "telemetry"], "trace");
}

#[test]
fn test_tracing_kept_with_keep_tracing_and_telemetry() {
    build_and_run(
        &["-m", "walltime,telemetry", "--keep-tracing"],
        &["-m", "walltime"],
        "trace",
    );
}

#[test]
fn test_telemetry_rejects_keep_tracing() {
    let dir = setup(DIR, Project::Simple);
    cargo_codspeed(&dir)
        .args(["build", "-m", "telemetry", "--keep-tracing"])
        .assert()
        .failure()
        .stderr(contains(
            "Error: `--keep-tracing` has no effect in telemetry mode",
        ));
    teardown(dir);
}
