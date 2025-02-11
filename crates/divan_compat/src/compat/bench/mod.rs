//! Handpicked stubs from [divan::bench](https://github.com/nvzqz/divan/blob/main/src/bench/mod.rs)
//! Minimally reimplemented in an API compatible way to run the benches using codspeed intrumentation
#![allow(clippy::needless_lifetimes)] // We keep explicit lifetime to be as close to original API as possible

mod args;
mod options;

pub use self::{
    args::{BenchArgs, BenchArgsRunner},
    options::BenchOptions,
};

use codspeed::codspeed::CodSpeed;
use std::cell::RefCell;

/// Using this in place of `()` for `GenI` prevents `Bencher::with_inputs` from
/// working with `()` unintentionally.
#[non_exhaustive]
pub struct Unit;

pub struct BencherConfig<GenI = Unit> {
    gen_input: RefCell<GenI>,
}

pub struct Bencher<'a, 'b, C = BencherConfig> {
    pub(crate) codspeed: &'a RefCell<CodSpeed>,
    pub(crate) uri: String,
    pub(crate) config: C,
    pub(crate) _marker: std::marker::PhantomData<&'b ()>,
}

impl<'a, 'b> Bencher<'a, 'b> {
    pub(crate) fn new(codspeed: &'a RefCell<CodSpeed>, uri: String) -> Self {
        Self {
            config: BencherConfig {
                gen_input: RefCell::new(Unit),
            },
            codspeed,
            uri,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn with_inputs<G>(self, gen_input: G) -> Bencher<'a, 'b, BencherConfig<G>> {
        Bencher {
            config: BencherConfig {
                gen_input: RefCell::new(gen_input),
            },
            codspeed: self.codspeed,
            uri: self.uri,
            _marker: self._marker,
        }
    }
}

impl<'a, 'b> Bencher<'a, 'b> {
    pub fn bench<O, B>(self, benched: B)
    where
        B: Fn() -> O + Sync,
    {
        self.with_inputs(|| ()).bench_values(|_| benched())
    }

    pub fn bench_local<O, B>(self, mut benched: B)
    where
        B: FnMut() -> O,
    {
        self.with_inputs(|| ()).bench_local_values(|_| benched())
    }
}

impl<'a, 'b, I, GenI> Bencher<'a, 'b, BencherConfig<GenI>>
where
    GenI: FnMut() -> I,
{
    pub fn bench_values<O, B>(self, benched: B)
    where
        B: Fn(I) -> O + Sync,
        GenI: Fn() -> I + Sync,
    {
        self.bench_local_values(benched)
    }

    pub fn bench_refs<O, B>(self, benched: B)
    where
        B: Fn(&mut I) -> O + Sync,
        GenI: Fn() -> I + Sync,
    {
        self.bench_local_refs(benched)
    }

    pub fn bench_local_values<O, B>(self, mut benched: B)
    where
        B: FnMut(I) -> O,
    {
        let mut codspeed = self.codspeed.borrow_mut();
        let mut gen_input = self.config.gen_input.borrow_mut();
        let input = gen_input();
        codspeed.start_benchmark(self.uri.as_str());
        divan::black_box(benched(input));
        codspeed.end_benchmark();
    }

    pub fn bench_local_refs<O, B>(self, mut benched: B)
    where
        B: FnMut(&mut I) -> O,
    {
        let mut codspeed = self.codspeed.borrow_mut();
        let mut gen_input = self.config.gen_input.borrow_mut();
        let mut input = gen_input();

        codspeed.start_benchmark(self.uri.as_str());
        divan::black_box(benched(&mut input));
        codspeed.end_benchmark();
    }
}
