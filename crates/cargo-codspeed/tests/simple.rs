use predicates::str::contains;

mod helpers;
use helpers::*;

const DIR: &str = "tests/simple.in";

#[test]
fn test_simple_run_without_build() {
    let dir = setup(DIR, Project::Simple);
    cargo_codspeed(&dir)
        .arg("run")
        .assert()
        .failure()
        .stderr(contains("Error No benchmarks found."));
    teardown(dir);
}

#[test]
fn test_simple_build() {
    let dir = setup(DIR, Project::Simple);
    cargo_codspeed(&dir)
        .arg("build")
        .assert()
        .success()
        .stderr(contains("Finished built 2 benchmark suite(s)"));
    teardown(dir);
}

#[test]
fn test_simple_build_and_run() {
    let dir = setup(DIR, Project::Simple);
    cargo_codspeed(&dir).arg("build").assert();
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
        .arg("another_bencher_example")
        .assert()
        .success()
        .stderr(contains("Finished built 1 benchmark suite(s)"))
        .stderr(contains("another_bencher_example"));
    teardown(dir);
}

#[test]
fn test_simple_build_and_run_single() {
    let dir = setup(DIR, Project::Simple);
    cargo_codspeed(&dir)
        .arg("build")
        .arg("another_bencher_example")
        .assert();
    cargo_codspeed(&dir)
        .arg("run")
        .arg("another_bencher_example")
        .assert()
        .success()
        .stderr(contains("Finished running 1 benchmark suite(s)"))
        .stderr(contains("another_bencher_example"));
    teardown(dir);
}
