use codspeed::codspeed::{black_box, CodSpeed};
use codspeed::compat_utils;
use codspeed::instrument_hooks::InstrumentHooks;
use colored::Colorize;
use criterion::BatchSize;

#[cfg(feature = "async")]
use criterion::async_executor::AsyncExecutor;
#[cfg(feature = "async")]
use std::future::Future;

pub struct Bencher<'a> {
    codspeed: &'a mut CodSpeed,
    uri: String,
}

#[allow(clippy::needless_lifetimes)]
impl<'a> Bencher<'a> {
    pub(crate) fn new(codspeed: &'a mut CodSpeed, uri: String) -> Self {
        Bencher { codspeed, uri }
    }

    #[inline(never)]
    pub fn iter<O, R>(&mut self, mut routine: R)
    where
        R: FnMut() -> O,
    {
        // NOTE: this structure hardens our benchmark against dead code elimination
        // https://godbolt.org/z/KnYeKMd1o

        // Warmup runs
        for _ in 0..codspeed::codspeed::WARMUP_RUNS {
            black_box(routine());
        }

        // Multiple measured rounds
        compat_utils::run_rounds(self.codspeed, self.uri.as_str(), || {
            InstrumentHooks::toggle_collect(); // Resume collection
            let output = routine();
            InstrumentHooks::toggle_collect(); // Pause collection
            black_box(output);
        });
    }

    #[inline(never)]
    pub fn iter_custom<R, MV>(&mut self, mut _routine: R)
    where
        R: FnMut(u64) -> MV,
    {
        println!(
            "{} {} (CodSpeed doesn't support custom iterations)",
            "Skipping:".to_string().yellow(),
            self.uri.yellow(),
        );
    }

    #[inline(never)]
    pub fn iter_batched<I, O, S, R>(&mut self, mut setup: S, mut routine: R, _size: BatchSize)
    where
        S: FnMut() -> I,
        R: FnMut(I) -> O,
    {
        // Warmup runs
        for _ in 0..codspeed::codspeed::WARMUP_RUNS {
            let input = black_box(setup());
            let output = routine(input);
            drop(black_box(output));
        }

        // Multiple measured rounds
        compat_utils::run_rounds(self.codspeed, self.uri.as_str(), || {
            let input = setup(); // Setup runs while collection is paused
            InstrumentHooks::toggle_collect(); // Resume collection
            let output = routine(input);
            InstrumentHooks::toggle_collect(); // Pause collection
            black_box(output);
        });
    }

    pub fn iter_with_setup<I, O, S, R>(&mut self, setup: S, routine: R)
    where
        S: FnMut() -> I,
        R: FnMut(I) -> O,
    {
        self.iter_batched(setup, routine, BatchSize::PerIteration);
    }

    pub fn iter_with_large_drop<O, R>(&mut self, mut routine: R)
    where
        R: FnMut() -> O,
    {
        self.iter_batched(|| (), |_| routine(), BatchSize::SmallInput);
    }

    pub fn iter_with_large_setup<I, O, S, R>(&mut self, setup: S, routine: R)
    where
        S: FnMut() -> I,
        R: FnMut(I) -> O,
    {
        self.iter_batched(setup, routine, BatchSize::NumBatches(1));
    }

    #[inline(never)]
    pub fn iter_batched_ref<I, O, S, R>(&mut self, mut setup: S, mut routine: R, _size: BatchSize)
    where
        S: FnMut() -> I,
        R: FnMut(&mut I) -> O,
    {
        // Warmup runs
        for _ in 0..codspeed::codspeed::WARMUP_RUNS {
            let mut input = black_box(setup());
            let output = black_box(routine(&mut input));
            drop(black_box(output));
            drop(black_box(input));
        }

        // Multiple measured rounds
        compat_utils::run_rounds(self.codspeed, self.uri.as_str(), || {
            let mut input = setup(); // Setup runs while collection is paused
            InstrumentHooks::toggle_collect(); // Resume collection
            let output = routine(&mut input);
            InstrumentHooks::toggle_collect(); // Pause collection
            black_box(input);
            black_box(output);
        });
    }

