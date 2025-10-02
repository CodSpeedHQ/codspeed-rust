use regex::Regex;

/// Filters which benchmark to run based on name.
pub(crate) enum Filter {
    Regex(Regex),
    Exact(String),
}

impl Filter {
    /// Returns `true` if a string matches this filter.
    pub fn is_match(&self, s: &str) -> bool {
        match self {
            Self::Regex(r) => r.is_match(s),
            Self::Exact(e) => e == s,
        }
    }
}
