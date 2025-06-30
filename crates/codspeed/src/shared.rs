//! WARNING: Has to be in sync with `runner`.

#[cfg(unix)]
pub const RUNNER_CTL_FIFO: &str = "/tmp/runner.ctl.fifo";
#[cfg(unix)]
pub const RUNNER_ACK_FIFO: &str = "/tmp/runner.ack.fifo";

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub enum Command {
    CurrentBenchmark { pid: u32, uri: String },
    StartBenchmark,
    StopBenchmark,
    Ack,
    PingPerf,
    SetIntegration { name: String, version: String },
    Err,
}
