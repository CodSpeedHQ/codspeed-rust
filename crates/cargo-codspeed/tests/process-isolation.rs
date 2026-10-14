use predicates::str::contains;
use rstest::rstest;
use std::time::Duration;

mod helpers;
use helpers::*;

const DIR: &str = "tests/process-isolation.in";
/// Process isolation only applies to analysis builds.
const MODE: [&str; 2] = ["-m", "simulation"];
const PROCESS_ISOLATION_ENV: &str = "CODSPEED_PROCESS_ISOLATION";
/// Exit code of a Rust process whose main thread panicked.
const PANIC_EXIT_CODE: i32 = 101;
/// Fails a run whose benchmark process got stuck instead of blocking the suite.
const RUN_TIMEOUT: Duration = Duration::from_secs(120);

fn build_and_run(dir: &String, bench: &str, isolation: &str) -> assert_cmd::assert::Assert {
    cargo_codspeed(dir)
        .args(["build", "--bench", bench])
        .args(MODE)
        .assert()
        .success();
    cargo_codspeed(dir)
        .args(["run", "--bench", bench])
        .args(MODE)
        .env(PROCESS_ISOLATION_ENV, isolation)
        .timeout(RUN_TIMEOUT)
        .assert()
}

#[rstest]
fn test_isolation_runs_all_benchmarks(#[values("true", "false")] isolation: &str) {
    let dir = setup(DIR, Project::Simple);
    build_and_run(&dir, "isolated", isolation)
        .success()
        .stdout(contains("isolated.rs::first"))
        .stdout(contains("isolated.rs::second"))
        .stdout(contains("isolated.rs::spawn_on_main_runtime"));
    teardown(dir);
}

#[rstest]
fn test_isolation_fails_the_run_when_a_benchmark_panics(
    #[values("true", "false")] isolation: &str,
) {
    let dir = setup(DIR, Project::Simple);
    build_and_run(&dir, "panicking", isolation)
        .code(PANIC_EXIT_CODE)
        .stderr(contains("benchmark failed"));
    teardown(dir);
}
