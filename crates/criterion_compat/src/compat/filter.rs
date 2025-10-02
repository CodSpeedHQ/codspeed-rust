use regex::Regex;

/// Benchmark filtering support - re-exported from criterion fork.
#[derive(Clone, Debug)]
pub enum BenchmarkFilter {
    /// Run all benchmarks.
    AcceptAll,
    /// Run benchmarks matching this regex.
    Regex(Regex),
    /// Run the benchmark matching this string exactly.
    Exact(String),
    /// Do not run any benchmarks.
    RejectAll,
}

impl BenchmarkFilter {
    /// Returns true if a string matches this filter.
    pub fn is_match(&self, id: &str) -> bool {
        match self {
            Self::AcceptAll => true,
            Self::Regex(r) => r.is_match(id),
            Self::Exact(e) => e == id,
            Self::RejectAll => false,
        }
    }
}
