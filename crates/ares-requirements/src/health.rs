use crate::models::RequirementStatus;
use crate::storage::RequirementStore;
use ares_core::{AresError, ProjectId, RequirementId};
use ares_store::db::Store;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementHealthScore {
    pub total_score: f64,
    pub ownership_score: f64,
    pub decision_coverage_score: f64,
    pub architecture_coverage_score: f64,
    pub code_coverage_score: f64,
    pub freshness_score: f64,
    pub status_quality_score: f64,
    pub issues: Vec<HealthIssue>,
    pub computed_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthIssue {
    pub requirement_id: RequirementId,
    pub issue_type: HealthIssueType,
    pub description: String,
    pub severity: IssueSeverity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthIssueType {
    NoOwner,
    NoDecision,
    NoArchitecture,
    NoCode,
    Stale,
    PotentialDuplicate,
    OrphanRequirement,
    StatusInconsistency,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

pub struct RequirementHealthEngine {
    store: Store,
    req_store: RequirementStore,
}

impl RequirementHealthEngine {
    pub fn new(store: Store) -> Self {
        Self {
            req_store: RequirementStore::new(store.clone()),
            store,
        }
    }

    pub fn compute_health(
        &self,
        project_id: &ProjectId,
    ) -> Result<RequirementHealthScore, AresError> {
        let filter = crate::models::RequirementFilter {
            status: None,
            priority: None,
            requirement_type: None,
            owner: None,
            tag: None,
            since: None,
            until: None,
        };

        let reqs = self.req_store.list(project_id, filter)?;
        let total = reqs.len();

        if total == 0 {
            return Ok(RequirementHealthScore {
                total_score: 100.0,
                ownership_score: 100.0,
                decision_coverage_score: 100.0,
                architecture_coverage_score: 100.0,
                code_coverage_score: 100.0,
                freshness_score: 100.0,
                status_quality_score: 100.0,
                issues: vec![],
                computed_at: Utc::now().timestamp_micros(),
            });
        }

        let mut has_owner = 0;
        let mut has_dec = 0;
        let mut has_arch = 0;
        let mut has_code = 0;
        let mut is_fresh = 0;
        let mut good_status = 0;
        let mut issues = Vec::new();

        let now = Utc::now().timestamp_micros();
        let six_months_us = 180 * 24 * 60 * 60 * 1_000_000_i64;

        for req in &reqs {
            let links = self.req_store.count_links_by_type(&req.id)?;

            let mut is_orphan = true;

            if req.owner.is_some() {
                has_owner += 1;
                is_orphan = false;
            } else {
                issues.push(HealthIssue {
                    requirement_id: req.id.clone(),
                    issue_type: HealthIssueType::NoOwner,
                    description: format!("Requirement {} has no owner.", req.id),
                    severity: IssueSeverity::Warning,
                });
            }

            if links.decision_links > 0 {
                has_dec += 1;
                is_orphan = false;
            } else {
                issues.push(HealthIssue {
                    requirement_id: req.id.clone(),
                    issue_type: HealthIssueType::NoDecision,
                    description: format!("Requirement {} is not linked to any decisions.", req.id),
                    severity: IssueSeverity::Warning,
                });
            }

            if links.architecture_links > 0 {
                has_arch += 1;
                is_orphan = false;
            } else {
                issues.push(HealthIssue {
                    requirement_id: req.id.clone(),
                    issue_type: HealthIssueType::NoArchitecture,
                    description: format!(
                        "Requirement {} is not linked to any architecture components.",
                        req.id
                    ),
                    severity: IssueSeverity::Warning,
                });
            }

            if links.code_links > 0 {
                has_code += 1;
                is_orphan = false;
            } else {
                issues.push(HealthIssue {
                    requirement_id: req.id.clone(),
                    issue_type: HealthIssueType::NoCode,
                    description: format!(
                        "Requirement {} is not linked to any code artifacts.",
                        req.id
                    ),
                    severity: IssueSeverity::Warning,
                });
            }

            if links.total > 0 {
                is_orphan = false;
            }

            if is_orphan {
                issues.push(HealthIssue {
                    requirement_id: req.id.clone(),
                    issue_type: HealthIssueType::OrphanRequirement,
                    description: format!(
                        "Requirement {} is completely orphaned (no owner, no links).",
                        req.id
                    ),
                    severity: IssueSeverity::Critical,
                });
            }

            if now - req.updated_at < six_months_us {
                is_fresh += 1;
            } else {
                issues.push(HealthIssue {
                    requirement_id: req.id.clone(),
                    issue_type: HealthIssueType::Stale,
                    description: format!(
                        "Requirement {} has not been updated in 6 months.",
                        req.id
                    ),
                    severity: IssueSeverity::Info,
                });
            }

            if req.status != RequirementStatus::Draft && req.status != RequirementStatus::Rejected {
                good_status += 1;
            }

            if req.status == RequirementStatus::Implemented && links.code_links == 0 {
                issues.push(HealthIssue {
                    requirement_id: req.id.clone(),
                    issue_type: HealthIssueType::StatusInconsistency,
                    description: format!(
                        "Requirement {} is marked Implemented but has no code links.",
                        req.id
                    ),
                    severity: IssueSeverity::Error,
                });
            }
        }

        let t = total as f64;
        let ownership_score = (has_owner as f64 / t) * 100.0;
        let decision_coverage_score = (has_dec as f64 / t) * 100.0;
        let architecture_coverage_score = (has_arch as f64 / t) * 100.0;
        let code_coverage_score = (has_code as f64 / t) * 100.0;
        let freshness_score = (is_fresh as f64 / t) * 100.0;
        let status_quality_score = (good_status as f64 / t) * 100.0;

        let total_score = (ownership_score * 0.20)
            + (decision_coverage_score * 0.20)
            + (architecture_coverage_score * 0.20)
            + (code_coverage_score * 0.20)
            + (freshness_score * 0.10)
            + (status_quality_score * 0.10);

        Ok(RequirementHealthScore {
            total_score,
            ownership_score,
            decision_coverage_score,
            architecture_coverage_score,
            code_coverage_score,
            freshness_score,
            status_quality_score,
            issues,
            computed_at: Utc::now().timestamp_micros(),
        })
    }

    pub fn save_snapshot(
        &self,
        project_id: &ProjectId,
        health: &RequirementHealthScore,
    ) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        let json_str = serde_json::to_string(health).unwrap();

        conn.execute(
            "INSERT INTO requirement_health_snapshots (id, project_id, snapshot_json, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![
                ares_core::id::new_id(),
                project_id.as_str(),
                json_str,
                health.computed_at
            ],
        )
        .map_err(AresError::db)?;

        Ok(())
    }

    pub fn get_latest_snapshot(
        &self,
        project_id: &ProjectId,
    ) -> Result<Option<RequirementHealthScore>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT snapshot_json FROM requirement_health_snapshots 
             WHERE project_id = ?1 ORDER BY created_at DESC LIMIT 1",
            )
            .map_err(AresError::db)?;

        let result = stmt.query_row(rusqlite::params![project_id.as_str()], |row| {
            row.get::<_, String>(0)
        });

        match result {
            Ok(json_str) => {
                let score = serde_json::from_str(&json_str)
                    .map_err(|e| AresError::validation(e.to_string()))?;
                Ok(Some(score))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AresError::db(e)),
        }
    }
}
