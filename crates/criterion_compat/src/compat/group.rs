use std::marker::PhantomData;
use std::{cell::RefCell, rc::Rc, time::Duration};

use codspeed::{codspeed::CodSpeed, utils::get_git_relative_path};
use criterion::measurement::WallTime;
use criterion::{measurement::Measurement, PlotConfiguration, SamplingMode, Throughput};

use crate::{Bencher, Criterion};

/// Deprecated: using the default measurement will be removed in the next major version.
/// Defaulting to WallTime differs from the original BenchmarkGroup implementation but avoids creating a breaking change
pub struct BenchmarkGroup<'a, M: Measurement = WallTime> {
    codspeed: Rc<RefCell<CodSpeed>>,
    current_file: String,
    macro_group: String,
    group_name: String,
    _marker: PhantomData<&'a M>,
}

#[allow(clippy::needless_lifetimes)]
impl<'a, M: Measurement> BenchmarkGroup<'a, M> {
    pub fn new(criterion: &mut Criterion<M>, group_name: String) -> BenchmarkGroup<M> {
        BenchmarkGroup::<M> {
            codspeed: criterion
                .codspeed
                .as_ref()
                .expect("non instrumented codspeed interface")
                .clone(),
            current_file: criterion.current_file.clone(),
            macro_group: criterion.macro_group.clone(),
            group_name,
            _marker: PhantomData,
        }
    }

    pub fn bench_function<ID: IntoBenchmarkId, F>(&mut self, id: ID, mut f: F) -> &mut Self
    where
        F: FnMut(&mut Bencher),
    {
        self.run_bench(id.into_benchmark_id(), &(), |b, _| f(b));
        self
    }

    pub fn bench_with_input<ID: IntoBenchmarkId, F, I>(
        &mut self,
        id: ID,
        input: &I,
        f: F,
    ) -> &mut Self
    where
        F: FnMut(&mut Bencher, &I),
        I: ?Sized,
    {
        self.run_bench(id.into_benchmark_id(), input, f);
        self
    }

    fn run_bench<F, I>(&mut self, id: BenchmarkId, input: &I, mut f: F)
    where
        F: FnMut(&mut Bencher, &I),
        I: ?Sized,
    {
        let git_relative_file_path = get_git_relative_path(&self.current_file);
        let mut uri = format!(
            "{}::{}::{}",
            git_relative_file_path.to_string_lossy(),
            self.macro_group,
            self.group_name,
        );
        if let Some(function_name) = id.function_name {
            uri = format!("{}::{}", uri, function_name);
        }
        if let Some(parameter) = id.parameter {
            uri = format!("{}[{}]", uri, parameter);
        }
        let mut b = Bencher::new(self.codspeed.clone(), uri);
        f(&mut b, input);
    }
}

// Dummy methods
#[allow(unused_variables, clippy::needless_lifetimes)]
impl<'a, M: Measurement> BenchmarkGroup<'a, M> {
    pub fn sample_size(&mut self, n: usize) -> &mut Self {
        self
    }
    pub fn warm_up_time(&mut self, dur: Duration) -> &mut Self {
        self
    }
    pub fn measurement_time(&mut self, dur: Duration) -> &mut Self {
        self
    }
    pub fn nresamples(&mut self, n: usize) -> &mut Self {
        self
    }
    pub fn noise_threshold(&mut self, threshold: f64) -> &mut Self {
        self
    }
    pub fn confidence_level(&mut self, cl: f64) -> &mut Self {
        self
    }
    pub fn significance_level(&mut self, sl: f64) -> &mut Self {
        self
    }
    pub fn throughput(&mut self, throughput: Throughput) -> &mut Self {
        self
    }
    pub fn sampling_mode(&mut self, new_mode: SamplingMode) -> &mut Self {
        self
    }
    pub fn plot_config(&mut self, new_config: PlotConfiguration) -> &mut Self {
        self
    }
    pub fn finish(self) {}
}

// BenchmarkId is a copy of the BenchmarkId struct from criterion.rs allowing private fields to
// be used in this crate.
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct BenchmarkId {
    pub(crate) function_name: Option<String>,
    pub(crate) parameter: Option<String>,
}

impl BenchmarkId {
    pub fn new<S: Into<String>, P: ::std::fmt::Display>(
        function_name: S,
        parameter: P,
    ) -> BenchmarkId {
        BenchmarkId {
            function_name: Some(function_name.into()),
            parameter: Some(format!("{}", parameter)),
        }
    }

    /// Construct a new benchmark ID from just a parameter value. Use this when benchmarking a
    /// single function with a variety of different inputs.
    pub fn from_parameter<P: ::std::fmt::Display>(parameter: P) -> BenchmarkId {
        BenchmarkId {
            function_name: None,
            parameter: Some(format!("{}", parameter)),
        }
    }

    pub(crate) fn no_function() -> BenchmarkId {
        BenchmarkId {
            function_name: None,
            parameter: None,
        }
    }

    pub(crate) fn no_function_with_input<P: ::std::fmt::Display>(parameter: P) -> BenchmarkId {
        BenchmarkId {
            function_name: None,
            parameter: Some(format!("{}", parameter)),
        }
    }
}

mod private {
    pub trait Sealed {}
    impl Sealed for super::BenchmarkId {}
    impl<S: Into<String>> Sealed for S {}
}

/// Sealed trait which allows users to automatically convert strings to benchmark IDs.
pub trait IntoBenchmarkId: private::Sealed {
    fn into_benchmark_id(self) -> BenchmarkId;
}
impl IntoBenchmarkId for BenchmarkId {
    fn into_benchmark_id(self) -> BenchmarkId {
        self
    }
}
impl<S: Into<String>> IntoBenchmarkId for S {
    fn into_benchmark_id(self) -> BenchmarkId {
        let function_name = self.into();
        assert!(
            !function_name.is_empty(),
            "Function name must not be empty."
        );

        BenchmarkId {
            function_name: Some(function_name),
            parameter: None,
        }
    }
}
