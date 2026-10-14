//! Process isolation: every benchmark runs in a fresh copy of the benchmark
//! executable, so the allocator state and threads that other benchmarks or
//! setup code leave behind can't change its measurement.

use std::{
    io::Write,
    process::{Command, ExitStatus},
    sync::LazyLock,
};

static CURRENT: LazyLock<Isolation> = LazyLock::new(Isolation::from_env);

/// Isolation is on unless this is `0` or `false`.
pub const PROCESS_ISOLATION_ENV: &str = "CODSPEED_PROCESS_ISOLATION";
/// Set on each child process: the URI of the only benchmark it runs.
const ISOLATED_BENCHMARK_ENV: &str = "CODSPEED_ISOLATED_BENCHMARK";

/// Exit code for a child that ended with neither an exit code nor a signal.
const GENERIC_FAILURE_EXIT_CODE: i32 = 1;
/// Shells report a child killed by signal N as exit code 128 + N.
#[cfg(unix)]
const SIGNAL_EXIT_CODE_OFFSET: i32 = 128;

/// How a harness runs each benchmark.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Isolation {
    /// All benchmarks share this process and its allocator state.
    SharedProcess,
    /// Every benchmark runs in its own child process.
    ProcessPerBenchmark,
    /// This process is such a child and only runs the benchmark with this URI.
    IsolatedBenchmark(String),
}

impl Isolation {
    /// The isolation of this process, read once from the environment.
    pub fn current() -> &'static Self {
        &CURRENT
    }

    fn from_env() -> Self {
        if let Ok(uri) = std::env::var(ISOLATED_BENCHMARK_ENV) {
            return Self::IsolatedBenchmark(uri);
        }

        match std::env::var(PROCESS_ISOLATION_ENV).as_deref() {
            Err(_) | Ok("1" | "true") => Self::ProcessPerBenchmark,
            Ok("0" | "false") => Self::SharedProcess,
            Ok(other) => {
                panic!("{PROCESS_ISOLATION_ENV} must be `true` or `false`, got `{other}`")
            }
        }
    }

    /// The only benchmark this process may run. It overrides CLI filters.
    pub fn only_benchmark(&self) -> Option<&str> {
        match self {
            Self::IsolatedBenchmark(uri) => Some(uri),
            Self::SharedProcess | Self::ProcessPerBenchmark => None,
        }
    }

    /// Runs the benchmark `uri`. `bench` must contain the whole measurement,
    /// from `start_benchmark` to `end_benchmark`.
    pub fn run(&self, uri: String, bench: impl FnOnce(String)) {
        match self {
            Self::SharedProcess | Self::IsolatedBenchmark(_) => bench(uri),
            Self::ProcessPerBenchmark => run_in_child_process(&uri),
        }
    }
}

/// Runs this executable again with the same arguments, limited to `uri`, and
/// exits with the child's code if it fails.
fn run_in_child_process(uri: &str) {
    // Keep the parent's output before the child's.
    let _ = std::io::stdout().flush();

    let exe = std::env::current_exe().expect("failed to locate the benchmark executable");
    let status = Command::new(exe)
        .args(std::env::args_os().skip(1))
        .env(ISOLATED_BENCHMARK_ENV, uri)
        .status()
        .unwrap_or_else(|error| panic!("failed to start the process for {uri}: {error}"));

    if !status.success() {
        std::process::exit(exit_code(status));
    }
}

/// Exit code of a failed child, in the shell convention.
fn exit_code(status: ExitStatus) -> i32 {
    #[cfg(unix)]
    if let Some(signal) = std::os::unix::process::ExitStatusExt::signal(&status) {
        return SIGNAL_EXIT_CODE_OFFSET + signal;
    }

    status.code().unwrap_or(GENERIC_FAILURE_EXIT_CODE)
}