    #[cfg(feature = "async")]
    pub fn to_async<'b, A: AsyncExecutor>(&'b mut self, runner: A) -> AsyncBencher<'a, 'b, A> {
        AsyncBencher { b: self, runner }
    }
}

#[cfg(feature = "async")]
pub struct AsyncBencher<'a, 'b, A: AsyncExecutor> {
    b: &'b mut Bencher<'a>,
    runner: A,
}

#[cfg(feature = "async")]
#[allow(clippy::needless_lifetimes)]
impl<'a, 'b, A: AsyncExecutor> AsyncBencher<'a, 'b, A> {
    #[allow(clippy::await_holding_refcell_ref)]
    #[inline(never)]
    pub fn iter<O, R, F>(&mut self, mut routine: R)
    where
        R: FnMut() -> F,
        F: Future<Output = O>,
    {
        use std::time::{Duration, Instant};

        let AsyncBencher { b, runner } = self;
        runner.block_on(async {
            // Warmup runs
            for _ in 0..codspeed::codspeed::WARMUP_RUNS {
                black_box(routine().await);
            }

            // Multiple measured rounds
            let (max_rounds, max_duration) = match std::env::var("CODSPEED_RUNNER_MODE").as_deref()
            {
                Ok("simulation") | Ok("instrumentation") => {
                    (None, Some(Duration::from_millis(100)))
                }
                Ok("memory") => (Some(1), None),
                Ok(m) => unreachable!("Invalid runner mode: {m}"),
                Err(err) => panic!("Failed to get runner mode: {err}"),
            };

            let mut rounds = 0;
            let rounds_start_time = Instant::now();

            // Start benchmark ONCE - this clears CPU caches
            b.codspeed.start_benchmark(b.uri.as_str());
            InstrumentHooks::toggle_collect(); // Pause collection before first iteration

            loop {
                rounds += 1;

                InstrumentHooks::toggle_collect(); // Resume collection
                let output = routine().await;
                InstrumentHooks::toggle_collect(); // Pause collection
                black_box(output);

                let within_rounds = max_rounds.map_or(true, |max| rounds < max);
                let within_duration =
                    max_duration.map_or(true, |max| rounds_start_time.elapsed() < max);

                if !(within_rounds && within_duration) {
                    break;
                }
            }

            // End benchmark ONCE
            b.codspeed.end_benchmark();
        });
    }

    #[inline(never)]
    pub fn iter_custom<R, F, MV>(&mut self, mut _routine: R)
    where
        R: FnMut(u64) -> F,
        F: Future<Output = MV>,
    {
        let AsyncBencher { b, .. } = self;
        println!(
            "{} {} (CodSpeed doesn't support custom iterations)",
            "Skipping:".to_string().yellow(),
            b.uri.yellow(),
        );
    }

    #[doc(hidden)]
    pub fn iter_with_setup<I, O, S, R, F>(&mut self, setup: S, routine: R)
    where
        S: FnMut() -> I,
        R: FnMut(I) -> F,
        F: Future<Output = O>,
    {
        self.iter_batched(setup, routine, BatchSize::PerIteration);
    }

    pub fn iter_with_large_drop<O, R, F>(&mut self, mut routine: R)
    where
        R: FnMut() -> F,
        F: Future<Output = O>,
    {
        self.iter_batched(|| (), |_| routine(), BatchSize::SmallInput);
    }

    #[doc(hidden)]
    pub fn iter_with_large_setup<I, O, S, R, F>(&mut self, setup: S, routine: R)
    where
        S: FnMut() -> I,
        R: FnMut(I) -> F,
        F: Future<Output = O>,
    {
        self.iter_batched(setup, routine, BatchSize::NumBatches(1));
    }

