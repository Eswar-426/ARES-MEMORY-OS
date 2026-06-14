use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────────
// KnowledgeType — what kind of knowledge was extracted
// ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeType {
    Decision,
    Bug,
    Architecture,
    Experiment,
}

impl KnowledgeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Decision => "decision",
            Self::Bug => "bug",
            Self::Architecture => "architecture",
            Self::Experiment => "experiment",
        }
    }
}

impl std::fmt::Display for KnowledgeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for KnowledgeType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "decision" => Ok(Self::Decision),
            "bug" => Ok(Self::Bug),
            "architecture" => Ok(Self::Architecture),
            "experiment" => Ok(Self::Experiment),
            other => Err(format!("Unknown knowledge type: {other}")),
        }
    }
}

// ─────────────────────────────────────────────────────────────────
// KnowledgeCandidate — the intermediate extraction result
// ─────────────────────────────────────────────────────────────────

/// A structured piece of knowledge extracted from a commit by an LLM.
/// Acts as an intermediate layer between raw LLM output and persisted
/// ARES memory/decision records.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeCandidate {
    /// Unique ID for this candidate
    pub id: String,
    /// What kind of knowledge this represents
    pub knowledge_type: KnowledgeType,
    /// LLM-assigned confidence (0.0 – 1.0) that this extraction is correct
    pub confidence: f32,
    /// The LLM's reasoning for why this knowledge was extracted
    pub reasoning: String,
    /// The extracted knowledge content (human-readable summary)
    pub content: String,
    /// Short title for the knowledge item
    pub title: String,
    /// The commit hash this knowledge was extracted from
    pub source_commit: String,
    /// Optional: files mentioned or affected
    pub affected_files: Vec<String>,
    /// Whether this candidate was persisted into the knowledge store
    pub persisted: bool,
    /// Unix microsecond timestamp of extraction
    pub extracted_at: i64,
}

// ─────────────────────────────────────────────────────────────────
// ExtractionRequest — input to the extraction engine
// ─────────────────────────────────────────────────────────────────

/// Request to extract knowledge from a git commit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionRequest {
    /// Git commit hash. If empty or "HEAD", defaults to the latest commit.
    pub commit_hash: Option<String>,
    /// Path to the git repository root. Uses cwd if omitted.
    pub repo_path: Option<String>,
    /// Optional project ID to associate extracted knowledge with.
    pub project_id: Option<String>,
}

// ─────────────────────────────────────────────────────────────────
// ExtractionResult — output of the extraction engine
// ─────────────────────────────────────────────────────────────────

/// The result of processing a single commit through the extraction pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionResult {
    /// The commit hash that was processed
    pub commit_hash: String,
    /// The commit message
    pub commit_message: String,
    /// All candidates extracted (before confidence filtering)
    pub all_candidates: Vec<KnowledgeCandidate>,
    /// Candidates that passed the confidence threshold and were persisted
    pub persisted_candidates: Vec<KnowledgeCandidate>,
    /// The confidence threshold that was applied
    pub confidence_threshold: f32,
    /// Number of candidates rejected (below threshold)
    pub rejected_count: usize,
}

// ─────────────────────────────────────────────────────────────────
// ExtractionConfig — engine configuration
// ─────────────────────────────────────────────────────────────────

/// Configuration for the knowledge extraction engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionConfig {
    /// Minimum confidence to persist a candidate. Default: 0.80
    pub confidence_threshold: f32,
    /// Maximum number of candidates to extract per commit. Default: 10
    pub max_candidates_per_commit: usize,
}

impl Default for ExtractionConfig {
    fn default() -> Self {
        Self {
            confidence_threshold: 0.80,
            max_candidates_per_commit: 10,
        }
    }
}
