use std::path::PathBuf;

use anyhow::Context;

// !!!!!!!!!!!!!!!!!!!!!!!!
// !! DO NOT TOUCH BELOW !!
// !!!!!!!!!!!!!!!!!!!!!!!!
// Has to be in sync with `runner`.
//
const RUNNER_CTL_FIFO_NAME: &str = "runner.ctl.fifo";
const RUNNER_ACK_FIFO_NAME: &str = "runner.ack.fifo";

pub fn runner_fifo_dir() -> anyhow::Result<PathBuf> {
    let raw_path = std::env::var("CODSPEED_FIFO_DIR")
        .context("CODSPEED_FIFO_DIR environment variable not set")?;
    Ok(PathBuf::from(raw_path))
}

pub fn runner_ctl_fifo_path() -> anyhow::Result<PathBuf> {
    runner_fifo_dir().map(|p| p.join(RUNNER_CTL_FIFO_NAME))
}

pub fn runner_ack_fifo_path() -> anyhow::Result<PathBuf> {
    runner_fifo_dir().map(|p| p.join(RUNNER_ACK_FIFO_NAME))
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub enum Command {
    CurrentBenchmark { pid: u32, uri: String },
    StartBenchmark,
    StopBenchmark,
    Ack,
}
//
// !!!!!!!!!!!!!!!!!!!!!!!!
// !! DO NOT TOUCH ABOVE !!
// !!!!!!!!!!!!!!!!!!!!!!!!
