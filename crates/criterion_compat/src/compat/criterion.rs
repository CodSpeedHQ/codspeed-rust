use std::{cell::RefCell, marker::PhantomData, rc::Rc, time::Duration};

use codspeed::codspeed::CodSpeed;
use criterion::{
    measurement::{Measurement, WallTime},
    profiler::Profiler,
    PlottingBackend,
};

use crate::{Bencher, BenchmarkGroup, BenchmarkId};

pub struct Criterion<M: Measurement = WallTime> {
    pub codspeed: Option<Rc<RefCell<CodSpeed>>>,
    pub current_file: String,
    pub macro_group: String,
    phantom: PhantomData<*const M>,
}

#[doc(hidden)]
impl Criterion {
    pub fn new_instrumented() -> Self {
        println!(
            "Harness: codspeed-criterion-compat v{}",
            env!("CARGO_PKG_VERSION"),
        );
        Criterion {
            codspeed: Some(Rc::new(RefCell::new(CodSpeed::new()))),
            current_file: String::new(),
            macro_group: String::new(),
            phantom: PhantomData,
        }
    }

    pub fn with_patched_measurement<M: Measurement>(&mut self, _: Criterion<M>) -> Criterion<M> {
        Criterion {
            codspeed: self.codspeed.clone(),
            current_file: self.current_file.clone(),
            macro_group: self.macro_group.clone(),
            phantom: PhantomData,
        }
    }
}

impl<M: Measurement> Criterion<M> {
    #[doc(hidden)]
    pub fn set_current_file(&mut self, file: impl Into<String>) {
        self.current_file = file.into();
    }

    #[doc(hidden)]
    pub fn set_macro_group(&mut self, macro_group: impl Into<String>) {
        self.macro_group = macro_group.into();
    }

    pub fn bench_function<F>(&mut self, id: &str, f: F) -> &mut Criterion<M>
    where
        F: FnMut(&mut Bencher),
    {
        self.benchmark_group(id)
            .bench_function(BenchmarkId::no_function(), f);
        self
    }

    pub fn bench_with_input<F, I>(&mut self, id: BenchmarkId, input: &I, f: F) -> &mut Criterion<M>
    where
        F: FnMut(&mut Bencher, &I),
    {
        let group_name = id.function_name.expect(
            "Cannot use BenchmarkId::from_parameter with Criterion::bench_with_input. \
                 Consider using a BenchmarkGroup or BenchmarkId::new instead.",
        );
        let parameter = id.parameter.unwrap();
        self.benchmark_group(group_name).bench_with_input(
            BenchmarkId::no_function_with_input(parameter),
            input,
            f,
        );
        self
    }

    pub fn benchmark_group<S: Into<String>>(&mut self, group_name: S) -> BenchmarkGroup<M> {
        BenchmarkGroup::<M>::new(self, group_name.into())
    }
}

// Dummy methods
#[allow(clippy::derivable_impls)]
impl Default for Criterion {
    // Dummy method creating an empty Criterion helper useful to mock the configuration
    fn default() -> Self {
        Criterion {
            codspeed: None,
            current_file: String::new(),
            macro_group: String::new(),
            phantom: PhantomData,
        }
    }
}

#[allow(dead_code, unused_variables, unused_mut)]
impl<M: Measurement> Criterion<M> {
    pub fn with_measurement<M2: Measurement>(self, m: M2) -> Criterion<M2> {
        Criterion {
            codspeed: self.codspeed,
            current_file: self.current_file,
            macro_group: self.macro_group,
            phantom: PhantomData::<*const M2>,
        }
    }
    pub fn with_profiler<P: Profiler + 'static>(self, p: P) -> Criterion<M> {
        self
    }
    pub fn plotting_backend(mut self, backend: PlottingBackend) -> Criterion<M> {
        self
    }
    pub fn sample_size(mut self, n: usize) -> Criterion<M> {
        self
    }
    pub fn warm_up_time(mut self, dur: Duration) -> Criterion<M> {
        self
    }
    pub fn measurement_time(mut self, dur: Duration) -> Criterion<M> {
        self
    }
    pub fn nresamples(mut self, n: usize) -> Criterion<M> {
        self
    }
    pub fn noise_threshold(mut self, threshold: f64) -> Criterion<M> {
        self
    }
    pub fn confidence_level(mut self, cl: f64) -> Criterion<M> {
        self
    }
    pub fn significance_level(mut self, sl: f64) -> Criterion<M> {
        self
    }
    pub fn with_plots(mut self) -> Criterion<M> {
        self
    }
    pub fn without_plots(mut self) -> Criterion<M> {
        self
    }
    pub fn can_plot(&self) -> bool {
        true
    }
    pub fn save_baseline(mut self, baseline: String) -> Criterion<M> {
        self
    }
    pub fn retain_baseline(mut self, baseline: String) -> Criterion<M> {
        self
    }
    pub fn with_filter<S: Into<String>>(mut self, filter: S) -> Criterion<M> {
        //FIXME: Implement
        self
    }
    pub fn with_output_color(mut self, enabled: bool) -> Criterion<M> {
        self
    }
    pub fn configure_from_args(mut self) -> Criterion<M> {
        self
    }
}
