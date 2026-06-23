use super::GapDetector;
use crate::models::{DetectionMethod, Gap, GapSeverity, GapType};
use ares_core::{
    id::{new_id, ProjectId},
    AresError,
};
use ares_decision_intelligence::models::DecisionStatus;
use ares_decision_intelligence::storage::DecisionStore;
use ares_store::Store;
use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;

pub struct DecisionGapDetector;

#[async_trait]
impl GapDetector for DecisionGapDetector {
    fn supported_types(&self) -> Vec<GapType> {
        vec![GapType::MissingEvidence, GapType::MissingOwner]
    }

    async fn detect(
        &self,
        project_id: &ProjectId,
        store: Arc<Store>,
    ) -> Result<Vec<Gap>, AresError> {
        let dec_store = DecisionStore::new((*store).clone());
        let mut gaps = Vec::new();
        let now = Utc::now().timestamp_micros();

        // Normally we'd list decisions by project, but DecisionStore currently doesn't have a list(project_id) method yet.
        // We will query the DB directly to get all decisions, or add list(project_id) to DecisionStore later.
        // For now, we will execute a raw query since we have the store connection.

        let conn = store.get_conn()?;
        let mut stmt = conn
            .prepare("SELECT id, title, owner, approval_status FROM decision_records")
            .map_err(AresError::db)?;

        let decisions = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let title: String = row.get(1)?;
                let owner: Option<String> = row.get(2)?;
                let status_str: String = row.get(3)?;
                let status = serde_json::from_str(&status_str).unwrap_or(DecisionStatus::Proposed);
                Ok((id, title, owner, status))
            })
            .map_err(AresError::db)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(AresError::db)?;

        for (id, title, owner, status) in decisions {
            let dec_id = ares_core::DecisionId::from(id.clone());

            // Check MissingOwner
            if status == DecisionStatus::Approved && owner.is_none() {
                gaps.push(Gap {
                    id: format!("gap_dec_owner_{}", new_id()),
                    project_id: project_id.clone(),
                    gap_type: GapType::MissingOwner,
                    description: format!("Decision '{}' is Approved but has no owner.", title),
                    source_id: id.clone(),
                    detection_method: DetectionMethod::Deterministic,
                    evidence_score: 1.0,
                    severity: GapSeverity::Warning,
                    identified_at: now,
                    metadata: HashMap::new(),
                    evidence: vec![],
                    reason: None,
                    priority_score: None,
                    impact_radius: None,
                });
            }

            // Check MissingEvidence
            let evidence = dec_store.get_evidence(&dec_id)?;
            if evidence.is_empty()
                && (status == DecisionStatus::Approved || status == DecisionStatus::Proposed)
            {
                gaps.push(Gap {
                    id: format!("gap_dec_ev_{}", new_id()),
                    project_id: project_id.clone(),
                    gap_type: GapType::MissingEvidence,
                    description: format!("Decision '{}' has no supporting evidence.", title),
                    source_id: id.clone(),
                    detection_method: DetectionMethod::RuleBased,
                    evidence_score: 0.9,
                    severity: GapSeverity::Warning,
                    identified_at: now,
                    metadata: HashMap::new(),
                    evidence: vec![],
                    reason: None,
                    priority_score: None,
                    impact_radius: None,
                });
            }
        }

        Ok(gaps)
    }
}
