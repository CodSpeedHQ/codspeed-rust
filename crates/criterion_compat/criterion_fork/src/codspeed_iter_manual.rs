//! CodSpeed addition: manual control over benchmark iteration counts.
//!
//! `iter_manual_setup*` lets the user pin down the exact number of measurement
//! rounds, inner iterations per round, and warmup rounds. It bypasses
//! Criterion's adaptive sampler entirely — see `routine.rs::sample` for the
//! short-circuit.

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

/// Options for the [`iter_manual_setup`](Bencher::iter_manual_setup) family.
///
/// Built with a consuming-self builder to match the rest of criterion's
/// configuration APIs (e.g. [`Criterion`](crate::Criterion)).
#[derive(Debug, Clone, Copy)]
pub struct IterManualOptions {
    rounds: u64,
    iterations: u64,
    warmup_iterations: u64,
}

impl Default for IterManualOptions {
    fn default() -> Self {
        Self {
            rounds: 1,
            iterations: 1,
            warmup_iterations: 0,
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
    pub fn iters(mut self, iterations: u64) -> Self {
        self.iterations = iterations;
        self
    }

    /// Number of unmeasured warmup iterations run before measurement starts.
    #[must_use]
    pub fn warmup(mut self, warmup_iterations: u64) -> Self {
        self.warmup_iterations = warmup_iterations;
        self
    }
}

/// Captured output of a manual run. Stored on the `Bencher` and read by
/// `routine.rs::sample` to short-circuit the adaptive sampler.
pub(crate) struct ManualMeasurement {
    /// One entry per measurement round, in nanoseconds (or whatever the
    /// measurement's `to_f64` returns).
    pub samples: Vec<f64>,
    /// Number of inner iterations per round.
    pub iterations: u64,
}

impl<'a, M: Measurement> Bencher<'a, M> {
    /// Run `routine` exactly `opts.iterations` times inside each of `opts.rounds`
    /// measurement rounds, with a `setup` closure producing fresh input for
    /// each round (outside the measured region). Optionally preceded by
    /// `opts.warmup_rounds` unmeasured rounds.
    ///
    /// Criterion's adaptive sampler is bypassed for this benchmark.
    #[inline(never)]
    pub fn iter_manual_setup<I, O, S, R>(&mut self, opts: IterManualOptions, setup: S, routine: R)
    where
        S: FnMut() -> I,
        R: FnMut(&mut I) -> O,
    {
        self.__codspeed_root_frame__iter_manual_setup_teardown(opts, setup, routine, |_| ());
    }

    /// Like [`iter_manual_setup`](Self::iter_manual_setup), with a `teardown`
    /// closure called after each round, outside the measured region.
    #[inline(never)]
    pub fn iter_manual_setup_teardown<I, O, S, R, T>(
        &mut self,
        opts: IterManualOptions,
        setup: S,
        routine: R,
        teardown: T,
    ) where
        S: FnMut() -> I,
        R: FnMut(&mut I) -> O,
        T: FnMut(I),
    {
        self.__codspeed_root_frame__iter_manual_setup_teardown(opts, setup, routine, teardown);
    }

    #[inline(never)]
    #[allow(missing_docs, non_snake_case)]
    pub fn __codspeed_root_frame__iter_manual_setup_teardown<I, O, S, R, T>(
        &mut self,
        opts: IterManualOptions,
        mut setup: S,
        mut routine: R,
        mut teardown: T,
    ) where
        S: FnMut() -> I,
        R: FnMut(&mut I) -> O,
        T: FnMut(I),
    {
        self.iterated = true;

        let mut elapsed_time = Duration::ZERO;

        for _ in 0..opts.warmup_iterations {
            let mut input = black_box(setup());
            for _ in 0..opts.iterations {
                black_box(routine(&mut input));
            }
            teardown(input);
        }

        let mut samples = Vec::with_capacity(opts.rounds as usize);
        for _ in 0..opts.rounds {
            let mut input = black_box(setup());

            let bench_start = InstrumentHooks::current_timestamp();
            let round_start = Instant::now();
            let start = self.measurement.start();
            for _ in 0..opts.iterations {
                black_box(routine(&mut input));
            }
            let value = self.measurement.end(start);
            elapsed_time += round_start.elapsed();
            let bench_end = InstrumentHooks::current_timestamp();
            InstrumentHooks::instance().add_benchmark_timestamps(bench_start, bench_end);

            teardown(input);
            samples.push(self.measurement.to_f64(&value));
        }

        self.elapsed_time = elapsed_time;

        self.codspeed_manual = Some(ManualMeasurement {
            samples,
            iterations: opts.iterations,
        });
    }
}

#[cfg(feature = "async")]
impl<'a, 'b, A: AsyncExecutor, M: Measurement> AsyncBencher<'a, 'b, A, M> {
    /// Async/await variant of [`Bencher::iter_manual_setup`].
    #[inline(never)]
    pub fn iter_manual_setup<I, O, S, R, F>(
        &mut self,
        opts: IterManualOptions,
        setup: S,
        routine: R,
    ) where
        S: FnMut() -> I,
        R: FnMut(&mut I) -> F,
        F: Future<Output = O>,
    {
        self.__codspeed_root_frame__iter_manual_setup_teardown(opts, setup, routine, |_| {
            std::future::ready(())
        });
    }

    /// Async/await variant of [`Bencher::iter_manual_setup_teardown`].
    #[inline(never)]
    pub fn iter_manual_setup_teardown<I, O, S, R, T, RF, TF>(
        &mut self,
        opts: IterManualOptions,
        setup: S,
        routine: R,
        teardown: T,
    ) where
        S: FnMut() -> I,
        R: FnMut(&mut I) -> RF,
        T: FnMut(I) -> TF,
        RF: Future<Output = O>,
        TF: Future<Output = ()>,
    {
        self.__codspeed_root_frame__iter_manual_setup_teardown(opts, setup, routine, teardown);
    }

    #[inline(never)]
    #[allow(missing_docs, non_snake_case)]
    pub fn __codspeed_root_frame__iter_manual_setup_teardown<I, O, S, R, T, RF, TF>(
        &mut self,
        opts: IterManualOptions,
        mut setup: S,
        mut routine: R,
        mut teardown: T,
    ) where
        S: FnMut() -> I,
        R: FnMut(&mut I) -> RF,
        T: FnMut(I) -> TF,
        RF: Future<Output = O>,
        TF: Future<Output = ()>,
    {
        let AsyncBencher { b, runner } = self;
        runner.block_on(async {
            b.iterated = true;

            let mut elapsed_time = Duration::ZERO;

            for _ in 0..opts.warmup_rounds {
                let mut input = black_box(setup());
                for _ in 0..opts.iterations {
                    black_box(routine(&mut input).await);
                }
                teardown(input).await;
            }

            let mut samples = Vec::with_capacity(opts.rounds as usize);
            for _ in 0..opts.rounds {
                let mut input = black_box(setup());

                let bench_start = InstrumentHooks::current_timestamp();
                let round_start = Instant::now();
                let start = b.measurement.start();
                for _ in 0..opts.iterations {
                    black_box(routine(&mut input).await);
                }
                let value = b.measurement.end(start);
                elapsed_time += round_start.elapsed();
                let bench_end = InstrumentHooks::current_timestamp();
                InstrumentHooks::instance().add_benchmark_timestamps(bench_start, bench_end);

                teardown(input).await;
                samples.push(b.measurement.to_f64(&value));
            }

            b.elapsed_time = elapsed_time;

            b.codspeed_manual = Some(ManualMeasurement {
                samples,
                iterations: opts.iterations,
            });
        });
    }
}
