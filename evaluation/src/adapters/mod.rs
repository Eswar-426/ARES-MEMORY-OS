use crate::canonical::CanonicalFact;
use crate::dataset::Evidence;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineMetadata {
    pub version: String,
    pub git_commit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryMetadata {
    pub commit: String,
    pub node_count: usize,
    pub edge_count: usize,
    pub schema_version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetMetadata {
    pub version: u32,
    pub schema: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationMetadata {
    pub timestamp: String,
    pub duration_ms: u64,
    pub stability_runs: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provenance {
    pub engine: EngineMetadata,
    pub repository: RepositoryMetadata,
    pub dataset: DatasetMetadata,
    pub evaluation: EvaluationMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeEngineResult {
    pub engine: String,
    pub answer: String,
    pub confidence: f64,
    pub evidence: Vec<Evidence>,
    pub traversal: Vec<String>,
    // In reality, this would have engine-specific claims. We abstract it here.
    pub raw_claims: Vec<crate::dataset::Claim>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationEngineResult {
    pub runtime: RuntimeEngineResult,
    pub claims: Vec<CanonicalFact>,
    pub traversal: Vec<String>,
    pub provenance: Provenance,
}

pub fn adapt_to_evaluation(
    runtime: RuntimeEngineResult,
    provenance: Provenance,
) -> EvaluationEngineResult {
    let mut claims = Vec::new();
    for rc in &runtime.raw_claims {
        let kind = match rc.kind.to_lowercase().as_str() {
            "requirement" => crate::canonical::FactKind::Requirement,
            "decision" => crate::canonical::FactKind::Decision,
            "file" => crate::canonical::FactKind::File,
            "function" => crate::canonical::FactKind::Function,
            "test" => crate::canonical::FactKind::Test,
            "architecture" => crate::canonical::FactKind::Architecture,
            "owner" => crate::canonical::FactKind::Owner,
            "dependency" => crate::canonical::FactKind::Dependency,
            _ => crate::canonical::FactKind::File, // Fallback
        };
        claims.push(CanonicalFact {
            schema_version: 1,
            kind,
            id: rc.id.clone(),
            confidence: runtime.confidence,
        });
    }

    EvaluationEngineResult {
        traversal: runtime.traversal.clone(),
        runtime,
        claims,
        provenance,
    }
}
