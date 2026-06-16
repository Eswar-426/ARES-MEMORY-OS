use crate::storage::RequirementStore;
use ares_core::{AresError, ProjectId, RequirementId};
use ares_store::db::Store;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementGap {
    pub requirement_id: RequirementId,
    pub gap_type: GapType,
    pub description: String,
    pub severity: GapSeverity,
    pub detected_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, std::hash::Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GapType {
    NoOwner,
    NoDecision,
    NoArchitecture,
    NoCode,
    NoTests,
    Stale,
    MissingDescription,
    Orphan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GapSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapSummary {
    pub total_requirements: u64,
    pub total_gaps: u64,
    pub gaps_by_type: Vec<(GapType, u64)>,
    pub most_gapped_requirements: Vec<(RequirementId, usize)>,
}

pub struct RequirementGapDetector {
    store: Store,
    req_store: RequirementStore,
}

impl RequirementGapDetector {
    pub fn new(store: Store) -> Self {
        Self {
            req_store: RequirementStore::new(store.clone()),
            store,
        }
    }

    pub fn detect_gaps(&self, requirement_id: &RequirementId) -> Result<Vec<RequirementGap>, AresError> {
        let req = self.req_store.get(requirement_id)?
            .ok_or_else(|| AresError::not_found("requirement", requirement_id.as_str()))?;
            
        let links = self.req_store.count_links_by_type(requirement_id)?;
        let mut gaps = Vec::new();
        let now = Utc::now().timestamp_micros();
        let six_months_us = 180 * 24 * 60 * 60 * 1_000_000_i64;
        
        let mut is_orphan = true;

        if req.owner.is_none() {
            gaps.push(RequirementGap {
                requirement_id: requirement_id.clone(),
                gap_type: GapType::NoOwner,
                description: "Missing owner".into(),
                severity: GapSeverity::Medium,
                detected_at: now,
            });
        } else {
            is_orphan = false;
        }
        
        if req.description.trim().is_empty() {
            gaps.push(RequirementGap {
                requirement_id: requirement_id.clone(),
                gap_type: GapType::MissingDescription,
                description: "Missing description".into(),
                severity: GapSeverity::Low,
                detected_at: now,
            });
        }

        if links.decision_links == 0 {
            gaps.push(RequirementGap {
                requirement_id: requirement_id.clone(),
                gap_type: GapType::NoDecision,
                description: "Missing decisions".into(),
                severity: GapSeverity::High,
                detected_at: now,
            });
        } else {
            is_orphan = false;
        }

        if links.architecture_links == 0 {
            gaps.push(RequirementGap {
                requirement_id: requirement_id.clone(),
                gap_type: GapType::NoArchitecture,
                description: "Missing architecture".into(),
                severity: GapSeverity::High,
                detected_at: now,
            });
        } else {
            is_orphan = false;
        }

        if links.code_links == 0 {
            gaps.push(RequirementGap {
                requirement_id: requirement_id.clone(),
                gap_type: GapType::NoCode,
                description: "Missing code".into(),
                severity: GapSeverity::High,
                detected_at: now,
            });
        } else {
            is_orphan = false;
        }

        if links.total > 0 {
            is_orphan = false;
        }

        if is_orphan {
            gaps.push(RequirementGap {
                requirement_id: requirement_id.clone(),
                gap_type: GapType::Orphan,
                description: "Completely orphaned".into(),
                severity: GapSeverity::Critical,
                detected_at: now,
            });
        }

        if now - req.updated_at > six_months_us {
            gaps.push(RequirementGap {
                requirement_id: requirement_id.clone(),
                gap_type: GapType::Stale,
                description: "Stale requirement (>6 months)".into(),
                severity: GapSeverity::Medium,
                detected_at: now,
            });
        }

        Ok(gaps)
    }

    pub fn detect_project_gaps(&self, project_id: &ProjectId) -> Result<Vec<RequirementGap>, AresError> {
        let reqs = self.req_store.list(project_id, crate::models::RequirementFilter {
            status: None, priority: None, requirement_type: None, owner: None, tag: None, since: None, until: None,
        })?;
        
        let mut all_gaps = Vec::new();
        for req in reqs {
            let mut gaps = self.detect_gaps(&req.id)?;
            all_gaps.append(&mut gaps);
        }
        
        Ok(all_gaps)
    }

    pub fn gap_summary(&self, project_id: &ProjectId) -> Result<GapSummary, AresError> {
        let reqs = self.req_store.list(project_id, crate::models::RequirementFilter {
            status: None, priority: None, requirement_type: None, owner: None, tag: None, since: None, until: None,
        })?;
        
        let mut gaps_by_type = std::collections::HashMap::new();
        let mut most_gapped = Vec::new();
        let mut total_gaps = 0;
        
        for req in &reqs {
            let gaps = self.detect_gaps(&req.id)?;
            total_gaps += gaps.len() as u64;
            most_gapped.push((req.id.clone(), gaps.len()));
            
            for g in gaps {
                *gaps_by_type.entry(g.gap_type).or_insert(0) += 1;
            }
        }
        
        most_gapped.sort_by(|a, b| b.1.cmp(&a.1));
        most_gapped.truncate(10);
        
        Ok(GapSummary {
            total_requirements: reqs.len() as u64,
            total_gaps,
            gaps_by_type: gaps_by_type.into_iter().collect(),
            most_gapped_requirements: most_gapped,
        })
    }
}
