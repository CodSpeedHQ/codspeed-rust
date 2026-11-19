use assert_cmd::Command;
use predicates::str::contains;
use rstest::rstest;

#[rstest]
#[case::help("--help", "Usage:")]
#[case::version("--version", env!("CARGO_PKG_VERSION"))]
fn display_options_succeed(#[case] flag: &str, #[case] expected: &str) {
    Command::new(env!("CARGO_BIN_EXE_cargo-codspeed"))
        .arg(flag)
        .assert()
        .success()
        .stdout(contains(expected))
        .stderr("");
}
