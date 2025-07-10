use assert_cmd::assert::OutputAssertExt;
use predicates::str::contains;

mod helpers;
use helpers::*;

const DIR: &str = "tests/simple-criterion.in";

#[test]
fn test_criterion_run_without_build() {
    let dir = setup(DIR, Project::Simple);
    cargo_codspeed(&dir)
        .arg("run")
        .assert()
        .failure()
        .stderr(contains("Error: No benchmarks found."));
    teardown(dir);
}

#[test]
fn test_criterion_build() {
    let dir = setup(DIR, Project::Simple);
    cargo_codspeed(&dir)
        .arg("build")
        .assert()
        .success()
        .stderr(contains("Built 2 benchmark suite(s)"));
    teardown(dir);
}

#[test]
fn test_criterion_build_and_run() {
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
fn test_criterion_build_single() {
    let dir = setup(DIR, Project::Simple);
    cargo_codspeed(&dir)
        .arg("build")
        .arg("another_criterion_example")
        .assert()
        .success()
        .stderr(contains("Built 1 benchmark suite(s)"))
        .stderr(contains("another_criterion_example"));
    teardown(dir);
}

#[test]
fn test_criterion_build_and_run_single() {
    let dir = setup(DIR, Project::Simple);
    cargo_codspeed(&dir)
        .arg("build")
        .arg("another_criterion_example")
        .assert()
        .success();
    cargo_codspeed(&dir)
        .arg("run")
        .arg("another_criterion_example")
        .assert()
        .success()
        .stderr(contains("Finished running 1 benchmark suite(s)"))
        .stderr(contains("another_criterion_example"));
    teardown(dir);
}

#[test]
fn test_criterion_cargo_bench_no_run() {
    let dir = setup(DIR, Project::Simple);
    std::process::Command::new("cargo")
        .arg("bench")
        .arg("--no-run")
        .current_dir(&dir)
        .assert()
        .success();
    teardown(dir);
}
