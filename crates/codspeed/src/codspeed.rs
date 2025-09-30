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
        } else {
            self.start_time = None;
        }
    }

    #[inline(always)]
    pub fn end_benchmark(&mut self) {
        measurement::stop(&self.current_benchmark);
        self.benchmarked
            .push(self.current_benchmark.to_str().unwrap().to_string());
        if self.show_details {
            if !self.is_instrumented {
                let elapsed = self
                    .start_time
                    .take()
                    .map(|start| start.elapsed())
                    .unwrap_or_default();
                let group_str = if self.group_stack.is_empty() {
                    "".to_string()
                } else {
                    format!(" (group: {})", self.group_stack.join("/"))
                };
                let formatted_duration = crate::utils::format_duration_nanos(elapsed.as_nanos());
                println!(
                    "  Checked: {}{} ({})",
                    self.current_benchmark.to_string_lossy(),
                    group_str,
                    formatted_duration
                );
            }
        } else {
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
}

impl Default for CodSpeed {
    fn default() -> Self {
        Self::new()
    }
}
