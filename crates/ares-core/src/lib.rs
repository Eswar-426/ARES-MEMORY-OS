//! ares-core — Shared types, error definitions, and ID generation.
//!
//! This crate has zero external network dependencies and is the foundation
//! for all other ARES crates. All public types implement Debug + Serialize + Deserialize.

pub mod error;
pub mod id;
pub mod types;
pub mod vector;

pub use error::AresError;
pub use id::{new_id, DecisionId, EventId, MemoryId, NodeId, ProjectId, ScanRunId};
pub use types::{
    decision::{
        Alternative, CreateDecisionInput, Decision, DecisionFilter, DecisionPatch,
        DecisionSearchResult, DecisionStatus, Risk,
    },
    event::{AresEvent, EventSource, EventType},
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
