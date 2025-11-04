use predicates::prelude::*;
use predicates::str::contains;

mod helpers;
use helpers::*;

const DIR: &str = "tests/default_harness_error.in";

#[test]
fn test_default_harness_error() {
    let dir = setup(DIR, Project::DefaultHarnessError);
    cargo_codspeed(&dir)
        .arg("build")
        .assert()
        .failure()
        .stderr(contains(
            "Error: CodSpeed will not work with the following benchmark targets:",
        ))
        // Should report bencher_example (explicit bench with missing harness = false)
        .stderr(contains(
            "`bencher_example` in package `default-harness-error-test`",
        ))
        // Should report bencher_no_section (no [[bench]] section means default harness)
        .stderr(contains(
            "`bencher_no_section` in package `default-harness-error-test`",
        ))
        .stderr(contains(
            "CodSpeed requires benchmark targets to disable the default test harness",
        ))
        // Ensure the correct benchmark with harness = false is NOT reported
        .stderr(contains("bencher_correct").not());
    teardown(dir);
}
