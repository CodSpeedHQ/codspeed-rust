use predicates::str::contains;

mod helpers;
use helpers::*;

const DIR: &str = "tests/package_in_deps.in";

#[test]
fn test_package_in_deps_build() {
    let dir = setup(DIR, Project::PackageInDeps);
    cargo_codspeed(&dir)
        .arg("build")
        .assert()
        .success()
        .stderr(contains("Finished built 1 benchmark suite(s)"));
    teardown(dir);
}
