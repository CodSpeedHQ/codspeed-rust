//! CodSpeed addition: manual control over benchmark iteration counts.
//!
//! `iter_manual*` lets the user pin down the exact number of measurement
//! rounds, inner iterations per round, and warmup rounds. It bypasses
//! Criterion's adaptive sampler entirely — see `routine.rs::sample` for the
//! short-circuit.

use std::time::Instant;

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

/// Options for the [`iter_manual`](Bencher::iter_manual) family.
#[derive(Debug, Clone, Copy)]
pub struct IterManualOptions {
    /// Number of measurement rounds (each produces one sample).
    pub rounds: u64,
    /// Number of routine invocations inside the measured region of each round.
    pub iterations: u64,
    /// Number of unmeasured warmup rounds run before measurement starts.
    pub warmup_rounds: u64,
}

impl IterManualOptions {
    /// Build options with the given rounds and iterations; `warmup_rounds` defaults to 0.
    pub fn new(rounds: u64, iterations: u64) -> Self {
        Self {
            rounds,
            iterations,
            warmup_rounds: 0,
        }
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
    /// measurement rounds, optionally preceded by `opts.warmup_rounds` unmeasured rounds.
    ///
    /// Criterion's adaptive sampler is bypassed for this benchmark.
    #[inline(never)]
    pub fn iter_manual<O, R>(&mut self, opts: IterManualOptions, mut routine: R)
    where
        R: FnMut() -> O,
    {
        self.iter_manual_setup_teardown(opts, || (), |_| routine(), |_| ());
    }

    /// Like [`iter_manual`](Self::iter_manual), with a `setup` closure producing
    /// fresh input for each round. The routine borrows the input mutably.
    #[inline(never)]
    pub fn iter_manual_setup<I, O, S, R>(&mut self, opts: IterManualOptions, setup: S, routine: R)
    where
        S: FnMut() -> I,
        R: FnMut(&mut I) -> O,
    {
        self.iter_manual_setup_teardown(opts, setup, routine, |_| ());
    }

    /// Like [`iter_manual_setup`](Self::iter_manual_setup), with a `teardown`
    /// closure called after each round, outside the measured region.
    #[inline(never)]
    pub fn iter_manual_setup_teardown<I, O, S, R, T>(
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

        let bench_start = InstrumentHooks::current_timestamp();
        let time_start = Instant::now();

        for _ in 0..opts.warmup_rounds {
            let mut input = black_box(setup());
            for _ in 0..opts.iterations {
                black_box(routine(&mut input));
            }
            teardown(input);
        }

        let mut samples = Vec::with_capacity(opts.rounds as usize);
        for _ in 0..opts.rounds {
            let mut input = black_box(setup());
            let start = self.measurement.start();
            for _ in 0..opts.iterations {
                black_box(routine(&mut input));
            }
            let value = self.measurement.end(start);
            teardown(input);
            samples.push(self.measurement.to_f64(&value));
        }

        self.elapsed_time = time_start.elapsed();
        let bench_end = InstrumentHooks::current_timestamp();
        InstrumentHooks::instance().add_benchmark_timestamps(bench_start, bench_end);

        self.codspeed_manual = Some(ManualMeasurement {
            samples,
            iterations: opts.iterations,
        });
    }
}

#[cfg(feature = "async")]
impl<'a, 'b, A: AsyncExecutor, M: Measurement> AsyncBencher<'a, 'b, A, M> {
    /// Async/await variant of [`Bencher::iter_manual`].
    #[inline(never)]
    pub fn iter_manual<O, R, F>(&mut self, opts: IterManualOptions, mut routine: R)
    where
        R: FnMut() -> F,
        F: Future<Output = O>,
    {
        self.iter_manual_setup_teardown(opts, || (), |_| routine(), |_| std::future::ready(()));
    }

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
        self.iter_manual_setup_teardown(opts, setup, routine, |_| std::future::ready(()));
    }

    /// Async/await variant of [`Bencher::iter_manual_setup_teardown`].
    #[inline(never)]
    pub fn iter_manual_setup_teardown<I, O, S, R, T, RF, TF>(
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

            let bench_start = InstrumentHooks::current_timestamp();
            let time_start = Instant::now();

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
                let start = b.measurement.start();
                for _ in 0..opts.iterations {
                    black_box(routine(&mut input).await);
                }
                let value = b.measurement.end(start);
                teardown(input).await;
                samples.push(b.measurement.to_f64(&value));
            }

            b.elapsed_time = time_start.elapsed();
            let bench_end = InstrumentHooks::current_timestamp();
            InstrumentHooks::instance().add_benchmark_timestamps(bench_start, bench_end);

            b.codspeed_manual = Some(ManualMeasurement {
                samples,
                iterations: opts.iterations,
            });
        });
    }
}
