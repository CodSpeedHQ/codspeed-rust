use crate::{instrument_hooks::InstrumentHooks, measurement};
use colored::Colorize;
use std::ffi::CString;

pub use std::hint::black_box;

pub const WARMUP_RUNS: u32 = 5;

pub fn display_native_harness() {
    println!("Harness: codspeed v{}", env!("CARGO_PKG_VERSION"),);
}

pub struct CodSpeed {
    benchmarked: Vec<String>,
    current_benchmark: CString,
    group_stack: Vec<String>,
    is_instrumented: bool,
}

impl CodSpeed {
    pub fn new() -> Self {
        let hooks_instance = InstrumentHooks::instance();
        let is_instrumented = hooks_instance.is_instrumented();
        if !is_instrumented {
            println!(
                "{} codspeed is enabled, but no performance measurement will be made since it's running in an unknown environment.",
                "NOTICE:".to_string().bold()
            );
        }
        measurement::set_metadata();
        Self {
            benchmarked: Vec::new(),
            current_benchmark: CString::new("").expect("CString::new failed"),
            group_stack: Vec::new(),
            is_instrumented,
        }
    }

    pub fn push_group(&mut self, group: &str) {
        self.group_stack.push(group.to_string());
    }

    pub fn pop_group(&mut self) {
        self.group_stack.pop();
    }

    #[inline(always)]
    pub fn start_benchmark(&mut self, name: &str) {
        self.current_benchmark = CString::new(name).expect("CString::new failed");
        let _ = InstrumentHooks::instance().start_benchmark();
        measurement::start();
    }

    #[inline(always)]
    pub fn end_benchmark(&mut self) {
        measurement::stop(&self.current_benchmark);
        let _ = InstrumentHooks::instance().stop_benchmark();
        let _ = InstrumentHooks::instance()
            .set_executed_benchmark(&self.current_benchmark.to_string_lossy());
        self.benchmarked
            .push(self.current_benchmark.to_str().unwrap().to_string());
        let action_str = if self.is_instrumented {
            "Measured"
        } else {
            "Checked"
        };
        let group_str = if self.group_stack.is_empty() {
            "".to_string()
        } else {
            format!(" (group: {})", self.group_stack.join("/"))
        };
        println!(
            "{}: {}{}",
            action_str,
            self.current_benchmark.to_string_lossy(),
            group_str
        );
    }
}

impl Default for CodSpeed {
    fn default() -> Self {
        Self::new()
    }
}
