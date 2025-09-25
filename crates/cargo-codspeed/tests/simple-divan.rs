use assert_cmd::assert::OutputAssertExt;
use predicates::str::contains;

mod helpers;
use helpers::*;

const DIR: &str = "tests/simple-divan.in";

#[test]
fn test_divan_run_without_build() {
    let dir = setup(DIR, Project::Simple);
    cargo_codspeed(&dir)
        .arg("run")
        .assert()
        .failure()
        .stderr(contains("Error: No benchmarks found."));
    teardown(dir);
}

#[test]
fn test_divan_build() {
    let dir = setup(DIR, Project::Simple);
    cargo_codspeed(&dir)
        .arg("build")
        .assert()
        .success()
        .stderr(contains("Built 2 benchmark suite(s)"));
    teardown(dir);
}

#[test]
fn test_divan_build_and_run() {
    let dir = setup(DIR, Project::Simple);
    cargo_codspeed(&dir).arg("build").assert().success();
    cargo_codspeed(&dir)
        .arg("run")
        .assert()
        .success()
        .stderr(contains("Finished running 2 benchmark suite(s)"));
    teardown(dir);
}

#[test]
fn test_divan_build_single() {
    let dir = setup(DIR, Project::Simple);
    cargo_codspeed(&dir)
        .arg("build")
        .args(["--bench", "another_divan_example"])
        .assert()
        .success()
        .stderr(contains("Built 1 benchmark suite(s)"))
        .stderr(contains("another_divan_example"));
    teardown(dir);
}

#[test]
fn test_divan_build_and_run_single() {
    let dir = setup(DIR, Project::Simple);
    cargo_codspeed(&dir)
        .arg("build")
        .args(["--bench", "another_divan_example"])
        .assert()
        .success();
    cargo_codspeed(&dir)
        .arg("run")
        .args(["--bench", "another_divan_example"])
        .assert()
        .success()
        .stderr(contains("Finished running 1 benchmark suite(s)"))
        .stderr(contains("another_divan_example"));
    teardown(dir);
}

#[test]
fn test_divan_build_and_run_filtered_by_name() {
    let dir = setup(DIR, Project::Simple);
    cargo_codspeed(&dir).arg("build").assert().success();
    cargo_codspeed(&dir)
        .arg("run")
        .arg("fib_20")
        .assert()
        .success()
        .stderr(contains("Finished running 2 benchmark suite(s)"));
    teardown(dir);
}

#[test]
fn test_divan_build_and_run_filtered_by_partial_name() {
    let dir = setup(DIR, Project::Simple);
    cargo_codspeed(&dir).arg("build").assert().success();
    cargo_codspeed(&dir)
        .arg("run")
        .arg("bubble_sort")
        .assert()
        .success()
        .stderr(contains("Finished running 2 benchmark suite(s)"));
    teardown(dir);
}

#[test]
fn test_divan_cargo_bench_no_run() {
    let dir = setup(DIR, Project::Simple);
    std::process::Command::new("cargo")
        .arg("bench")
        .arg("--no-run")
        .current_dir(&dir)
        .assert()
        .success();
    teardown(dir);
}
