use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════
// Layer 4: Evidence — Raw (from engines) and Processed (from knowledge pipeline)
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GraphEvidence {
    pub nodes: Vec<String>,
    pub edges: Vec<String>,
    pub paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GitEvidence {
    pub commits: Vec<String>,
    pub authors: Vec<String>,
    pub blame: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ArchitectureEvidence {
    pub adrs: Vec<String>,
    pub requirements: Vec<String>,
    pub decisions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CodeEvidence {
    pub files: Vec<String>,
    pub functions: Vec<String>,
    pub symbols: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuntimeEvidence {
    pub confidence: f32,
    pub statistics: HashMap<String, f64>,
    pub sources: Vec<String>,
}

/// Raw evidence produced by engines. Never modified after creation.
/// The Knowledge Pipeline reads this to produce a ProcessedEvidenceBundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawEvidenceBundle {
    pub version: u32,
    pub graph: Option<GraphEvidence>,
    pub git: Option<GitEvidence>,
    pub architecture: Option<ArchitectureEvidence>,
    pub code: Option<CodeEvidence>,
    pub runtime: Option<RuntimeEvidence>,
    pub metadata: HashMap<String, String>,
}

impl Default for RawEvidenceBundle {
    fn default() -> Self {
        Self {
            version: 1,
            graph: None,
            git: None,
            architecture: None,
            code: None,
            runtime: None,
            metadata: HashMap::new(),
        }
    }
}

/// Processed evidence after validation and the Knowledge Pipeline.
/// This is the version embedded in RepositoryResponse.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedEvidenceBundle {
    pub version: u32,
    pub graph: Option<GraphEvidence>,
    pub git: Option<GitEvidence>,
    pub architecture: Option<ArchitectureEvidence>,
    pub code: Option<CodeEvidence>,
    pub runtime: Option<RuntimeEvidence>,
    pub metadata: HashMap<String, String>,
    /// Knowledge pipeline enrichments
    pub confidence: f32,
    pub canonical_ids: Vec<String>,
    pub deduplicated: bool,
}

impl Default for ProcessedEvidenceBundle {
    fn default() -> Self {
        Self {
            version: 1,
            graph: None,
            git: None,
            architecture: None,
            code: None,
            runtime: None,
            metadata: HashMap::new(),
            confidence: 0.0,
            canonical_ids: Vec::new(),
            deduplicated: false,
        }
    }
}

impl From<RawEvidenceBundle> for ProcessedEvidenceBundle {
    fn from(raw: RawEvidenceBundle) -> Self {
        Self {
            version: raw.version,
            graph: raw.graph,
            git: raw.git,
            architecture: raw.architecture,
            code: raw.code,
            runtime: raw.runtime,
            metadata: raw.metadata,
            confidence: 0.0,
            canonical_ids: Vec::new(),
            deduplicated: false,
        }
    }
}

// Backwards-compatible alias so existing code referencing EvidenceBundle keeps compiling
// during the migration. Will be removed once all consumers are updated.
pub type EvidenceBundle = RawEvidenceBundle;
