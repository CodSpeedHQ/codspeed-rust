//! Handpicked stubs from [divan::entry](https://github.com/nvzqz/divan/blob/main/src/entry/mod.rs)
//! Minimally reimplemented in an API compatible way to run the benches using codspeed intrumentation
use codspeed::codspeed::CodSpeed;
use std::{cell::RefCell, rc::Rc};

/// Benchmarking options set directly by the user in `#[divan::bench]` and
/// `#[divan::bench_group]`.
///
/// Changes to fields must be reflected in the "Options" sections of the docs
/// for `#[divan::bench]` and `#[divan::bench_group]`.
#[derive(Default)]
pub struct BenchOptions<'a> {
    pub(crate) _marker: std::marker::PhantomData<&'a ()>,
}

/// Using this in place of `()` for `GenI` prevents `Bencher::with_inputs` from
/// working with `()` unintentionally.
#[non_exhaustive]
pub struct Unit;

pub struct BencherConfig<GenI = Unit> {
    gen_input: GenI,
}

pub struct Bencher<'a, 'b, C = BencherConfig> {
    pub(crate) codspeed: Rc<RefCell<CodSpeed>>,
    pub(crate) uri: String,
    pub(crate) config: C,
    pub(crate) _marker: std::marker::PhantomData<&'a &'b ()>,
}

#[allow(clippy::needless_lifetimes)]
impl<'a, 'b> Bencher<'a, 'b> {
    pub(crate) fn new(uri: String) -> Self {
        Self {
            codspeed: Rc::new(RefCell::new(CodSpeed::new())),
            config: BencherConfig { gen_input: Unit },
            uri,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn with_inputs<G>(self, gen_input: G) -> Bencher<'a, 'b, BencherConfig<G>> {
        Bencher {
            config: BencherConfig { gen_input },
            codspeed: self.codspeed,
            uri: self.uri,
            _marker: self._marker,
        }
    }
}

#[allow(clippy::needless_lifetimes)]
impl<'a, 'b, I, GenI> Bencher<'a, 'b, BencherConfig<GenI>>
where
    GenI: FnMut() -> I,
{
    pub fn bench<O, B>(&self, benched: B)
    where
        B: Fn() -> O + Sync,
    {
        let mut codspeed = self.codspeed.borrow_mut();
        codspeed.start_benchmark(self.uri.as_str());
        divan::black_box(benched());
        codspeed.end_benchmark();
    }

    pub fn bench_values<O, B>(self, benched: B)
    where
        B: Fn(I) -> O + Sync,
        GenI: Fn() -> I + Sync,
    {
        let mut codspeed = self.codspeed.borrow_mut();
        let gen_input = self.config.gen_input;
        let input = gen_input();

        codspeed.start_benchmark(self.uri.as_str());
        divan::black_box(benched(input));
        codspeed.end_benchmark();
    }
}
