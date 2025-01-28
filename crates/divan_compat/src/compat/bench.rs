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

pub struct Bencher<'a, 'b> {
    pub(crate) codspeed: Rc<RefCell<CodSpeed>>,
    pub(crate) uri: String,
    pub(crate) _marker: std::marker::PhantomData<&'a &'b ()>,
}

#[allow(clippy::needless_lifetimes)]
impl<'a, 'b> Bencher<'a, 'b> {
    pub(crate) fn new(uri: String) -> Self {
        Self {
            codspeed: Rc::new(RefCell::new(CodSpeed::new())),
            uri,
            _marker: std::marker::PhantomData,
        }
    }
}

#[allow(clippy::needless_lifetimes)]
impl<'a, 'b> Bencher<'a, 'b> {
    pub fn bench<O, B>(&self, benched: B)
    where
        B: Fn() -> O + Sync,
    {
        let mut codspeed = self.codspeed.borrow_mut();
        codspeed.start_benchmark(self.uri.as_str());
        divan::black_box(benched());
        codspeed.end_benchmark();
    }
}
