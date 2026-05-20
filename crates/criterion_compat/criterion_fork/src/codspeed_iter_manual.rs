//! CodSpeed addition: manual control over benchmark sampling.
//!
//! `iter_manual_unstable` lets the user pin down the exact number of measurement
//! rounds and iterations per round, bypassing criterion's adaptive sampler. See
//! `routine.rs::sample` for the short-circuit that picks up the result.

use std::time::{Duration, Instant};

use codspeed::instrument_hooks::InstrumentHooks;

#[cfg(feature = "async")]
use crate::async_executor::AsyncExecutor;
use crate::black_box;
use crate::measurement::Measurement;
#[cfg(feature = "async")]
use crate::AsyncBencher;
use crate::Bencher;

#[cfg(feature = "async")]
use std::future::Future;

/// Options for [`Bencher::iter_manual_unstable`].
#[derive(Debug, Clone, Copy)]
pub struct IterManualOptions {
    rounds: u64,
    iters: u64,
    warmup_rounds: u64,
}

impl Default for IterManualOptions {
    fn default() -> Self {
        Self {
            rounds: 1,
            iters: 1,
            warmup_rounds: 0,
        }
    }
}

impl IterManualOptions {
    /// Start with defaults: 1 round, 1 iteration per round, 0 warmup rounds.
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of measurement rounds (each produces one sample).
    #[must_use]
    pub fn rounds(mut self, rounds: u64) -> Self {
        self.rounds = rounds;
        self
    }

    /// Number of routine invocations inside a measurement round.
    #[must_use]
    pub fn iters(mut self, iters: u64) -> Self {
        self.iters = iters;
        self
    }

    /// Number of unmeasured warmup rounds run before measurement starts.
    #[must_use]
    pub fn warmup(mut self, warmup_rounds: u64) -> Self {
        self.warmup_rounds = warmup_rounds;
        self
    }
}

/// Captured output of a manual run. Stored on the `Bencher` and read by
/// `routine.rs::sample` to short-circuit the adaptive sampler.
pub(crate) struct ManualMeasurement {
    /// One entry per measurement round, in the units of `Measurement::to_f64`.
    pub samples: Vec<f64>,
    /// Number of routine invocations per round.
    pub iterations: u64,
}

impl<'a, M: Measurement> Bencher<'a, M> {
    /// Run `routine` with a precise schedule: `opts.rounds` measurement rounds
    /// (each producing one sample), each consisting of `opts.iters` calls to
    /// `routine`. Optionally preceded by `opts.warmup_rounds` unmeasured rounds
    /// of the same shape.
    ///
    /// This bypasses criterion's adaptive sampler entirely: the schedule you
    /// pass is exactly what runs.
    ///
    /// **Unstable.** This API is still under development and its name,
    /// signature, and behavior may change in future releases.
    #[inline(never)]
    pub fn iter_manual_unstable<O, R>(&mut self, opts: IterManualOptions, routine: R)
    where
        R: FnMut() -> O,
    {
        self.__codspeed_root_frame__iter_manual_unstable(opts, routine);
    }

    #[inline(never)]
    #[allow(missing_docs, non_snake_case)]
    pub fn __codspeed_root_frame__iter_manual_unstable<O, R>(
        &mut self,
        opts: IterManualOptions,
        mut routine: R,
    ) where
        R: FnMut() -> O,
    {
        self.iterated = true;

        for _ in 0..opts.warmup_rounds {
            for _ in 0..opts.iters {
                black_box(routine());
            }
        }

        self.elapsed_time = Duration::ZERO;
        let mut samples = Vec::with_capacity(opts.rounds as usize);
        for _ in 0..opts.rounds {
            let bench_start = InstrumentHooks::current_timestamp();
            let round_start = Instant::now();
            let start = self.measurement.start();
            for _ in 0..opts.iters {
                black_box(routine());
            }
            let value = self.measurement.end(start);
            self.elapsed_time += round_start.elapsed();
            let bench_end = InstrumentHooks::current_timestamp();
            InstrumentHooks::instance().add_benchmark_timestamps(bench_start, bench_end);

            samples.push(self.measurement.to_f64(&value));
        }

        self.codspeed_manual = Some(ManualMeasurement {
            samples,
            iterations: opts.iters,
        });
    }
}

#[cfg(feature = "async")]
impl<'a, 'b, A: AsyncExecutor, M: Measurement> AsyncBencher<'a, 'b, A, M> {
    /// Async/await variant of [`Bencher::iter_manual_unstable`]. Bypasses
    /// criterion's adaptive sampler and runs the exact schedule you pass.
    ///
    /// **Unstable.** This API is still under development and its name,
    /// signature, and behavior may change in future releases.
    #[inline(never)]
    pub fn iter_manual_unstable<O, R, F>(&mut self, opts: IterManualOptions, routine: R)
    where
        R: FnMut() -> F,
        F: Future<Output = O>,
    {
        self.__codspeed_root_frame__iter_manual_unstable(opts, routine);
    }

    #[inline(never)]
    #[allow(missing_docs, non_snake_case)]
    pub fn __codspeed_root_frame__iter_manual_unstable<O, R, F>(
        &mut self,
        opts: IterManualOptions,
        mut routine: R,
    ) where
        R: FnMut() -> F,
        F: Future<Output = O>,
    {
        let AsyncBencher { b, runner } = self;
        runner.block_on(async {
            b.iterated = true;

            for _ in 0..opts.warmup_rounds {
                for _ in 0..opts.iters {
                    black_box(routine().await);
                }
            }

            b.elapsed_time = Duration::ZERO;
            let mut samples = Vec::with_capacity(opts.rounds as usize);
            for _ in 0..opts.rounds {
                let bench_start = InstrumentHooks::current_timestamp();
                let round_start = Instant::now();
                let start = b.measurement.start();
                for _ in 0..opts.iters {
                    black_box(routine().await);
                }
                let value = b.measurement.end(start);
                b.elapsed_time += round_start.elapsed();
                let bench_end = InstrumentHooks::current_timestamp();
                InstrumentHooks::instance().add_benchmark_timestamps(bench_start, bench_end);

                samples.push(b.measurement.to_f64(&value));
            }

            b.codspeed_manual = Some(ManualMeasurement {
                samples,
                iterations: opts.iters,
            });
        });
    }
}
