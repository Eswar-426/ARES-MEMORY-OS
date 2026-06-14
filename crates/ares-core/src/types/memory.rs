use crate::id::{MemoryId, ProjectId};
use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────────
// Enumerations
// ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryType {
    Project,
    Feature,
    Bug,
    Decision,
    Architecture,
    Agent,
    Team,
    Workflow,
    Experiment,
}

impl MemoryType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Project => "project",
            Self::Feature => "feature",
            Self::Bug => "bug",
            Self::Decision => "decision",
            Self::Architecture => "architecture",
            Self::Agent => "agent",
            Self::Team => "team",
            Self::Workflow => "workflow",
            Self::Experiment => "experiment",
        }
    }
}

impl std::fmt::Display for MemoryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for MemoryType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "project" => Ok(Self::Project),
            "feature" => Ok(Self::Feature),
            "bug" => Ok(Self::Bug),
            "decision" => Ok(Self::Decision),
            "architecture" => Ok(Self::Architecture),
            "agent" => Ok(Self::Agent),
            "team" => Ok(Self::Team),
            "workflow" => Ok(Self::Workflow),
            "experiment" => Ok(Self::Experiment),
            other => Err(format!("Unknown memory type: {other}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MemoryStatus {
    #[default]
    Active,
    Deprecated,
    Archived,
}

impl MemoryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Deprecated => "deprecated",
            Self::Archived => "archived",
        }
    }
}

impl std::str::FromStr for MemoryStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(Self::Active),
            "deprecated" => Ok(Self::Deprecated),
            "archived" => Ok(Self::Archived),
            other => Err(format!("Unknown memory status: {other}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MemorySource {
    #[default]
    Human,
    Scanner,
    Agent,
    Inference,
}

impl MemorySource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Human => "human",
            Self::Scanner => "scanner",
            Self::Agent => "agent",
            Self::Inference => "inference",
        }
    }
}

impl std::str::FromStr for MemorySource {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "human" => Ok(Self::Human),
            "scanner" => Ok(Self::Scanner),
            "agent" => Ok(Self::Agent),
            "inference" => Ok(Self::Inference),
            other => Err(format!("Unknown memory source: {other}")),
        }
    }
}

// ─────────────────────────────────────────────────────────────────
// Importance Level
// ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ImportanceLevel {
    Critical,
    High,
    #[default]
    Medium,
    Low,
}

impl ImportanceLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Critical => "critical",
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
        }
    }
}

impl std::str::FromStr for ImportanceLevel {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "critical" => Ok(Self::Critical),
            "high" => Ok(Self::High),
            "medium" => Ok(Self::Medium),
            "low" => Ok(Self::Low),
            other => Err(format!("Unknown importance level: {other}")),
        }
    }
}

// ─────────────────────────────────────────────────────────────────
// Core Memory struct
// ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: MemoryId,
    pub project_id: ProjectId,
    pub memory_type: MemoryType,
    pub title: String,
    /// Structured JSON payload — shape varies per memory_type
    pub content: serde_json::Value,
    pub status: MemoryStatus,
    pub version: u32,
    /// None = this IS the root version
    pub parent_id: Option<MemoryId>,
    /// 0.0 – 1.0; human writes default 1.0; agent writes default 0.7
    pub confidence: f32,
    pub importance: ImportanceLevel,
    pub source: MemorySource,
    pub ai_assisted: bool,
    /// Unix microseconds
    pub created_at: i64,
    pub updated_at: i64,
    pub deleted_at: Option<i64>,
}

// ─────────────────────────────────────────────────────────────────
// Input / filter / patch types
// ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMemoryInput {
    pub project_id: ProjectId,
    pub memory_type: MemoryType,
    pub title: String,
    pub content: serde_json::Value,
    pub confidence: Option<f32>,
    pub importance: Option<ImportanceLevel>,
    pub source: Option<MemorySource>,
    pub ai_assisted: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryPatch {
    pub title: Option<String>,
    pub content: Option<serde_json::Value>,
    pub status: Option<MemoryStatus>,
    pub confidence: Option<f32>,
    pub importance: Option<ImportanceLevel>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryFilter {
    pub memory_type: Option<MemoryType>,
    pub status: Option<MemoryStatus>,
    pub source: Option<MemorySource>,
    pub since: Option<i64>, // created_at >= since
    pub until: Option<i64>, // created_at <= until
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySearchResult {
    pub memory: Memory,
    /// BM25 rank score from FTS5
    pub score: f64,
    /// Snippet with highlighted terms
    pub snippet: String,
}
