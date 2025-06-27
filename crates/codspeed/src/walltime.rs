use serde::{Deserialize, Serialize};

use std::{
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct BenchmarkMetadata {
    pub name: String,
    pub uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RawWallTimeData {
    #[serde(flatten)]
    pub metadata: BenchmarkMetadata,
    pub iters_per_round: Vec<u128>,
    pub times_per_round_ns: Vec<u128>,
    pub max_time_ns: Option<u128>,
}

impl RawWallTimeData {
    fn from_runtime_data(
        name: String,
        uri: String,
        iters_per_round: Vec<u128>,
        times_per_round_ns: Vec<u128>,
        max_time_ns: Option<u128>,
    ) -> Self {
        RawWallTimeData {
            metadata: BenchmarkMetadata { name, uri },
            iters_per_round,
            max_time_ns,
            times_per_round_ns,
        }
    }

    fn dump_to_results(&self, workspace_root: &Path, scope: &str) {
        let output_dir = get_raw_result_dir_from_workspace_root(workspace_root).join(scope);
        std::fs::create_dir_all(&output_dir).unwrap();
        let bench_id = uuid::Uuid::new_v4().to_string();
        let output_path = output_dir.join(format!("{}.json", bench_id));
        let mut writer = std::fs::File::create(&output_path).expect("Failed to create the file");
        serde_json::to_writer_pretty(&mut writer, self).expect("Failed to write the data");
        writer.flush().expect("Failed to flush the writer");
    }
}

/// Entry point called in patched integration to harvest raw walltime data
///
/// `CODSPEED_CARGO_WORKSPACE_ROOT` is expected to be set for this to work
///
/// # Arguments
///
/// - `scope`: The used integration, e.g. "divan" or "criterion"
/// - `name`: The name of the benchmark
/// - `uri`: The URI of the benchmark
/// - `iters_per_round`: The number of iterations for each round (=sample_size), e.g. `[1, 2, 3]` (variable) or `[2, 2, 2, 2]` (constant).
/// - `times_per_round_ns`: The measured time for each round in nanoseconds, e.g. `[1000, 2000, 3000]`
/// - `max_time_ns`: The time limit for the benchmark in nanoseconds (if defined)
///
/// # Pseudo-code
///
/// ```text
/// let sample_count = /* The number of executions for the same benchmark. */
/// let sample_size = iters_per_round = vec![/* The number of iterations within each sample. */];
/// for round in 0..sample_count {
///     let times_per_round_ns = 0;
///     for iteration in 0..sample_size[round] {
///         run_benchmark();
///         times_per_round_ns += /* measured execution time */;
///     }
/// }
/// ```
///
pub fn collect_raw_walltime_results(
    scope: &str,
    name: String,
    uri: String,
    iters_per_round: Vec<u128>,
    times_per_round_ns: Vec<u128>,
    max_time_ns: Option<u128>,
) {
    if !crate::utils::running_with_codspeed_runner() {
        return;
    }
    let workspace_root = std::env::var("CODSPEED_CARGO_WORKSPACE_ROOT").map(PathBuf::from);
    let Ok(workspace_root) = workspace_root else {
        eprintln!("codspeed failed to get workspace root. skipping");
        return;
    };
    let data = RawWallTimeData::from_runtime_data(
        name,
        uri,
        iters_per_round,
        times_per_round_ns,
        max_time_ns,
    );
    data.dump_to_results(&workspace_root, scope);
}

// FIXME: This assumes that the cargo target dir is `target`, and duplicates information with
// `cargo-codspeed::helpers::get_codspeed_target_dir`
pub fn get_raw_result_dir_from_workspace_root(workspace_root: &Path) -> PathBuf {
    workspace_root
        .join("target")
        .join("codspeed")
        .join("walltime")
        .join("raw_results")
}
