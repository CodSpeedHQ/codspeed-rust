use codspeed::codspeed::{black_box, CodSpeed};
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
        for i in 0..codspeed::codspeed::WARMUP_RUNS + 1 {
            if i < codspeed::codspeed::WARMUP_RUNS {
                black_box(routine());
            } else {
                self.codspeed.start_benchmark(self.uri.as_str());
                black_box(routine());
                self.codspeed.end_benchmark();
            }
        }
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
        for i in 0..codspeed::codspeed::WARMUP_RUNS + 1 {
            let input = black_box(setup());
            let output = if i < codspeed::codspeed::WARMUP_RUNS {
                routine(input)
            } else {
                let input = black_box(setup());
                self.codspeed.start_benchmark(self.uri.as_str());
                let output = routine(input);
                self.codspeed.end_benchmark();
                output
            };
            drop(black_box(output));
        }
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
        for i in 0..codspeed::codspeed::WARMUP_RUNS + 1 {
            let mut input = black_box(setup());
            let output = if i < codspeed::codspeed::WARMUP_RUNS {
                black_box(routine(&mut input))
            } else {
                self.codspeed.start_benchmark(self.uri.as_str());
                let output = black_box(routine(&mut input));
                self.codspeed.end_benchmark();
                output
            };
            drop(black_box(output));
            drop(black_box(input));
        }
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
        let AsyncBencher { b, runner } = self;
        runner.block_on(async {
            for i in 0..codspeed::codspeed::WARMUP_RUNS + 1 {
                if i < codspeed::codspeed::WARMUP_RUNS {
                    black_box(routine().await);
                } else {
                    b.codspeed.start_benchmark(b.uri.as_str());
                    black_box(routine().await);
                    b.codspeed.end_benchmark();
                }
            }
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
        let AsyncBencher { b, runner } = self;
        runner.block_on(async {
            for i in 0..codspeed::codspeed::WARMUP_RUNS + 1 {
                let input = black_box(setup());
                let output = if i < codspeed::codspeed::WARMUP_RUNS {
                    routine(input).await
                } else {
                    b.codspeed.start_benchmark(b.uri.as_str());
                    let output = routine(input).await;
                    b.codspeed.end_benchmark();
                    output
                };
                drop(black_box(output));
            }
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
        let AsyncBencher { b, runner } = self;
        runner.block_on(async {
            for i in 0..codspeed::codspeed::WARMUP_RUNS + 1 {
                let mut input = black_box(setup());
                let output = if i < codspeed::codspeed::WARMUP_RUNS {
                    black_box(routine(&mut input).await)
                } else {
                    b.codspeed.start_benchmark(b.uri.as_str());
                    let output = black_box(routine(&mut input).await);
                    b.codspeed.end_benchmark();
                    output
                };
                drop(black_box(output));
                drop(black_box(input));
            }
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
