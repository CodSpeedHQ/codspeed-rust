use predicates::str::contains;

mod helpers;
use helpers::*;

const DIR: &str = "tests/simple-criterion.in";

#[test]
fn test_build_multiple_measurement_modes() {
    let dir = setup(DIR, Project::Simple);
    cargo_codspeed(&dir)
        .args(["build", "-m", "simulation", "-m", "walltime"])
        .assert()
        .success()
        .stderr(contains(
            "[cargo-codspeed] Measurement modes: simulation, walltime",
        ));
    teardown(dir);
}

#[test]
fn test_build_multiple_measurement_modes_comma_separated() {
    let dir = setup(DIR, Project::Simple);
    cargo_codspeed(&dir)
        .args(["build", "-m", "simulation,walltime"])
        .assert()
        .success()
        .stderr(contains(
            "[cargo-codspeed] Measurement modes: simulation, walltime",
        ));
    teardown(dir);
}
