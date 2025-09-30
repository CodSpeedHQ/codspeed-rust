use assert_cmd::assert::OutputAssertExt;
use predicates::prelude::*;
use predicates::str::contains;

mod helpers;
use helpers::*;

const DIR: &str = "tests/simple-bencher.in";

#[test]
fn test_simple_run_without_build() {
    let dir = setup(DIR, Project::Simple);
    cargo_codspeed(&dir)
        .arg("run")
        .assert()
        .failure()
        .stderr(contains("Error: No benchmarks found."));
    teardown(dir);
}

#[test]
fn test_simple_build() {
    let dir = setup(DIR, Project::Simple);
    cargo_codspeed(&dir)
        .arg("build")
        .assert()
        .success()
        .stderr(contains("Built 2 benchmark suite(s)"));
    teardown(dir);
}

#[test]
fn test_simple_build_and_run() {
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
fn test_simple_build_single() {
    let dir = setup(DIR, Project::Simple);
    cargo_codspeed(&dir)
        .arg("build")
        .args(["--bench", "another_bencher_example"])
        .assert()
        .success()
        .stderr(contains("Built 1 benchmark suite(s)"))
        .stderr(contains("another_bencher_example"));
    teardown(dir);
}

#[test]
fn test_simple_build_and_run_single() {
    let dir = setup(DIR, Project::Simple);
    cargo_codspeed(&dir)
        .arg("build")
        .args(["--bench", "another_bencher_example"])
        .assert()
        .success();
    cargo_codspeed(&dir)
        .arg("run")
        .args(["--bench", "another_bencher_example"])
        .assert()
        .success()
        .stderr(contains("Finished running 1 benchmark suite(s)"))
        .stderr(contains("another_bencher_example"));
    teardown(dir);
}

#[test]
fn test_simple_cargo_bench_no_run() {
    let dir = setup(DIR, Project::Simple);
    std::process::Command::new("cargo")
        .arg("bench")
        .arg("--no-run")
        .current_dir(&dir)
        .assert()
        .success();
    teardown(dir);
}

#[test]
fn test_simple_run_without_details() {
    let dir = setup(DIR, Project::Simple);
    cargo_codspeed(&dir).arg("build").assert().success();
    cargo_codspeed(&dir)
        .arg("run")
        .assert()
        .success()
        .stderr(contains("Finished running 2 benchmark suite(s)"))
        .stderr(predicates::str::contains("benchmarks total").not())
        .stdout(
            predicates::str::is_match(r"  Checked: .* \([0-9]+(\.[0-9]+)? (ns|us|ms|s)\)")
                .unwrap()
                .not(),
        );
    teardown(dir);
}

#[test]
fn test_simple_run_with_details() {
    let dir = setup(DIR, Project::Simple);
    cargo_codspeed(&dir).arg("build").assert().success();
    cargo_codspeed(&dir)
        .arg("run")
        .arg("--details")
        .assert()
        .success()
        .stderr(contains("benchmarks total"))
        .stderr(contains("Done running"))
        .stdout(
            predicates::str::is_match(r"  Checked: .* \([0-9]+(\.[0-9]+)? (ns|us|ms|s)\)").unwrap(),
        );
    teardown(dir);
}

#[test]
fn test_benchmark_counting_with_details() {
    let dir = setup(DIR, Project::Simple);
    cargo_codspeed(&dir).arg("build").assert().success();
    cargo_codspeed(&dir)
        .arg("run")
        .arg("--details")
        .assert()
        .success()
        .stderr(contains("Done running bencher_example (2 benchmarks)"))
        .stderr(contains(
            "Done running another_bencher_example (2 benchmarks)",
        ))
        .stderr(contains(
            "Finished running 2 benchmark suite(s) (4 benchmarks total)",
        ));
    teardown(dir);
}

#[test]
fn test_single_benchmark_counting_with_details() {
    let dir = setup(DIR, Project::Simple);
    cargo_codspeed(&dir)
        .arg("build")
        .args(["--bench", "bencher_example"])
        .assert()
        .success();
    cargo_codspeed(&dir)
        .arg("run")
        .arg("--details")
        .args(["--bench", "bencher_example"])
        .assert()
        .success()
        .stderr(contains("Done running bencher_example (2 benchmarks)"))
        .stderr(contains(
            "Finished running 1 benchmark suite(s) (2 benchmarks total)",
        ));
    teardown(dir);
}
