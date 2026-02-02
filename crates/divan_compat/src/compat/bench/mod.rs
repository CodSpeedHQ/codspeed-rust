//! Handpicked stubs from [divan::bench](https://github.com/nvzqz/divan/blob/main/src/bench/mod.rs)
//! Minimally reimplemented in an API compatible way to run the benches using codspeed intrumentation
#![allow(clippy::needless_lifetimes)] // We keep explicit lifetime to be as close to original API as possible

mod args;
mod options;

pub use self::{
    args::{BenchArgs, BenchArgsRunner},
    options::BenchOptions,
};

use ::codspeed::codspeed::CodSpeed;
use ::codspeed::instrument_hooks::InstrumentHooks;
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

    /// Add a counter to this benchmark (placeholder implementation).
    ///
    /// Note: Counters are not yet supported by codspeed-divan-compat.
    /// This method is provided for API compatibility but does not affect benchmarking.
    pub fn counter<C>(self, _counter: C) -> Self {
        eprintln!("Warning: Counter feature is not yet supported by codspeed-divan-compat");
        self
    }

    /// Add a counter based on input to this benchmark (placeholder implementation).
    ///
    /// Note: Counters are not yet supported by codspeed-divan-compat.
    /// This method is provided for API compatibility but does not affect benchmarking.
    pub fn input_counter<C, F>(self, _counter_fn: F) -> Self
    where
        F: Fn() -> C,
    {
        eprintln!("Warning: Counter feature is not yet supported by codspeed-divan-compat");
        self
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
    /// Add a counter to this benchmark (placeholder implementation).
    ///
    /// Note: Counters are not yet supported by codspeed-divan-compat.
    /// This method is provided for API compatibility but does not affect benchmarking.
    pub fn counter<C>(self, _counter: C) -> Self {
        eprintln!("Warning: Counter feature is not yet supported by codspeed-divan-compat");
        self
    }

    /// Add a counter based on input to this benchmark (placeholder implementation).
    ///
    /// Note: Counters are not yet supported by codspeed-divan-compat.
    /// This method is provided for API compatibility but does not affect benchmarking.
    pub fn input_counter<C, F>(self, _counter_fn: F) -> Self
    where
        F: Fn(&I) -> C,
    {
        eprintln!("Warning: Counter feature is not yet supported by codspeed-divan-compat");
        self
    }
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

        codspeed::run_rounds(&mut codspeed, self.uri.as_str(), || {
            // FIXME: We could also run multiple rounds here
            let input = gen_input();
            InstrumentHooks::toggle_collect();
            let output = benched(divan::black_box(input));
            InstrumentHooks::toggle_collect();
            divan::black_box(output);
        });
    }

    pub fn bench_local_refs<O, B>(self, mut benched: B)
    where
        B: FnMut(&mut I) -> O,
    {
        let mut codspeed = self.codspeed.borrow_mut();
        let mut gen_input = self.config.gen_input.borrow_mut();

        codspeed::run_rounds(&mut codspeed, self.uri.as_str(), || {
            let mut input = gen_input();
            InstrumentHooks::toggle_collect();
            let output = benched(&mut input);
            InstrumentHooks::toggle_collect();
            divan::black_box(input);
            divan::black_box(output);
        });
    }
}

mod codspeed {
    use super::*;
    use std::time::{Duration, Instant};

    pub fn run_rounds(codspeed: &mut CodSpeed, uri: &str, mut run_iteration: impl FnMut()) {
        // FIXME: Maybe move this to codspeed
        let (max_rounds, max_duration) = match std::env::var("CODSPEED_RUNNER_MODE").as_deref() {
            Ok("simulation") | Ok("instrumentation") => (None, Some(Duration::from_millis(100))),
            Ok("memory") => (Some(1), None),
            Ok(m) => unreachable!("Invalid runner mode: {m}"),
            Err(err) => panic!("Failed to get runner mode: {err}"),
        };
        let mut rounds = 0;
        let rounds_start_time = Instant::now();

        codspeed.start_benchmark(uri);
        InstrumentHooks::toggle_collect(); // Pause collection

        loop {
            rounds += 1;

            run_iteration();

            let within_rounds = max_rounds.map_or(true, |max| rounds < max);
            let within_duration =
                max_duration.map_or(true, |max| rounds_start_time.elapsed() < max);
            if !(within_rounds && within_duration) {
                break;
            }
        }

        codspeed.end_benchmark();
    }
}
