use crate::events::models::*;
use ares_core::types::event::{EventSource, EventType};
use ares_core::{AresError, ProjectId};
use ares_store::repositories::event::SqliteEventRepository;
use std::sync::Arc;

pub struct PlannerEventPublisher {
    repo: Arc<SqliteEventRepository>,
}

impl PlannerEventPublisher {
    pub fn new(repo: Arc<SqliteEventRepository>) -> Self {
        Self { repo }
    }

    pub fn publish_goal_created(
        &self,
        project_id: Option<ProjectId>,
        payload: GoalCreatedPayload,
    ) -> Result<(), AresError> {
        let value = serde_json::to_value(payload)?;
        self.repo.emit(
            EventType::GoalCreated,
            project_id,
            value,
            EventSource::Agent,
        )?;
        Ok(())
    }

    pub fn publish_goal_decomposed(
        &self,
        project_id: Option<ProjectId>,
        payload: GoalDecomposedPayload,
    ) -> Result<(), AresError> {
        let value = serde_json::to_value(payload)?;
        self.repo.emit(
            EventType::GoalDecomposed,
            project_id,
            value,
            EventSource::Agent,
        )?;
        Ok(())
    }

    pub fn publish_plan_generated(
        &self,
        project_id: Option<ProjectId>,
        payload: PlanGeneratedPayload,
    ) -> Result<(), AresError> {
        let value = serde_json::to_value(payload)?;
        self.repo.emit(
            EventType::PlanGenerated,
            project_id,
            value,
            EventSource::Agent,
        )?;
        Ok(())
    }

    pub fn publish_plan_approved(
        &self,
        project_id: Option<ProjectId>,
        payload: PlanApprovedPayload,
    ) -> Result<(), AresError> {
        let value = serde_json::to_value(payload)?;
        self.repo.emit(
            EventType::PlanApproved,
            project_id,
            value,
            EventSource::Agent,
        )?;
        Ok(())
    }

    pub fn publish_plan_started(
        &self,
        project_id: Option<ProjectId>,
        payload: PlanStartedPayload,
    ) -> Result<(), AresError> {
        let value = serde_json::to_value(payload)?;
        self.repo.emit(
            EventType::PlanStarted,
            project_id,
            value,
            EventSource::Agent,
        )?;
        Ok(())
    }

    pub fn publish_plan_completed(
        &self,
        project_id: Option<ProjectId>,
        payload: PlanCompletedPayload,
    ) -> Result<(), AresError> {
        let value = serde_json::to_value(payload)?;
        self.repo.emit(
            EventType::PlanCompleted,
            project_id,
            value,
            EventSource::Agent,
        )?;
        Ok(())
    }

    pub fn publish_replanning_triggered(
        &self,
        project_id: Option<ProjectId>,
        payload: ReplanningTriggeredPayload,
    ) -> Result<(), AresError> {
        let value = serde_json::to_value(payload)?;
        self.repo.emit(
            EventType::ReplanningTriggered,
            project_id,
            value,
            EventSource::Agent,
        )?;
        Ok(())
    }
}
