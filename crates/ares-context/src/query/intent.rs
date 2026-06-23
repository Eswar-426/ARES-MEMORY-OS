use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum QueryIntent {
    #[default]
    Unknown,
    FileExplanation,
    DependencyTrace,
    ArchitectureQuery,
    ComponentOwner,
    // Legacy mapping aliases
    ChangeImpact,
    DeadCodeDiscovery,

    // New canonical names
    ImpactAnalysis,
    DeadCodeQuery,
    CircularDependencyQuery,
    RiskAnalysis,

    EntryPointDiscovery,
    MemoryLookup,
    RepositoryOverview,
    ImplementationSearch,
}
