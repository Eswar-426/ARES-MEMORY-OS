use ares_core::id::NodeId;
use ares_core::types::evidence::{Evidence, EvidenceSource, EvidenceType};
use ares_store::repositories::evidence::EvidenceRepository;
use chrono::Utc;
use std::sync::Arc;

pub struct EvidenceEngine {
    repo: Arc<dyn EvidenceRepository>,
}

impl EvidenceEngine {
    pub fn new(repo: Arc<dyn EvidenceRepository>) -> Self {
        Self { repo }
    }

    /// Records a new piece of evidence in the graph.
    pub async fn record_evidence(
        &self,
        project_id: &str,
        evidence: Evidence,
    ) -> Result<(), String> {
        self.repo
            .record_evidence(project_id, evidence)
            .await
            .map_err(|e| e.to_string())
    }

    /// Retrieves all evidence associated with a given node.
    pub async fn get_evidence_for_node(
        &self,
        project_id: &str,
        node_id: &str,
    ) -> Result<Vec<Evidence>, String> {
        self.repo
            .get_evidence_for_node(project_id, node_id)
            .await
            .map_err(|e| e.to_string())
    }

    /// A simple deterministic fact extractor that looks for high-confidence static patterns.
    /// In a real system, this would be fed by a full AST or configuration scanner.
    pub fn extract_facts_from_content(
        &self,
        source_node: NodeId,
        filename: &str,
        content: &str,
    ) -> Vec<Evidence> {
        let mut evidence = Vec::new();

        // Fact 1: Dependencies (e.g., Cargo.toml, package.json)
        if filename.ends_with("Cargo.toml") || filename.ends_with("package.json") {
            if content.contains("oauth2") {
                evidence.push(Evidence {
                    id: NodeId::new(),
                    evidence_type: EvidenceType::DependencyFact,
                    source_node: source_node.clone(),
                    observed_value: "Uses OAuth2".to_string(),
                    observed_at: Utc::now(),
                    confidence: 1.0,
                    source: EvidenceSource::Scanner,
                });
            }
            if content.contains("redis") {
                evidence.push(Evidence {
                    id: NodeId::new(),
                    evidence_type: EvidenceType::DependencyFact,
                    source_node: source_node.clone(),
                    observed_value: "Uses Redis".to_string(),
                    observed_at: Utc::now(),
                    confidence: 1.0,
                    source: EvidenceSource::Scanner,
                });
            }
            if content.contains("postgres") || content.contains("pg") {
                evidence.push(Evidence {
                    id: NodeId::new(),
                    evidence_type: EvidenceType::DependencyFact,
                    source_node: source_node.clone(),
                    observed_value: "Uses PostgreSQL".to_string(),
                    observed_at: Utc::now(),
                    confidence: 1.0,
                    source: EvidenceSource::Scanner,
                });
            }
        }

        // Fact 2: Imports
        if (filename.ends_with(".rs") || filename.ends_with(".ts") || filename.ends_with(".go"))
            && (content.contains("use oauth2::") || content.contains("import oauth2"))
        {
            evidence.push(Evidence {
                id: NodeId::new(),
                evidence_type: EvidenceType::ScannerFact,
                source_node: source_node.clone(),
                observed_value: "OAuth2 capability detected".to_string(),
                observed_at: Utc::now(),
                confidence: 0.95,
                source: EvidenceSource::Scanner,
            });
        }

        // Fact 3: Configuration
        if (filename.ends_with(".env") || filename.ends_with(".yaml") || filename.ends_with(".yml"))
            && (content.contains("OIDC_ENABLED=true") || content.contains("oidc_enabled: true"))
        {
            evidence.push(Evidence {
                id: NodeId::new(),
                evidence_type: EvidenceType::ConfigurationFact,
                source_node: source_node.clone(),
                observed_value: "OIDC capability detected".to_string(),
                observed_at: Utc::now(),
                confidence: 1.0,
                source: EvidenceSource::Scanner,
            });
        }

        // Fact 4: Ownership
        if filename == "CODEOWNERS" {
            // Simplified ownership fact extraction
            evidence.push(Evidence {
                id: NodeId::new(),
                evidence_type: EvidenceType::OwnershipFact,
                source_node: source_node.clone(),
                observed_value: "Ownership fact".to_string(),
                observed_at: Utc::now(),
                confidence: 1.0,
                source: EvidenceSource::Scanner,
            });
        }

        evidence
    }
}
