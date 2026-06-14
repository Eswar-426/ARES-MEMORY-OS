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
pub use id::{
    new_id, AgentId, ClusterId, DecisionId, EpisodeId, EventId, ExecutionId, ExperienceId,
    ForecastId, LessonId, MemoryId, NodeId, PlanCandidateId, PlanId, PredictionId, PrincipleId,
    ProjectId, RiskReportId, ScanRunId, ScenarioId, SemanticMemoryId, SimulationId, StepId, TaskId,
    WorkflowId, WorldStateId,
};
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
    plan::{Goal, Milestone, Plan, PlanDetails, PlanStatus, Task, TaskDependency, TaskStatus},
    project::{Language, Project, ProjectMaturity},
    workflow::{
        AgentHealth, AgentInfo, AgentPerformance, CompensationAction, DeadLetterEntry,
        ExecutionPlan, ExecutionState, RetryPolicy, TaskPriority, WorkflowDefinition,
        WorkflowDependency, WorkflowEvent, WorkflowEventType, WorkflowExecutionSnapshot,
        WorkflowStatus, WorkflowStepDef, WORKFLOW_EVENT_SCHEMA_VERSION,
    },
};

// Week 5 — Semantic Memory Engine re-exports
pub use vector::{
    cosine_similarity, normalize_vector, Embedding, EmbeddingMetadata, EmbeddingProvider,
    RetrievalDiagnostics, SimilarityResult, StoredEmbedding, VectorRepository,
};
