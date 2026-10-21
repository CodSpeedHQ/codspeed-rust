use crate::measurement;
use colored::Colorize;
use std::ffi::CString;
use std::time::{Duration, Instant};

pub use std::hint::black_box;

/// Every benchmark runs at least this many unmeasured iterations before the measured one.
pub const WARMUP_MIN_ITERATIONS: u32 = 1;
/// Upper bound on unmeasured iterations, even when [`WARMUP_MAX_DURATION`] is not reached.
pub const WARMUP_MAX_ITERATIONS: u32 = 5;
/// Once the warmup has run for this long, no further unmeasured iteration is started.
/// The clock runs under the simulator, so this is simulator time, not native time.
pub const WARMUP_MAX_DURATION: Duration = Duration::from_secs(1);

/// Progress of the unmeasured iterations that precede a measured iteration.
///
/// Instrumentation (and therefore cache simulation) is already running while these
/// iterations execute, so the measured iteration starts with the caches populated by
/// the benchmark itself instead of with the flushed state a fresh
/// `CALLGRIND_START_INSTRUMENTATION` would leave behind.
struct Warmup {
    started: Instant,
    iterations: u32,
}

impl Warmup {
    fn begin() -> Self {
        measurement::start_instrumentation();
        Self {
            started: Instant::now(),
            iterations: 0,
        }
    }

    fn should_iterate(&self) -> bool {
        if self.iterations < WARMUP_MIN_ITERATIONS {
            return true;
        }
        if self.iterations >= WARMUP_MAX_ITERATIONS {
            return false;
        }
        self.started.elapsed() < WARMUP_MAX_DURATION
    }
}

pub fn display_native_harness() {
    eprintln!("Harness: codspeed v{}", env!("CARGO_PKG_VERSION"),);
}

#[derive(PartialEq)]
pub enum InstrumentationStatus {
    /// Instrumentation detected via inline assembly directly
    Valgrind,
    /// Instrumentation detected via InstrumentHooks
    InstrumentHooks(&'static crate::instrument_hooks::InstrumentHooks),
    NotInstrumented,
}

impl InstrumentationStatus {
    pub fn is_instrumented(&self) -> bool {
        *self != InstrumentationStatus::NotInstrumented
    }
}

pub struct CodSpeed {
    benchmarked: Vec<String>,
    current_benchmark: CString,
    group_stack: Vec<String>,
    instrumentation_status: InstrumentationStatus,
    warmup: Option<Warmup>,
    warmup_iterations: u32,
}

impl CodSpeed {
    pub fn new() -> Self {
        use crate::instrument_hooks::InstrumentHooks;
        let instrumentation_status = {
            // Always initialize InstrumentHooks for environment collection,
            // even when using Valgrind for the actual measurements.
            let hooks_instance = InstrumentHooks::instance();

            // We bypass InstrumentHooks if we detect Valgrind via inline assembly
            // Until we can reliably get rid of the inline assembly without causing breaking
            // changes in CPU simulation measurements by switching to InstrumentHooks only, we need
            // to keep this separation.
            if measurement::is_instrumented() {
                InstrumentationStatus::Valgrind
            } else if hooks_instance.is_instrumented() {
                InstrumentationStatus::InstrumentHooks(hooks_instance)
            } else {
                InstrumentationStatus::NotInstrumented
            }
        };

        if !instrumentation_status.is_instrumented() {
            eprintln!(
                "{} codspeed is enabled, but no performance measurement will be made since it's running in an unknown environment.",
                "NOTICE:".to_string().bold()
            );
        };

        measurement::set_metadata();

        Self {
            benchmarked: Vec::new(),
            current_benchmark: CString::new("").expect("CString::new failed"),
            group_stack: Vec::new(),
            instrumentation_status,
            warmup: None,
            warmup_iterations: 0,
        }
    }

    pub fn push_group(&mut self, group: &str) {
        self.group_stack.push(group.to_string());
    }

    pub fn pop_group(&mut self) {
        self.group_stack.pop();
    }

    /// Returns whether the harness should run one more unmeasured iteration before the
    /// measured one. The first call switches instrumentation on; the following
    /// [`CodSpeed::start_benchmark`] then only zeroes the counters, keeping the simulated
    /// cache state the warmup iterations built up.
    ///
    /// Call it before every candidate iteration:
    ///
    /// ```ignore
    /// while codspeed.warmup_iteration() {
    ///     black_box(routine());
    /// }
    /// codspeed.start_benchmark(uri);
    /// black_box(routine());
    /// codspeed.end_benchmark();
    /// ```
    #[inline(always)]
    pub fn warmup_iteration(&mut self) -> bool {
        let warmup = self.warmup.get_or_insert_with(Warmup::begin);
        if !warmup.should_iterate() {
            return false;
        }
        warmup.iterations += 1;
        true
    }

    #[inline(always)]
    pub fn start_benchmark(&mut self, name: &str) {
        self.current_benchmark = CString::new(name).expect("CString::new failed");

        if let InstrumentationStatus::InstrumentHooks(hooks_instance) = &self.instrumentation_status
        {
            let _ = hooks_instance.start_benchmark();
        }

        // We intentionally do this no matter the instrumentation status.
        // The overhead in case we are not running valgrind is extremely low, and we do not want to
        // add a conditionnal branch to valgrind measurements.
        //
        // Everything executed before `zero_statistics` is discarded from the measurement, so the
        // branch below is invisible to the counters.
        match self.warmup.take() {
            Some(warmup) => self.warmup_iterations = warmup.iterations,
            None => {
                self.warmup_iterations = 0;
                measurement::start_instrumentation();
            }
        }
        measurement::zero_statistics();
    }

    #[inline(always)]
    pub fn end_benchmark(&mut self) {
        // We intentionally do this no matter the instrumentation status.
        // The overhead in case we are not running valgrind is extremely low, and we do not want to
        // add a conditionnal branch to valgrind measurements.
        measurement::stop(&self.current_benchmark);
        if let InstrumentationStatus::InstrumentHooks(hooks_instance) = &self.instrumentation_status
        {
            let _ = hooks_instance.stop_benchmark();
            let _ =
                hooks_instance.set_executed_benchmark(&self.current_benchmark.to_string_lossy());
        }
        self.benchmarked
            .push(self.current_benchmark.to_str().unwrap().to_string());

        let action_str = if self.instrumentation_status.is_instrumented() {
            "Measured"
        } else {
            "Checked"
        };

        let group_str = if self.group_stack.is_empty() {
            "".to_string()
        } else {
            format!(" (group: {})", self.group_stack.join("/"))
        };
        println!(
            "{}: {}{} (warmup: {} iterations)",
            action_str,
            self.current_benchmark.to_string_lossy(),
            group_str,
            self.warmup_iterations,
        );
    }
}

impl Default for CodSpeed {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warmup_runs_between_min_and_max_iterations_per_benchmark() {
        let mut codspeed = CodSpeed::new();
        for _ in 0..2 {
            let mut iterations = 0;
            while codspeed.warmup_iteration() {
                iterations += 1;
            }
            assert!(iterations >= WARMUP_MIN_ITERATIONS);
            assert!(iterations <= WARMUP_MAX_ITERATIONS);
            codspeed.start_benchmark("warmup_test");
            assert_eq!(codspeed.warmup_iterations, iterations);
            codspeed.end_benchmark();
        }
    }

    #[test]
    fn measuring_without_warmup_reports_zero_warmup_iterations() {
        let mut codspeed = CodSpeed::new();
        codspeed.start_benchmark("no_warmup");
        assert_eq!(codspeed.warmup_iterations, 0);
        codspeed.end_benchmark();
    }
}
