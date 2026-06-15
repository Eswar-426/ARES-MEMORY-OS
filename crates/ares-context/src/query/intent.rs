use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum QueryIntent {
    #[default]
    Unknown,
    FileExplanation,
    DependencyTrace,
    ArchitectureQuery,
    ComponentOwner,
    ChangeImpact,
    DeadCodeDiscovery,
    EntryPointDiscovery,
    MemoryLookup,
    RepositoryOverview,
    ImplementationSearch,
}
