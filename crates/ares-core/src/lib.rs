//! ares-core — Shared types, error definitions, and ID generation.
//!
//! This crate has zero external network dependencies and is the foundation
//! for all other ARES crates. All public types implement Debug + Serialize + Deserialize.

pub mod error;
pub mod id;
pub mod types;
#[path = "vector/mod.rs"]
pub mod vector;

pub use error::AresError;
pub use id::{new_id, DecisionId, EventId, MemoryId, NodeId, ProjectId, ScanRunId};
pub use types::{
    decision::{
        Alternative, CreateDecisionInput, Decision, DecisionFilter, DecisionPatch,
        DecisionSearchResult, DecisionStatus, Risk,
    },
    event::{AresEvent, EventSource, EventType},
    graph_intelligence::{
        AnalysisScope, ArchitectureHealthReport, EvidenceStep, GraphStatistics, ImpactPrediction,
        KnowledgeCluster, KnowledgeGraph, RiskAssessment, RootCauseAnalysis,
    },
    intelligence::{AccessContext, ContradictionRecord, MemoryAccessLog, RankingCache},
    memory::{
        CreateMemoryInput, ImportanceLevel, Memory, MemoryFilter, MemoryPatch, MemorySearchResult,
        MemorySource, MemoryStatus, MemoryType,
    },
    node::{
        Contradiction, EdgeDirection, EdgeType, GraphEdge, GraphNode, ImpactEntry, ImpactGraph,
        NodeType,
    },
    pagination::{Page, Pagination},
    project::{Language, Project, ProjectMaturity},
};

// Week 5 — Semantic Memory Engine re-exports
pub use vector::{
    cosine_similarity, normalize_vector, Embedding, EmbeddingMetadata, EmbeddingProvider,
    RetrievalDiagnostics, SimilarityResult, StoredEmbedding, VectorRepository,
};
