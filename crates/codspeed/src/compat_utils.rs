use crate::codspeed::CodSpeed;
use crate::instrument_hooks::InstrumentHooks;
use std::time::{Duration, Instant};

/// Runs multiple rounds of a benchmark based on CODSPEED_RUNNER_MODE.
///
/// # Important
/// `start_benchmark()` and `end_benchmark()` are called on the OUTSIDE because they
/// clear CPU caches. This is expensive and should only happen once per benchmark.
/// Inside the loop, we use `toggle_collect()` to pause/resume data collection
/// between iterations without clearing caches.
///
/// # Arguments
/// * `codspeed` - The CodSpeed instance to use for benchmarking
/// * `uri` - The benchmark identifier/URI
/// * `run_iteration` - Closure that runs a single benchmark iteration.
///   Should call `toggle_collect()` to resume/pause collection
///   around the measured code.
pub fn run_rounds(codspeed: &mut CodSpeed, uri: &str, mut run_iteration: impl FnMut()) {
    let (max_rounds, max_duration) = match std::env::var("CODSPEED_RUNNER_MODE").as_deref() {
        Ok("simulation") | Ok("instrumentation") => (None, Some(Duration::from_millis(100))),
        Ok("memory") => (Some(1), None),
        Ok(m) => unreachable!("Invalid runner mode: {m}"),
        Err(err) => panic!("Failed to get runner mode: {err}"),
    };

    let mut rounds = 0;
    let rounds_start_time = Instant::now();

    // Start benchmark ONCE - this clears CPU caches
    codspeed.start_benchmark(uri);
    InstrumentHooks::toggle_collect(); // Pause collection before first iteration

    loop {
        rounds += 1;

        run_iteration();

        let within_rounds = max_rounds.map_or(true, |max| rounds < max);
        let within_duration = max_duration.map_or(true, |max| rounds_start_time.elapsed() < max);

        if !(within_rounds && within_duration) {
            break;
        }
    }

    // End benchmark ONCE
    codspeed.end_benchmark();
}
