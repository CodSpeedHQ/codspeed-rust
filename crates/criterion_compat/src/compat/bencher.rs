use std::{cell::RefCell, rc::Rc};

use codspeed::codspeed::{black_box, CodSpeed};
use colored::Colorize;
use criterion::BatchSize;

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
    pub fn iter_custom<R, M>(&mut self, mut _routine: R)
    where
        R: FnMut(u64) -> M,
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
}
