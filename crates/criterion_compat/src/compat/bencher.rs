use std::{cell::RefCell, rc::Rc};

use codspeed::codspeed::{black_box, CodSpeed};
use colored::Colorize;
use criterion::BatchSize;

#[cfg(feature = "async")]
use criterion::async_executor::AsyncExecutor;
#[cfg(feature = "async")]
use std::future::Future;

pub struct Bencher {
    codspeed: Rc<RefCell<CodSpeed>>,
    uri: String,
}

impl Bencher {
    pub fn new(codspeed: Rc<RefCell<CodSpeed>>, uri: String) -> Self {
        Bencher { codspeed, uri }
    }

    #[inline(never)]
    pub fn iter<O, R>(&mut self, mut routine: R)
    where
        R: FnMut() -> O,
    {
        let mut codspeed = self.codspeed.borrow_mut();
        codspeed.start_benchmark(self.uri.as_str());
        black_box(routine());
        codspeed.end_benchmark();
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
        let mut codspeed = self.codspeed.borrow_mut();
        let input = black_box(setup());
        codspeed.start_benchmark(self.uri.as_str());
        let output = routine(input);
        codspeed.end_benchmark();

        drop(black_box(output));
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
        let mut codspeed = self.codspeed.borrow_mut();
        let mut input = black_box(setup());

        codspeed.start_benchmark(self.uri.as_str());
        let output = routine(&mut input);
        codspeed.end_benchmark();

        drop(black_box(output));
        drop(black_box(input));
    }

    #[cfg(feature = "async")]
    pub fn to_async<A: AsyncExecutor>(&mut self, runner: A) -> AsyncBencher<A> {
        AsyncBencher { b: self, runner }
    }
}

#[cfg(feature = "async")]
pub struct AsyncBencher<'b, A: AsyncExecutor> {
    b: &'b mut Bencher,
    runner: A,
}

#[cfg(feature = "async")]
impl<'b, A: AsyncExecutor> AsyncBencher<'b, A> {
    #[allow(clippy::await_holding_refcell_ref)]
    #[inline(never)]
    pub fn iter<O, R, F>(&mut self, mut routine: R)
    where
        R: FnMut() -> F,
        F: Future<Output = O>,
    {
        let AsyncBencher { b, runner } = self;
        runner.block_on(async {
            let mut codspeed = b.codspeed.borrow_mut();
            codspeed.start_benchmark(b.uri.as_str());
            black_box(routine().await);
            codspeed.end_benchmark();
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
            let mut codspeed = b.codspeed.borrow_mut();
            let input = black_box(setup());
            codspeed.start_benchmark(b.uri.as_str());
            let output = routine(input).await;
            codspeed.end_benchmark();
            drop(black_box(output));
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
            let mut codspeed = b.codspeed.borrow_mut();
            let mut input = black_box(setup());

            codspeed.start_benchmark(b.uri.as_str());
            let output = routine(&mut input).await;
            codspeed.end_benchmark();

            drop(black_box(output));
            drop(black_box(input));
        });
    }
}
