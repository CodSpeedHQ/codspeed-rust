use predicates::{prelude::PredicateBooleanExt, str::contains};

mod helpers;
use helpers::*;

const DIR: &str = "tests/features.in";

#[test]
fn test_without_feature() {
    let dir = setup(DIR, Project::Features);
    cargo_codspeed(&dir).arg("build").assert().success();
    cargo_codspeed(&dir)
        .arg("run")
        .assert()
        .success()
        .stderr(contains("Finished running 1 benchmark suite(s)"))
        .stdout(contains("without_feature"))
        .stdout(contains("with_feature").not());
    teardown(dir);
}

#[test]
fn test_with_feature() {
    let dir = setup(DIR, Project::Features);
    cargo_codspeed(&dir)
        .arg("build")
        .arg("-F")
        .arg("sample_feature")
        .assert()
        .success();
    cargo_codspeed(&dir)
        .arg("run")
        .assert()
        .success()
        .stderr(contains("Finished running 1 benchmark suite(s)"))
        .stdout(contains("with_feature"))
        .stdout(contains("without_feature").not());
    teardown(dir);
}

#[test]
fn test_no_default_features() {
    let dir = setup(DIR, Project::Features);
    cargo_codspeed(&dir)
        .arg("build")
        .arg("--no-default-features")
        .assert()
        .success();
    cargo_codspeed(&dir)
        .arg("run")
        .assert()
        .success()
        .stderr(contains("Finished running 1 benchmark suite(s)"))
        .stdout(contains("with_default_feature").not())
        .stdout(contains("without_default_feature"));

    teardown(dir);
}

#[test]
fn test_all_features() {
    let dir = setup(DIR, Project::Features);
    cargo_codspeed(&dir)
        .arg("build")
        .arg("--all-features")
        .assert()
        .success();
    cargo_codspeed(&dir)
        .arg("run")
        .assert()
        .success()
        .stderr(contains("Finished running 1 benchmark suite(s)"))
        .stdout(contains("with_feature"))
        .stdout(contains("with_default_feature"));
    teardown(dir);
}
