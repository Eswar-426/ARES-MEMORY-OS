use super::GapDetector;
use crate::models::{DetectionMethod, Gap, GapSeverity, GapType};
use ares_core::{AresError, id::{ProjectId, new_id}};
use ares_requirements::storage::RequirementStore;
use ares_requirements::models::{RequirementFilter, RequirementStatus};
use ares_store::Store;
use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;

pub struct RequirementGapDetector;

#[async_trait]
impl GapDetector for RequirementGapDetector {
    fn supported_types(&self) -> Vec<GapType> {
        vec![
            GapType::MissingDecision,
            GapType::MissingImplementation,
            GapType::StaleRequirement,
        ]
    }

    async fn detect(&self, project_id: &ProjectId, store: Arc<Store>) -> Result<Vec<Gap>, AresError> {
        let req_store = RequirementStore::new((*store).clone());
        let mut gaps = Vec::new();

        let filter = RequirementFilter {
            status: None,
            priority: None,
            requirement_type: None,
            owner: None,
            tag: None,
            since: None,
            until: None,
        };

        let reqs = req_store.list(project_id, filter)?;
        let now = Utc::now().timestamp_micros();
        let six_months_us = 180 * 24 * 60 * 60 * 1_000_000_i64;

        for req in reqs {
            let links = req_store.count_links_by_type(&req.id)?;

            // Check MissingDecision
            if links.decision_links == 0 && req.status != RequirementStatus::Draft && req.status != RequirementStatus::Rejected {
                gaps.push(Gap {
                    id: format!("gap_req_dec_{}", new_id()),
                    project_id: project_id.clone(),
                    gap_type: GapType::MissingDecision,
                    description: format!("Requirement '{}' is not linked to any decisions.", req.title),
                    source_id: req.id.as_str().to_string(),
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

            // Check MissingImplementation
            if req.status == RequirementStatus::Implemented && links.code_links == 0 {
                gaps.push(Gap {
                    id: format!("gap_req_impl_{}", new_id()),
                    project_id: project_id.clone(),
                    gap_type: GapType::MissingImplementation,
                    description: format!("Requirement '{}' is marked Implemented but lacks code links.", req.title),
                    source_id: req.id.as_str().to_string(),
                    detection_method: DetectionMethod::Deterministic,
                    evidence_score: 1.0,
                    severity: GapSeverity::Critical,
                    identified_at: now,
                    metadata: HashMap::new(),
                    evidence: vec![],
                    reason: None,
                    priority_score: None,
                    impact_radius: None,
                });
            }

            // Check StaleRequirement
            if (now - req.updated_at) > six_months_us {
                gaps.push(Gap {
                    id: format!("gap_req_stale_{}", new_id()),
                    project_id: project_id.clone(),
                    gap_type: GapType::StaleRequirement,
                    description: format!("Requirement '{}' has not been updated in 6 months.", req.title),
                    source_id: req.id.as_str().to_string(),
                    detection_method: DetectionMethod::RuleBased,
                    evidence_score: 0.8,
                    severity: GapSeverity::Info,
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