    #[allow(clippy::await_holding_refcell_ref)]
    #[inline(never)]
    pub fn iter_batched<I, O, S, R, F>(&mut self, mut setup: S, mut routine: R, _size: BatchSize)
    where
        S: FnMut() -> I,
        R: FnMut(I) -> F,
        F: Future<Output = O>,
    {
        use std::time::{Duration, Instant};

        let AsyncBencher { b, runner } = self;
        runner.block_on(async {
            // Warmup runs
            for _ in 0..codspeed::codspeed::WARMUP_RUNS {
                let input = black_box(setup());
                let output = routine(input).await;
                drop(black_box(output));
            }

            // Multiple measured rounds
            let (max_rounds, max_duration) = match std::env::var("CODSPEED_RUNNER_MODE").as_deref()
            {
                Ok("simulation") | Ok("instrumentation") => {
                    (None, Some(Duration::from_millis(100)))
                }
                Ok("memory") => (Some(1), None),
                Ok(m) => unreachable!("Invalid runner mode: {m}"),
                Err(err) => panic!("Failed to get runner mode: {err}"),
            };

            let mut rounds = 0;
            let rounds_start_time = Instant::now();

            // Start benchmark ONCE - this clears CPU caches
            b.codspeed.start_benchmark(b.uri.as_str());
            InstrumentHooks::toggle_collect(); // Pause collection before first iteration

            loop {
                rounds += 1;

                let input = setup(); // Setup runs while collection is paused
                InstrumentHooks::toggle_collect(); // Resume collection
                let output = routine(input).await;
                InstrumentHooks::toggle_collect(); // Pause collection
                black_box(output);

                let within_rounds = max_rounds.map_or(true, |max| rounds < max);
                let within_duration =
                    max_duration.map_or(true, |max| rounds_start_time.elapsed() < max);

                if !(within_rounds && within_duration) {
                    break;
                }
            }

            // End benchmark ONCE
            b.codspeed.end_benchmark();
        })
    }

    #[allow(clippy::await_holding_refcell_ref)]
    #[inline(never)]
    pub fn iter_batched_ref<I, O, S, R, F>(
        &mut self,
        mut setup: S,
        mut routine: R,
        _size: BatchSize,
    ) where
        S: FnMut() -> I,
        R: FnMut(&mut I) -> F,
        F: Future<Output = O>,
    {
        use std::time::{Duration, Instant};

        let AsyncBencher { b, runner } = self;
        runner.block_on(async {
            // Warmup runs
            for _ in 0..codspeed::codspeed::WARMUP_RUNS {
                let mut input = black_box(setup());
                let output = black_box(routine(&mut input).await);
                drop(black_box(output));
                drop(black_box(input));
            }

            // Multiple measured rounds
            let (max_rounds, max_duration) = match std::env::var("CODSPEED_RUNNER_MODE").as_deref()
            {
                Ok("simulation") | Ok("instrumentation") => {
                    (None, Some(Duration::from_millis(100)))
                }
                Ok("memory") => (Some(1), None),
                Ok(m) => unreachable!("Invalid runner mode: {m}"),
                Err(err) => panic!("Failed to get runner mode: {err}"),
            };

            let mut rounds = 0;
            let rounds_start_time = Instant::now();

            // Start benchmark ONCE - this clears CPU caches
            b.codspeed.start_benchmark(b.uri.as_str());
            InstrumentHooks::toggle_collect(); // Pause collection before first iteration

            loop {
                rounds += 1;

                let mut input = setup(); // Setup runs while collection is paused
                InstrumentHooks::toggle_collect(); // Resume collection
                let output = routine(&mut input).await;
                InstrumentHooks::toggle_collect(); // Pause collection
                black_box(input);
                black_box(output);

                let within_rounds = max_rounds.map_or(true, |max| rounds < max);
                let within_duration =
                    max_duration.map_or(true, |max| rounds_start_time.elapsed() < max);

                if !(within_rounds && within_duration) {
                    break;
                }
            }

            // End benchmark ONCE
            b.codspeed.end_benchmark();
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    #[test]
    fn test_auto_traits() {
        assert_send::<Bencher>();
        assert_sync::<Bencher>();
    }
}
