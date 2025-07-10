use std::process::Command;

mod helpers;
use helpers::*;

#[test]
fn test_cargo_config_rustflags() {
    let tmp_dir = setup("tests/cargo_config.in", Project::Simple);

    // Test that cargo bench works with the custom flag
    let output = Command::new("cargo")
        .arg("bench")
        .arg("--no-run")
        .current_dir(&tmp_dir)
        .output()
        .expect("Failed to execute cargo bench");

    assert!(
        output.status.success(),
        "cargo codspeed bench should succeed with .cargo/config.toml rustflags",
    );

    // Test that cargo codspeed build also works with the custom flag
    let output = cargo_codspeed(&tmp_dir)
        .arg("build")
        .output()
        .expect("Failed to execute cargo codspeed build");

    assert!(
        output.status.success(),
        "cargo codspeed build should succeed with .cargo/config.toml rustflags",
    );

    teardown(tmp_dir);
}
