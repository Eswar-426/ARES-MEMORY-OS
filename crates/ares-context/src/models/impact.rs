use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactReport {
    pub target: String,
    pub affected_modules: Vec<String>,
    pub affected_functions: Vec<String>,
    pub depth_analyzed: usize,
}
