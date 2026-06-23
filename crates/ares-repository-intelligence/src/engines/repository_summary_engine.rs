use crate::models::RepositoryState;

pub struct RepositorySummaryEngine;

impl Default for RepositorySummaryEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl RepositorySummaryEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn summarize(&self, state: &RepositoryState) -> String {
        format!("# Repository Summary\nPurpose: {}", state.purpose.purpose)
    }
}
