//! Change Tracker — reads recent changes from the memory and decision stores.

use crate::types::*;
use ares_core::{AresError, MemoryFilter, MemoryType, Pagination, ProjectId};
use ares_decision_intelligence::DecisionSummary;
use ares_store::repositories::decision::SqliteDecisionRepository;
use ares_store::repositories::memory::SqliteMemoryRepository;
use std::sync::Arc;
use tracing::debug;

pub struct ChangeTracker {
    memory_repo: Arc<SqliteMemoryRepository>,
    decision_repo: Arc<SqliteDecisionRepository>,
}

impl ChangeTracker {
    pub fn new(
        memory_repo: Arc<SqliteMemoryRepository>,
        decision_repo: Arc<SqliteDecisionRepository>,
    ) -> Self {
        Self {
            memory_repo,
            decision_repo,
        }
    }

    /// Get recent changes for a project, combining memories and decisions.
    pub fn get_recent_changes(
        &self,
        project_id: &ProjectId,
        limit: u32,
    ) -> Result<Vec<ChangeRecord>, AresError> {
        let mut changes = Vec::new();

        // Get recent memories of each type
        let filter = MemoryFilter::default();
        let page = Pagination {
            page: 1,
            page_size: limit,
        };
        let memories = self.memory_repo.list(project_id, filter, page)?;

        for mem in &memories.items {
            let change_type = match mem.memory_type {
                MemoryType::Decision => ChangeType::DecisionMade,
                MemoryType::Feature => ChangeType::FeatureAdded,
                MemoryType::Bug => ChangeType::BugFixed,
                _ => {
                    if mem.version > 1 {
                        ChangeType::MemoryUpdated
                    } else {
                        ChangeType::MemoryCreated
                    }
                }
            };

            changes.push(ChangeRecord {
                change_type,
                description: mem.title.clone(),
                files_affected: vec![],
                timestamp: mem.created_at,
            });
        }

        // Get recent decisions
        let decisions = self
            .decision_repo
            .list(project_id, ares_core::DecisionFilter::default())?;
        for d in decisions.into_iter().take(limit as usize) {
            changes.push(ChangeRecord {
                change_type: ChangeType::DecisionMade,
                description: format!("Decision '{}' ({})", d.title, d.status.as_str()),
                files_affected: d.files_impacted,
                timestamp: d.created_at,
            });
        }

        // Sort by timestamp descending
        changes.sort_by_key(|b| std::cmp::Reverse(b.timestamp));
        changes.truncate(limit as usize);

        debug!(
            project_id = %project_id,
            changes = changes.len(),
            "Tracked recent changes"
        );

        Ok(changes)
    }

    /// Get decision summaries for a project.
    pub fn get_decisions(&self, project_id: &ProjectId) -> Result<Vec<DecisionSummary>, AresError> {
        let decisions = self
            .decision_repo
            .list(project_id, ares_core::DecisionFilter::default())?;

        Ok(decisions
            .into_iter()
            .map(|d| {
                let status_str = d.status.as_str().to_lowercase();
                let approval_status = match status_str.as_str() {
                    "accepted" | "approved" => ares_decision_intelligence::DecisionStatus::Approved,
                    "rejected" => ares_decision_intelligence::DecisionStatus::Rejected,
                    "deprecated" | "superseded" => {
                        ares_decision_intelligence::DecisionStatus::Deprecated
                    }
                    _ => ares_decision_intelligence::DecisionStatus::Proposed,
                };

                DecisionSummary {
                    id: d.id.as_str().to_string(),
                    title: d.title,
                    approval_status,
                }
            })
            .collect())
    }

    /// Get feature summaries from memory store.
    pub fn get_features(&self, project_id: &ProjectId) -> Result<Vec<FeatureSummary>, AresError> {
        let filter = MemoryFilter {
            memory_type: Some(MemoryType::Feature),
            ..Default::default()
        };
        let page = Pagination {
            page: 1,
            page_size: 100,
        };
        let memories = self.memory_repo.list(project_id, filter, page)?;

        Ok(memories
            .items
            .into_iter()
            .map(|m| FeatureSummary {
                id: m.id.as_str().to_string(),
                title: m.title,
                status: m.status.as_str().to_string(),
                created_at: m.created_at,
            })
            .collect())
    }

    /// Get bug summaries from memory store.
    pub fn get_bugs(&self, project_id: &ProjectId) -> Result<Vec<BugSummary>, AresError> {
        let filter = MemoryFilter {
            memory_type: Some(MemoryType::Bug),
            ..Default::default()
        };
        let page = Pagination {
            page: 1,
            page_size: 100,
        };
        let memories = self.memory_repo.list(project_id, filter, page)?;

        Ok(memories
            .items
            .into_iter()
            .map(|m| BugSummary {
                id: m.id.as_str().to_string(),
                title: m.title,
                status: m.status.as_str().to_string(),
                severity: m.importance.as_str().to_string(),
                created_at: m.created_at,
            })
            .collect())
    }
}
