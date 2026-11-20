use anyhow::{Context, Result};
use std::{
    collections::BTreeSet,
    io::{BufReader, Write},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use statrs::statistics::{Data, Distribution, Max, Min};

use crate::utils::generate_unique_id;

// TODO(COD-4241): go-runner in codspeed-go copies this file's percentile and round sampling code;
// make it call this crate instead.
const IQR_OUTLIER_FACTOR: f64 = 1.5;
const STDEV_OUTLIER_FACTOR: f64 = 3.0;
/// Above this, the rounds file keeps a uniformly random subsample of the rounds, in execution order.
const MAX_WALLTIME_ROUNDS_PER_BENCHMARK: usize = 100_000;

#[derive(Debug, Serialize, Deserialize)]
pub struct BenchmarkMetadata {
    pub name: String,
    pub uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct BenchmarkStats {
    min_ns: f64,
    max_ns: f64,
    mean_ns: f64,
    stdev_ns: f64,

    q1_ns: f64,
    median_ns: f64,
    q3_ns: f64,
    /// p1..p99 of the time per iteration
    #[serde(default, skip_serializing_if = "Option::is_none")]
    percentiles_ns: Option<Vec<f64>>,

    rounds: u64,
    total_time: f64,
    iqr_outlier_rounds: u64,
    stdev_outlier_rounds: u64,
    iter_per_round: u64,
    warmup_iters: u64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct BenchmarkConfig {
    warmup_time_ns: Option<f64>,
    min_round_time_ns: Option<f64>,
    max_time_ns: Option<f64>,
    max_rounds: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BenchmarkRounds {
    times_per_round_ns: Vec<u128>,
    iters_per_round: Vec<u128>,
}

#[derive(Debug, Serialize)]
struct UriRounds {
    uri: String,
    #[serde(flatten)]
    rounds: BenchmarkRounds,
}

/// Content of `walltime_rounds/<pid>.json.gz` in the profile folder
#[derive(Debug, Serialize)]
pub struct WalltimeRounds {
    benchmarks: Vec<UriRounds>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WalltimeBenchmark {
    #[serde(flatten)]
    metadata: BenchmarkMetadata,

    config: BenchmarkConfig,
    stats: BenchmarkStats,

    /// Rounds in execution order, capped at [`MAX_WALLTIME_ROUNDS_PER_BENCHMARK`]: fewer than
    /// `stats.rounds` when subsampled. Left out of the results file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    walltime_rounds: Option<BenchmarkRounds>,
}

impl WalltimeBenchmark {
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
        let data = WalltimeBenchmark::from_runtime_data(
            name,
            uri,
            iters_per_round,
            times_per_round_ns,
            max_time_ns,
        );
        data.dump_to_results(&workspace_root, scope);
    }

    pub fn from_runtime_data(
        name: String,
        uri: String,
        iters_per_round: Vec<u128>,
        times_per_round_ns: Vec<u128>,
        max_time_ns: Option<u128>,
    ) -> Self {
        let total_time = times_per_round_ns.iter().sum::<u128>() as f64 / 1_000_000_000.0;
        let mut time_per_iteration_per_round_ns: Vec<_> = times_per_round_ns
            .iter()
            .zip(&iters_per_round)
            .map(|(time_per_round, iter_per_round)| time_per_round / iter_per_round)
            .map(|t| t as f64)
            .collect::<Vec<f64>>();

        // statrs' `quantile` formula (R-8) on rounds sorted once: statrs itself runs an O(n) selection per call
        time_per_iteration_per_round_ns.sort_unstable_by(f64::total_cmp);
        let sorted = &time_per_iteration_per_round_ns;
        let percentiles_ns: Vec<f64> = (1..100)
            .map(|p| {
                let h = (sorted.len() as f64 + 1.0 / 3.0) * (p as f64 / 100.0) + 1.0 / 3.0;
                let hf = h as usize;
                if hf == 0 {
                    return sorted[0];
                }
                if hf >= sorted.len() {
                    return sorted[sorted.len() - 1];
                }
                let (a, b) = (sorted[hf - 1], sorted[hf]);
                a + (h - hf as f64) * (b - a)
            })
            .collect();
        let q1_ns = percentiles_ns[24];
        let median_ns = percentiles_ns[49];
        let q3_ns = percentiles_ns[74];

        let data = Data::new(time_per_iteration_per_round_ns);
        let rounds = data.len() as u64;

        let mean_ns = data.mean().unwrap();

        let stdev_ns = if data.len() < 2 {
            // std_dev() returns f64::NAN if data has less than two entries, so we have to
            // manually handle this case.
            0.0
        } else {
            data.std_dev().unwrap()
        };

        let iqr_ns = q3_ns - q1_ns;
        let iqr_outlier_rounds = data
            .iter()
            .filter(|&&t| {
                t < q1_ns - IQR_OUTLIER_FACTOR * iqr_ns || t > q3_ns + IQR_OUTLIER_FACTOR * iqr_ns
            })
            .count() as u64;

        let stdev_outlier_rounds = data
            .iter()
            .filter(|&&t| {
                t < mean_ns - STDEV_OUTLIER_FACTOR * stdev_ns
                    || t > mean_ns + STDEV_OUTLIER_FACTOR * stdev_ns
            })
            .count() as u64;

        let min_ns = data.min();
        let max_ns = data.max();

        // TODO(COD-1056): We currently only support single iteration count per round
        let iter_per_round =
            (iters_per_round.iter().sum::<u128>() / iters_per_round.len() as u128) as u64;
        let warmup_iters = 0; // FIXME: add warmup detection

        let stats = BenchmarkStats {
            min_ns,
            max_ns,
            mean_ns,
            stdev_ns,
            q1_ns,
            median_ns,
            q3_ns,
            percentiles_ns: Some(percentiles_ns),
            rounds,
            total_time,
            iqr_outlier_rounds,
            stdev_outlier_rounds,
            iter_per_round,
            warmup_iters,
        };

        let walltime_rounds = if times_per_round_ns.len() <= MAX_WALLTIME_ROUNDS_PER_BENCHMARK {
            BenchmarkRounds {
                times_per_round_ns,
                iters_per_round,
            }
        } else {
            // Floyd's sampling of distinct indices; the set iterates them in execution order
            let mut random = vec![0u8; 8 * MAX_WALLTIME_ROUNDS_PER_BENCHMARK];
            getrandom::fill(&mut random).expect("Failed to generate random bytes");
            let mut kept_indices = BTreeSet::new();
            let n = times_per_round_ns.len();
            for (j, r) in (n - MAX_WALLTIME_ROUNDS_PER_BENCHMARK..n).zip(random.chunks_exact(8)) {
                let t = (u64::from_ne_bytes(r.try_into().unwrap()) % (j as u64 + 1)) as usize;
                if !kept_indices.insert(t) {
                    kept_indices.insert(j);
                }
            }
            BenchmarkRounds {
                times_per_round_ns: kept_indices
                    .iter()
                    .map(|&i| times_per_round_ns[i])
                    .collect(),
                iters_per_round: kept_indices.iter().map(|&i| iters_per_round[i]).collect(),
            }
        };

        WalltimeBenchmark {
            metadata: BenchmarkMetadata { name, uri },
            config: BenchmarkConfig {
                max_time_ns: max_time_ns.map(|t| t as f64),
                ..Default::default()
            },
            stats,
            walltime_rounds: Some(walltime_rounds),
        }
    }

    fn dump_to_results(&self, workspace_root: &Path, scope: &str) {
        let output_dir = result_dir_from_workspace_root(workspace_root).join(scope);
        std::fs::create_dir_all(&output_dir).unwrap();
        let bench_id = generate_unique_id();
        let output_path = output_dir.join(format!("{bench_id}.json"));
        let mut writer = std::io::BufWriter::new(
            std::fs::File::create(&output_path).expect("Failed to create the file"),
        );
        serde_json::to_writer(&mut writer, self).expect("Failed to write the data");
        writer.flush().expect("Failed to flush the writer");
    }

    pub fn is_invalid(&self) -> bool {
        self.stats.min_ns < f64::EPSILON
    }

    pub fn name(&self) -> &str {
        &self.metadata.name
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Instrument {
    #[serde(rename = "type")]
    type_: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Creator {
    name: String,
    version: String,
    pid: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WalltimeResults {
    creator: Creator,
    instrument: Instrument,
    benchmarks: Vec<WalltimeBenchmark>,
}

impl WalltimeResults {
    pub fn collect_walltime_results(workspace_root: &Path) -> Result<Self> {
        // retrieve data from `{workspace_root}/target/codspeed/raw_results/{scope}/*.json
        let benchmarks = glob::glob(&format!(
            "{}/**/*.json",
            result_dir_from_workspace_root(workspace_root)
                .to_str()
                .unwrap(),
        ))?
        .map(|sample| -> Result<_> {
            let sample = sample?;
            serde_json::from_reader::<_, WalltimeBenchmark>(BufReader::new(std::fs::File::open(
                &sample,
            )?))
            .context("Failed to read benchmark data")
        })
        .collect::<Result<Vec<_>>>()?;

        Ok(WalltimeResults {
            instrument: Instrument {
                type_: "walltime".to_string(),
            },
            creator: Creator {
                name: "codspeed-rust".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                pid: std::process::id(),
            },
            benchmarks,
        })
    }

    pub fn clear(workspace_root: &Path) -> Result<()> {
        let raw_results_dir = result_dir_from_workspace_root(workspace_root);
        std::fs::remove_dir_all(&raw_results_dir).ok(); // ignore errors when the directory does not exist
        std::fs::create_dir_all(&raw_results_dir)
            .context("Failed to create raw_results directory")?;
        Ok(())
    }

    pub fn benchmarks(&self) -> &[WalltimeBenchmark] {
        &self.benchmarks
    }

    /// Moves the walltime rounds out of the benchmarks; `None` when no benchmark has any
    pub fn take_walltime_rounds(&mut self) -> Option<WalltimeRounds> {
        let benchmarks: Vec<_> = self
            .benchmarks
            .iter_mut()
            .filter_map(|bench| {
                Some(UriRounds {
                    uri: bench.metadata.uri.clone(),
                    rounds: bench.walltime_rounds.take()?,
                })
            })
            .collect();
        (!benchmarks.is_empty()).then_some(WalltimeRounds { benchmarks })
    }
}

// FIXME: This assumes that the cargo target dir is `target`, and duplicates information with
// `cargo-codspeed::helpers::get_codspeed_target_dir`
fn result_dir_from_workspace_root(workspace_root: &Path) -> PathBuf {
    workspace_root
        .join("target")
        .join("codspeed")
        .join("walltime")
        .join("raw_results")
}

#[cfg(test)]
mod tests {
    use super::*;

    const NAME: &str = "benchmark";
    const URI: &str = "test::benchmark";

    #[test]
    fn test_parse_single_benchmark() {
        let benchmark = WalltimeBenchmark::from_runtime_data(
            NAME.to_string(),
            URI.to_string(),
            vec![1],
            vec![42],
            None,
        );
        assert_eq!(benchmark.stats.stdev_ns, 0.);
        assert_eq!(benchmark.stats.min_ns, 42.);
        assert_eq!(benchmark.stats.max_ns, 42.);
        assert_eq!(benchmark.stats.mean_ns, 42.);
    }

    #[test]
    fn test_parse_bench_with_variable_iterations() {
        let iters_per_round = vec![1, 2, 3, 4, 5, 6];
        let total_rounds = iters_per_round.iter().sum::<u128>() as f64;

        let benchmark = WalltimeBenchmark::from_runtime_data(
            NAME.to_string(),
            URI.to_string(),
            iters_per_round,
            vec![42, 42 * 2, 42 * 3, 42 * 4, 42 * 5, 42 * 6],
            None,
        );

        assert_eq!(benchmark.stats.stdev_ns, 0.);
        assert_eq!(benchmark.stats.min_ns, 42.);
        assert_eq!(benchmark.stats.max_ns, 42.);
        assert_eq!(benchmark.stats.mean_ns, 42.);
        assert_eq!(
            benchmark.stats.total_time,
            42. * total_rounds / 1_000_000_000.0
        );
    }

    #[test]
    fn test_percentiles_match_statrs() {
        for rounds in [1u128, 2, 3, 10, 100] {
            // 1..=rounds in a shuffled execution order
            let times: Vec<u128> = (1..=rounds).map(|i| i * 37 % (rounds + 1)).collect();
            let benchmark = WalltimeBenchmark::from_runtime_data(
                NAME.to_string(),
                URI.to_string(),
                vec![1; times.len()],
                times.clone(),
                None,
            );

            let percentiles = benchmark.stats.percentiles_ns.unwrap();
            let mut data = Data::new(times.iter().map(|&t| t as f64).collect::<Vec<_>>());
            let expected: Vec<f64> = (1..100)
                .map(|p| statrs::statistics::OrderStatistics::percentile(&mut data, p))
                .collect();
            assert_eq!(percentiles, expected, "{rounds} rounds");
            assert!(percentiles.windows(2).all(|w| w[0] <= w[1]));
            assert_eq!(benchmark.stats.q1_ns, percentiles[24]);
            assert_eq!(benchmark.stats.median_ns, percentiles[49]);
            assert_eq!(benchmark.stats.q3_ns, percentiles[74]);
        }
    }

    #[test]
    fn test_walltime_rounds_capped_in_execution_order() {
        let rounds = MAX_WALLTIME_ROUNDS_PER_BENCHMARK + 50_000;
        let iters: Vec<u128> = (1..=rounds as u128).collect();
        let times: Vec<u128> = iters.iter().map(|i| i * 10).collect();
        let benchmark = WalltimeBenchmark::from_runtime_data(
            NAME.to_string(),
            URI.to_string(),
            iters,
            times,
            None,
        );

        assert_eq!(benchmark.stats.rounds, rounds as u64);
        let kept = benchmark.walltime_rounds.unwrap();
        assert_eq!(
            kept.times_per_round_ns.len(),
            MAX_WALLTIME_ROUNDS_PER_BENCHMARK
        );
        assert_eq!(
            kept.iters_per_round.len(),
            MAX_WALLTIME_ROUNDS_PER_BENCHMARK
        );
        // A random subsample, not the first rounds
        assert!(*kept.iters_per_round.last().unwrap() > MAX_WALLTIME_ROUNDS_PER_BENCHMARK as u128);
        assert!(kept.times_per_round_ns.windows(2).all(|w| w[0] < w[1]));
        assert!(kept
            .times_per_round_ns
            .iter()
            .zip(&kept.iters_per_round)
            .all(|(&t, &i)| t == i * 10 && (1..=rounds as u128).contains(&i)));
    }

    #[test]
    fn test_take_walltime_rounds() {
        // A dump from a codspeed version without percentiles nor walltime rounds
        let old_dump: WalltimeBenchmark = serde_json::from_value(serde_json::json!({
            "name": "old", "uri": "test::old", "config": {},
            "stats": {
                "min_ns": 1.0, "max_ns": 1.0, "mean_ns": 1.0, "stdev_ns": 0.0,
                "q1_ns": 1.0, "median_ns": 1.0, "q3_ns": 1.0, "rounds": 1, "total_time": 1e-9,
                "iqr_outlier_rounds": 0, "stdev_outlier_rounds": 0, "iter_per_round": 1, "warmup_iters": 0
            }
        }))
        .unwrap();
        let mut results = WalltimeResults {
            creator: Creator {
                name: "codspeed-rust".to_string(),
                version: "0.0.0".to_string(),
                pid: 1,
            },
            instrument: Instrument {
                type_: "walltime".to_string(),
            },
            benchmarks: vec![old_dump],
        };
        assert!(results.take_walltime_rounds().is_none());

        results
            .benchmarks
            .push(WalltimeBenchmark::from_runtime_data(
                NAME.to_string(),
                URI.to_string(),
                vec![1, 2],
                vec![30, 50],
                None,
            ));
        let rounds = serde_json::to_value(results.take_walltime_rounds().unwrap()).unwrap();
        assert_eq!(
            rounds,
            serde_json::json!({"benchmarks": [
                {"uri": URI, "times_per_round_ns": [30, 50], "iters_per_round": [1, 2]}
            ]})
        );
        let results = serde_json::to_value(&results).unwrap();
        assert!(results["benchmarks"][1].get("walltime_rounds").is_none());
        assert_eq!(
            results["benchmarks"][1]["stats"]["percentiles_ns"]
                .as_array()
                .unwrap()
                .len(),
            99
        );
        assert!(results["benchmarks"][0]["stats"]
            .get("percentiles_ns")
            .is_none());
    }
}
