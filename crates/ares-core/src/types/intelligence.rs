use crate::id::{MemoryId, NodeId, ProjectId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessContext {
    Query,
    BackgroundScan,
    ContextAssembly,
    Retrieval,
}

impl AccessContext {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Query => "query",
            Self::BackgroundScan => "background_scan",
            Self::ContextAssembly => "context_assembly",
            Self::Retrieval => "retrieval",
        }
    }
}

impl std::str::FromStr for AccessContext {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "query" => Ok(Self::Query),
            "background_scan" => Ok(Self::BackgroundScan),
            "context_assembly" => Ok(Self::ContextAssembly),
            "retrieval" => Ok(Self::Retrieval),
            other => Err(format!("Unknown access context: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryAccessLog {
    pub id: String,
    pub memory_id: MemoryId,
    pub project_id: ProjectId,
    pub accessed_at: i64,
    pub context: AccessContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankingCache {
    pub memory_id: MemoryId,
    pub project_id: ProjectId,
    pub score: f32,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ContradictionRecord {
    pub id: String,
    pub project_id: ProjectId,
    pub source_id: NodeId,
    pub target_id: NodeId,
    pub reason: String,
    pub confidence: f32,
    pub created_at: i64,
    pub resolved_at: Option<i64>,
}
