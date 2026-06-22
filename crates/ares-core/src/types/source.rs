use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the origin or source of a piece of memory.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemorySource {
    /// Explicitly documented memory (e.g., ARES .md files, Architecture docs)
    ExplicitDocumentation,
    /// Ownership definitions (e.g., CODEOWNERS)
    OwnershipConfig,
    /// Facts extracted directly from the Git tree (commits, branches, releases)
    GitHistory,
    /// Synthesized intelligence (e.g., inferred from P3.3 Intelligence)
    InferredIntelligence,
    /// External issue trackers (e.g., Jira, GitHub Issues)
    ExternalTracker,
    /// Unknown or legacy source
    Unknown,
}

/// The state of a memory source within the repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceStatus {
    /// The source was detected and successfully ingested.
    Active,
    /// The source was detected but failed ingestion.
    Failed(String),
    /// The source is not available or not applicable in this repository.
    Unavailable,
}

/// A registry that tracks the available memory sources in a repository.
/// 
/// This allows higher-level intelligence engines to know exactly what facts
/// they are operating on and whether they are missing critical context.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemorySourceRegistry {
    pub sources: HashMap<MemorySource, SourceStatus>,
}

impl MemorySourceRegistry {
    pub fn new() -> Self {
        Self {
            sources: HashMap::new(),
        }
    }

    /// Registers a source with a specific status.
    pub fn register(&mut self, source: MemorySource, status: SourceStatus) {
        self.sources.insert(source, status);
    }

    /// Returns the status of a given source.
    pub fn get_status(&self, source: &MemorySource) -> Option<&SourceStatus> {
        self.sources.get(source)
    }

    /// Returns true if the given source is active and was successfully ingested.
    pub fn is_active(&self, source: &MemorySource) -> bool {
        matches!(self.get_status(source), Some(SourceStatus::Active))
    }

    /// Returns all active memory sources.
    pub fn active_sources(&self) -> Vec<MemorySource> {
        self.sources
            .iter()
            .filter(|(_, status)| matches!(status, SourceStatus::Active))
            .map(|(source, _)| source.clone())
            .collect()
    }
}
