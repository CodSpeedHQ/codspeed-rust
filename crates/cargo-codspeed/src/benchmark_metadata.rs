use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BenchmarkMetadata {
    name: String,
    uri: String,
}

impl BenchmarkMetadata {
    pub fn new(name: String, uri: String) -> Self {
        BenchmarkMetadata { name, uri }
    }
}
