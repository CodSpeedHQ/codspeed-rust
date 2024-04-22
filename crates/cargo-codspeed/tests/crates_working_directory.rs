use predicates::str::contains;
use std::path::PathBuf;

mod helpers;
use helpers::*;

const DIR: &str = "tests/crates_working_directory.in";

#[test]
fn test_crates_working_directory_build_and_run_explicit() {
    let dir = setup(DIR, Project::CratesWorkingDirectory);
    cargo_codspeed(&dir)
        .args(["build", "-p", "the_crate"])
        .assert()
        .success();
    cargo_codspeed(&dir)
        .args(["run", "-p", "the_crate"])
        .assert()
        .success()
        .stderr(contains("Finished running 1 benchmark suite(s)"));
    teardown(dir);
}

#[test]
fn test_crates_working_directory_build_and_run_implicit() {
    let dir = setup(DIR, Project::CratesWorkingDirectory);
    cargo_codspeed(&dir)
        .args(["build", "-p", "the_crate"])
        .assert()
        .success();
    cargo_codspeed(&dir)
        .arg("run")
        .assert()
        .success()
        .stderr(contains("Finished running 1 benchmark suite(s)"));
    teardown(dir);
}

#[test]
fn test_crates_working_directory_build_in_subfolder_and_run() {
    let dir = setup(DIR, Project::CratesWorkingDirectory);
    cargo_codspeed(&dir)
        .current_dir(PathBuf::from(&dir).join("the_crate"))
        .args(["build"])
        .assert()
        .success();
    cargo_codspeed(&dir)
        .arg("run")
        .current_dir(PathBuf::from(&dir).join("the_crate"))
        .assert()
        .success()
        .stderr(contains("Finished running 1 benchmark suite(s)"));
    cargo_codspeed(&dir)
        .arg("run")
        .assert()
        .success()
        .stderr(contains("Finished running 1 benchmark suite(s)"));
    cargo_codspeed(&dir)
        .args(["run", "-p", "the_crate"])
        .assert()
        .success()
        .stderr(contains("Finished running 1 benchmark suite(s)"));
    teardown(dir);
}
