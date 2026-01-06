use crate::measurement;
use colored::Colorize;
use std::ffi::CString;

pub use std::hint::black_box;

pub const WARMUP_RUNS: u32 = 5;

pub fn display_native_harness() {
    eprintln!("Harness: codspeed v{}", env!("CARGO_PKG_VERSION"),);
}

#[derive(PartialEq)]
pub enum InstrumentationStatus {
    /// Instrumentation detected via inline assembly directly
    Valgrind,
    /// Instrumentation detected via InstrumentHooks
    InstrumentHooks(&'static crate::instrument_hooks::InstrumentHooks),
    NotInstrumented,
}

impl InstrumentationStatus {
    pub fn is_instrumented(&self) -> bool {
        *self != InstrumentationStatus::NotInstrumented
    }
}

pub struct CodSpeed {
    benchmarked: Vec<String>,
    current_benchmark: CString,
    group_stack: Vec<String>,
    instrumentation_status: InstrumentationStatus,
}

impl CodSpeed {
    pub fn new() -> Self {
        use crate::instrument_hooks::InstrumentHooks;
        let instrumentation_status = {
            // We completely bypass InstrumentHooks if we detect Valgrind via inline assembly
            // Until we can reliably get rid of the inline assembly without causing breaking
            // changes in CPU simulation measurements by switching to InstrumentHooks only, we need
            // to keep this separation.
            if measurement::is_instrumented() {
                InstrumentationStatus::Valgrind
            } else {
                let hooks_instance = InstrumentHooks::instance();
                if hooks_instance.is_instrumented() {
                    InstrumentationStatus::InstrumentHooks(hooks_instance)
                } else {
                    InstrumentationStatus::NotInstrumented
                }
            }
        };

        if !instrumentation_status.is_instrumented() {
            eprintln!(
                "{} codspeed is enabled, but no performance measurement will be made since it's running in an unknown environment.",
                "NOTICE:".to_string().bold()
            );
        };

        measurement::set_metadata();

        Self {
            benchmarked: Vec::new(),
            current_benchmark: CString::new("").expect("CString::new failed"),
            group_stack: Vec::new(),
            instrumentation_status,
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

        if let InstrumentationStatus::InstrumentHooks(hooks_instance) = &self.instrumentation_status
        {
            let _ = hooks_instance.start_benchmark();
        }

        // We intentionally do this no matter the instrumentation status.
        // The overhead in case we are not running valgrind is extremely low, and we do not want to
        // add a conditionnal branch to valgrind measurements.
        measurement::start();
    }

    #[inline(always)]
    pub fn end_benchmark(&mut self) {
        // We intentionally do this no matter the instrumentation status.
        // The overhead in case we are not running valgrind is extremely low, and we do not want to
        // add a conditionnal branch to valgrind measurements.
        measurement::stop(&self.current_benchmark);
        if let InstrumentationStatus::InstrumentHooks(hooks_instance) = &self.instrumentation_status
        {
            let _ = hooks_instance.stop_benchmark();
            let _ =
                hooks_instance.set_executed_benchmark(&self.current_benchmark.to_string_lossy());
        }
        self.benchmarked
            .push(self.current_benchmark.to_str().unwrap().to_string());

        let action_str = if self.instrumentation_status.is_instrumented() {
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
