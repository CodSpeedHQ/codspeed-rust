use crate::{measurement, utils};
use colored::Colorize;
use std::ffi::CString;
use std::time::Instant;

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
    start_time: Option<Instant>,
    show_details: bool,
}

impl CodSpeed {
    pub fn new() -> Self {
        let is_instrumented = measurement::is_instrumented();
        let show_details = utils::show_details();
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
            start_time: None,
            show_details,
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
        measurement::start();
        if self.show_details && !self.is_instrumented {
            self.start_time = Some(Instant::now());
        }
    }

    #[inline(always)]
    pub fn end_benchmark(&mut self) {
        measurement::stop(&self.current_benchmark);
        self.benchmarked
            .push(self.current_benchmark.to_str().unwrap().to_string());

        // Early return for instrumented mode with details - no output needed
        if self.show_details && self.is_instrumented {
            return;
        }

        // For --details mode in non-instrumented environment, show timing
        if self.show_details {
            let elapsed = self
                .start_time
                .take()
                .map(|start| start.elapsed())
                .unwrap_or_default();
            if self.group_stack.is_empty() {
                println!(
                    "  Checked: {} ({})",
                    self.current_benchmark.to_string_lossy(),
                    crate::utils::format_duration_nanos(elapsed.as_nanos())
                );
            } else {
                println!(
                    "  Checked: {} (group: {}) ({})",
                    self.current_benchmark.to_string_lossy(),
                    self.group_stack.join("/"),
                    crate::utils::format_duration_nanos(elapsed.as_nanos())
                );
            }
            return;
        }

        // Default output for non-details mode
        let action_str = if self.is_instrumented {
            "Measured"
        } else {
            "Checked"
        };
        if self.group_stack.is_empty() {
            println!(
                "{}: {}",
                action_str,
                self.current_benchmark.to_string_lossy()
            );
        } else {
            println!(
                "{}: {} (group: {})",
                action_str,
                self.current_benchmark.to_string_lossy(),
                self.group_stack.join("/")
            );
        }
    }
}

impl Default for CodSpeed {
    fn default() -> Self {
        Self::new()
    }
}
