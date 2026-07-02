use serde::{Deserialize, Serialize};

use crate::core::engine::Artifact;
use crate::core::evidence::ProcessedEvidenceBundle;
use crate::core::metadata::{ExecutionMetadata, PlannerTrace};

// ═══════════════════════════════════════════════════════════════════
// Layer 6: RepositoryResponse — Versioned, canonical output
// ═══════════════════════════════════════════════════════════════════

pub const SCHEMA_VERSION: u32 = 1;
pub const PLANNER_VERSION: &str = "0.1.0";
pub const KNOWLEDGE_VERSION: &str = "0.1.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionSuggestion {
    pub label: String,
    pub command: String,
    pub arguments: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    pub kind: String,
    pub id: String,
    pub title: String,
    pub location: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsBundle {
    pub health_score: f32,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub missing_requirements: Vec<String>,
}

/// Knowledge pipeline output — placeholder for canonicalized, deduplicated knowledge.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KnowledgeBundle {
    pub canonical_entities: Vec<String>,
    pub relationships: Vec<String>,
    pub confidence_scores: std::collections::HashMap<String, f32>,
}

/// The single, versioned response consumed by every frontend.
/// `answer` is optional — Graph Explorer and Dashboard don't need LLM output.
/// `replay_id` references a persisted replay on disk, not embedded inline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryResponse {
    // Version control — frozen ABI
    pub schema_version: u32,
    pub planner_version: String,
    pub knowledge_version: String,

    // Identity
    pub response_id: String,
    pub execution_id: String,

    // Core output
    pub answer: Option<String>,
    pub evidence: ProcessedEvidenceBundle,
    pub knowledge: KnowledgeBundle,
    pub metadata: ExecutionMetadata,
    pub diagnostics: DiagnosticsBundle,

    // Observability
    pub planner_trace: PlannerTrace,
    pub replay_id: Option<String>,

    // Extensions
    pub artifacts: Vec<Artifact>,
    pub actions: Vec<ActionSuggestion>,
    pub citations: Vec<Citation>,
}
