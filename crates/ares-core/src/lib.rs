//! ares-core — Shared types, error definitions, and ID generation.
//!
//! This crate has zero external network dependencies and is the foundation
//! for all other ARES crates. All public types implement Debug + Serialize + Deserialize.

pub mod error;
pub mod id;
pub mod types;

pub use error::AresError;
pub use id::{new_id, DecisionId, EventId, MemoryId, NodeId, ProjectId, ScanRunId};
pub use types::{
    decision::{Alternative, CreateDecisionInput, Decision, DecisionFilter, DecisionPatch, DecisionSearchResult, DecisionStatus, Risk},
    event::{AresEvent, EventSource, EventType},
    memory::{Memory, CreateMemoryInput, MemoryFilter, MemoryPatch, MemorySearchResult, MemorySource, MemoryStatus, MemoryType},
    node::{Contradiction, EdgeDirection, EdgeType, GraphEdge, GraphNode, ImpactEntry, ImpactGraph, NodeType},
    project::{Language, Project, ProjectMaturity},
    pagination::{Page, Pagination},
};
