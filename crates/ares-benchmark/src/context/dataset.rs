use ares_core::AresError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkQuery {
    pub query: String,
    pub intent: String,
    pub expected_files: Vec<String>,
    pub expected_min_nodes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkDataset {
    pub queries: Vec<BenchmarkQuery>,
}

impl BenchmarkDataset {
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, AresError> {
        let content = fs::read_to_string(path)
            .map_err(|e| AresError::validation(format!("Failed to read dataset: {}", e)))?;

        let queries: Vec<BenchmarkQuery> = serde_json::from_str(&content)
            .map_err(|e| AresError::validation(format!("Failed to parse dataset: {}", e)))?;

        Ok(Self { queries })
    }
}
