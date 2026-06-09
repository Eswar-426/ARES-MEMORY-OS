use crate::events::models::PlanApprovedPayload;
use crate::events::publisher::PlannerEventPublisher;
use crate::models::approval::{ApprovalMode, PlanApproval};
use crate::models::candidate::PlanCandidate;
use crate::repository::approvals::SqliteApprovalRepository;
use ares_core::{AresError, ProjectId};
use chrono::Utc;
use std::sync::Arc;

pub struct ApprovalService {
    repo: Arc<SqliteApprovalRepository>,
    publisher: Arc<PlannerEventPublisher>,
}

impl ApprovalService {
    pub fn new(repo: Arc<SqliteApprovalRepository>, publisher: Arc<PlannerEventPublisher>) -> Self {
        Self { repo, publisher }
    }

    /// Determines if a plan candidate requires manual approval or can be auto-approved.
    pub fn process_approval(
        &self,
        project_id: Option<ProjectId>,
        candidate: &PlanCandidate,
        mode: ApprovalMode,
    ) -> Result<PlanApproval, AresError> {
        let is_approved = match mode {
            ApprovalMode::AutoApprove => true,
            ApprovalMode::HumanInTheLoop => false,
            ApprovalMode::Hybrid => {
                // Heuristic: If risk score > 0.5, require human approval
                candidate.score < 0.5 // High risk usually yields low score, wait actually score is higher = better.
                                      // For now, hybrid requires approval if score < 0.8
            }
        };

        let is_approved = if mode == ApprovalMode::Hybrid {
            candidate.score >= 0.8
        } else {
            is_approved
        };

        let approval = PlanApproval {
            id: uuid::Uuid::new_v4().to_string(),
            plan_id: ares_core::id::PlanId::new(), // In reality, map from candidate -> plan
            approved_by: if is_approved {
                Some("SYSTEM".into())
            } else {
                None
            },
            approved_at: if is_approved { Some(Utc::now()) } else { None },
            mode,
            status: if is_approved {
                crate::models::approval::ApprovalStatus::Approved
            } else {
                crate::models::approval::ApprovalStatus::Pending
            },
            notes: None,
        };

        self.repo.create(&approval)?;

        if is_approved {
            self.publisher.publish_plan_approved(
                project_id,
                PlanApprovedPayload {
                    plan_id: approval.plan_id.clone(),
                    approved_by: Some("SYSTEM".to_string()),
                },
            )?;
        }

        Ok(approval)
    }
}
