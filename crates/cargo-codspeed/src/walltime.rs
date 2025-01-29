use crate::{benchmark_metadata::BenchmarkMetadata, prelude::*};
use serde::{Deserialize, Serialize};
use statrs::statistics::{Data, Distribution, Max, Min, OrderStatistics};
use std::{iter::zip, path::Path};

#[derive(Debug, Serialize, Deserialize)]
pub struct BenchmarkSample {
    iters: Vec<f64>,
    times: Vec<f64>,
}

pub fn parse_sample_json(path: &Path) -> Result<BenchmarkSample> {
    let file = std::fs::File::open(path)?;
    let sample: BenchmarkSample = serde_json::from_reader(file)?;
    Ok(sample)
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

    rounds: u64,
    total_time: f64,
    iqr_outlier_rounds: u64,
    stdev_outlier_rounds: u64,
    iter_per_round: u64,
    warmup_iters: u64,
}

const IQR_OUTLIER_FACTOR: f64 = 1.5;
const STDEV_OUTLIER_FACTOR: f64 = 3.0;

impl From<BenchmarkSample> for BenchmarkStats {
    fn from(value: BenchmarkSample) -> Self {
        let times_ns: Vec<f64> = zip(value.times.iter(), value.iters.iter())
            .map(|(times, iter)| times / iter)
            .collect();

        let mut data = Data::new(times_ns.clone());
        let rounds = data.len() as u64;
        let total_time = value.times.iter().sum();

        let mean_ns = data.mean().unwrap();
        let stdev_ns = data.std_dev().unwrap();

        let q1_ns = data.quantile(0.25);
        let median_ns = data.median();
        let q3_ns = data.quantile(0.75);

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

        // TODO: fill with correct data, maybe adding a `total_iters` field to `BenchmarkStats` is better
        let iter_per_round = value.iters.first().cloned().unwrap_or(0.0) as u64;
        let warmup_iters = 0;

        BenchmarkStats {
            min_ns,
            max_ns,
            mean_ns,
            stdev_ns,
            q1_ns,
            median_ns,
            q3_ns,
            rounds,
            total_time,
            iqr_outlier_rounds,
            stdev_outlier_rounds,
            iter_per_round,
            warmup_iters,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
/// Configuration for the benchmark
///
/// At the moment it is not used
struct BenchmarkConfig {
    warmup_time_ns: f64,
    min_round_time_ns: f64,
    max_time_ns: f64,
    max_rounds: Option<u64>,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        BenchmarkConfig {
            warmup_time_ns: 1_000_000_000.0,
            min_round_time_ns: 1_000_000.0, // TODO: use clock_info
            max_time_ns: 3_000_000_000.0,
            max_rounds: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WalltimeBenchmark {
    #[serde(flatten)]
    metadata: BenchmarkMetadata,

    config: BenchmarkConfig,
    stats: BenchmarkStats,
}

// TODO: change to fetch metadata from build step, instead of hardcoding the uri
impl From<(String, BenchmarkSample)> for WalltimeBenchmark {
    fn from((name, sample): (String, BenchmarkSample)) -> Self {
        WalltimeBenchmark {
            metadata: BenchmarkMetadata::new(name.clone(), format!("file://{}", name)),
            config: BenchmarkConfig::default(),
            stats: BenchmarkStats::from(sample),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Instrument {
    #[serde(rename = "type")]
    type_: String, // TODO: use enum for this
}

// TODO: change this to a trait to allow different instruments
impl Default for Instrument {
    fn default() -> Self {
        Instrument {
            type_: "walltime".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Results {
    instrument: Instrument,
    // TODO: here use a struct that implements a `BenchmarkMetadata` trait, which is implemented by `WalltimeBenchmark`
    benchmarks: Vec<WalltimeBenchmark>,
}

impl Results {
    pub fn new(benchmarks: Vec<WalltimeBenchmark>) -> Self {
        Results {
            instrument: Instrument::default(),
            benchmarks,
        }
    }
}
