use std::{
    io::Write,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BenchmarkMetadata {
    pub name: String,
    pub uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RawWallTimeData {
    #[serde(flatten)]
    pub metadata: BenchmarkMetadata,
    pub iter_per_round: u32,
    pub max_time_ns: Option<u128>,
    pub times_ns: Vec<u128>,
}

impl RawWallTimeData {
    fn from_runtime_data(
        name: String,
        uri: String,
        iter_per_round: u32,
        max_time_ns: Option<u128>,
        times_ns: Vec<u128>,
    ) -> Self {
        RawWallTimeData {
            metadata: BenchmarkMetadata { name, uri },
            iter_per_round,
            max_time_ns,
            times_ns,
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
pub fn collect_raw_walltime_results(
    scope: &str,
    name: String,
    uri: String,
    iter_per_round: u32,
    max_time_ns: Option<u128>,
    times_ns: Vec<u128>,
) {
    if std::env::var("CODSPEED_ENV").is_err() {
        return;
    }
    let workspace_root = std::env::var("CODSPEED_CARGO_WORKSPACE_ROOT").map(PathBuf::from);
    let Ok(workspace_root) = workspace_root else {
        eprintln!("codspeed failed to get workspace root. skipping");
        return;
    };
    let data = RawWallTimeData::from_runtime_data(name, uri, iter_per_round, max_time_ns, times_ns);
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
