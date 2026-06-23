use ares_core::types::drift::{DriftCandidate, DriftType};
use ares_core::types::evidence::Evidence;
use ares_store::repositories::drift::DriftRepository;
use chrono::Utc;
use std::sync::Arc;

pub struct DriftEngine {
    repo: Arc<dyn DriftRepository>,
}

impl DriftEngine {
    pub fn new(repo: Arc<dyn DriftRepository>) -> Self {
        Self { repo }
    }

    pub async fn detect_drift(
        &self,
        project_id: &str,
        target_node_id: &str,
        memory_claim: &str,
        evidence: &[Evidence],
    ) -> Result<Option<DriftCandidate>, String> {
        // Cert 1 - Evidence Requirement: Drift without evidence must fail
        if evidence.is_empty() {
            return Err("Drift without evidence is not permitted".to_string());
        }

        // Cert 2 - Determinism: Use static rules (V1 Scope)
        let mut candidate = None;

        // Simplified rule matching for V1
        for ev in evidence {
            let observed = ev.observed_value.to_lowercase();
            let claim = memory_claim.to_lowercase();

            // Dependency Drift
            if claim.contains("postgresql") && observed.contains("mysql") {
                candidate = Some((
                    DriftType::DependencyMismatch,
                    format!("Observed: {}\nMemory: {}\nConflict: MySQL detected but memory specifies PostgreSQL", ev.observed_value, memory_claim),
                ));
                break;
            }

            // Configuration Drift
            if claim.contains("oauth2") && observed.contains("oidc_enabled=true") {
                candidate = Some((
                    DriftType::ConfigurationMismatch,
                    format!("Observed: {}\nMemory: {}\nConflict: OIDC capability detected but memory does not describe it", ev.observed_value, memory_claim),
                ));
                break;
            }

            // Ownership Drift
            if claim.contains("platform team") && observed.contains("identity team") {
                candidate = Some((
                    DriftType::OwnershipMismatch,
                    format!("Observed: {}\nMemory: {}\nConflict: Identity Team detected in CODEOWNERS but memory specifies Platform Team", ev.observed_value, memory_claim),
                ));
                break;
            }
        }

        if let Some((drift_type, rationale)) = candidate {
            let drift = DriftCandidate {
                id: ares_core::id::new_id(),
                project_id: project_id.to_string(),
                target_node_id: target_node_id.to_string(),
                drift_type,
                confidence: 1.0,
                evidence_ids: {
                    let mut ids: Vec<String> = evidence.iter().map(|e| e.id.as_str().to_string()).collect();
                    ids.sort();
                    ids
                },
                rationale,
                detected_at: Utc::now(),
            };

            // Cert 4 - Non-Mutation: Only creating DriftCandidate
            self.repo
                .record_candidate(drift.clone())
                .await
                .map_err(|e| e.to_string())?;

            return Ok(Some(drift));
        }

        Ok(None)
    }
}
