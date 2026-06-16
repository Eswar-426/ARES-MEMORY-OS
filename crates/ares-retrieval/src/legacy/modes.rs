use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RetrievalMode {
    /// Normal semantic + graph hybrid search
    #[default]
    General,
    /// Heavily weights recency, ideal for context assembly
    RecentContext,
    /// Summarizes the entire project architecture and state
    ProjectSummary,
    /// Only searches decisions
    DecisionHistory,
    /// Only searches bugs and error resolutions
    BugHistory,
    /// Only searches feature planning and implementations
    FeatureHistory,
    /// Only searches architecture patterns and design docs
    ArchitectureContext,
}
