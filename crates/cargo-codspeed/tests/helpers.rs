use std::env;
use std::env::temp_dir;
use std::path::PathBuf;
use std::str::FromStr;

use assert_cmd::Command;
use fs_extra::dir::copy;
use fs_extra::dir::create;
use fs_extra::dir::remove;
use uuid::Uuid;

fn replace_in_file(path: &str, from: &str, to: &str) {
    let mut contents = std::fs::read_to_string(path).unwrap();
    contents = contents.replace(from, to);
    std::fs::write(path, contents).unwrap();
}

#[allow(dead_code)]
pub enum Project {
    Simple,
    Features,
    Workspace,
}

pub fn setup(dir: &str, project: Project) -> String {
    //Create a new unique named temp directory
    let tmp_dir = temp_dir().join(format!("cargo-codspeed-test-{}", Uuid::new_v4()));
    create(&tmp_dir, false).unwrap();
    let mut copy_opts = fs_extra::dir::CopyOptions::new();
    copy_opts.content_only = true;
    copy(dir, &tmp_dir, &copy_opts).unwrap();
    if env::var("DEBUG").is_ok() {
        println!("tmp_dir={:?}", tmp_dir);
    }

    let package_root = PathBuf::from_str(env!("CARGO_MANIFEST_DIR")).unwrap();
    let workspace_root = package_root.parent().unwrap().parent().unwrap();
    match project {
        Project::Simple | Project::Features => {
            replace_in_file(
                tmp_dir.join("Cargo.toml").to_str().unwrap(),
                "../../..",
                workspace_root.join("crates").to_str().unwrap(),
            );
        }
        Project::Workspace => {
            replace_in_file(
                tmp_dir.join("a").join("Cargo.toml").to_str().unwrap(),
                "../../../..",
                workspace_root.join("crates").to_str().unwrap(),
            );
            replace_in_file(
                tmp_dir.join("b").join("Cargo.toml").to_str().unwrap(),
                "../../../..",
                workspace_root.join("crates").to_str().unwrap(),
            );
        }
    }
    tmp_dir.to_str().unwrap().to_string()
}

pub fn teardown(dir: String) {
    if env::var("DEBUG").is_err() {
        remove(dir).unwrap();
    }
}

pub fn cargo_codspeed(dir: &String) -> Command {
    let mut cmd = Command::cargo_bin("cargo-codspeed").unwrap();
    cmd.current_dir(dir);
    cmd
}
