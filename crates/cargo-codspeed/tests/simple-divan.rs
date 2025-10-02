use assert_cmd::assert::OutputAssertExt;
use predicates::{prelude::PredicateBooleanExt, str::contains};

mod helpers;
use helpers::*;

const DIR: &str = "tests/simple-divan.in";
const FIB_BENCH_NAME: &str = "fib_20";
const BUBBLE_SORT_BENCH_NAME: &str = "bubble_sort_bench";

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
        .stdout(contains(FIB_BENCH_NAME))
        .stdout(contains(BUBBLE_SORT_BENCH_NAME).not())
        .stderr(contains("Finished running 2 benchmark suite(s)"));
    cargo_codspeed(&dir)
        .arg("run")
        .arg("bu.*le_sort")
        .assert()
        .success()
        .stdout(contains(FIB_BENCH_NAME).not())
        .stdout(contains(BUBBLE_SORT_BENCH_NAME))
        .stderr(contains("Finished running 2 benchmark suite(s)"));
    teardown(dir);
}

#[test]
fn test_divan_build_and_run_filtered_by_name_single() {
    let dir = setup(DIR, Project::Simple);
    cargo_codspeed(&dir).arg("build").assert().success();
    cargo_codspeed(&dir)
        .arg("run")
        .arg("bu.*le_sort")
        .args(["--bench", "divan_example"])
        .assert()
        .success()
        .stdout(contains(FIB_BENCH_NAME).not())
        .stdout(contains(BUBBLE_SORT_BENCH_NAME).not()) // We are filtering with a name that is not in the selected benchmark
        .stderr(contains("Finished running 1 benchmark suite(s)"));
    cargo_codspeed(&dir)
        .arg("run")
        .arg("fib")
        .args(["--bench", "divan_example"])
        .assert()
        .success()
        .stdout(contains(FIB_BENCH_NAME))
        .stdout(contains(BUBBLE_SORT_BENCH_NAME).not())
        .stderr(contains("Finished running 1 benchmark suite(s)"));
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
