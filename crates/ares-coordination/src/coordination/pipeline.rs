use serde::{Deserialize, Serialize};

/// The stages of the coordination pipeline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PipelineStage {
    GoalReceived,
    OrganizationBuilt,
    ResourcesChecked,
    GovernorApproved,
    TeamCreated,
    TasksDelegated,
    Executing,
    ConsensusReached,
    Verified,
    Reflected,
    LearningRecorded,
    Completed,
    Failed(String),
}

/// Tracks the progress of a coordination pipeline execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationPipeline {
    pub goal: String,
    pub stages: Vec<PipelineStage>,
    pub current_stage: PipelineStage,
    pub started_at: i64,
    pub completed_at: Option<i64>,
}

impl CoordinationPipeline {
    pub fn new(goal: impl Into<String>) -> Self {
        Self {
            goal: goal.into(),
            stages: vec![PipelineStage::GoalReceived],
            current_stage: PipelineStage::GoalReceived,
            started_at: chrono::Utc::now().timestamp(),
            completed_at: None,
        }
    }

    /// Advance to the next stage.
    pub fn advance(&mut self, stage: PipelineStage) {
        self.stages.push(stage.clone());
        self.current_stage = stage;
    }

    /// Mark the pipeline as completed.
    pub fn complete(&mut self) {
        self.current_stage = PipelineStage::Completed;
        self.stages.push(PipelineStage::Completed);
        self.completed_at = Some(chrono::Utc::now().timestamp());
    }

    /// Mark the pipeline as failed.
    pub fn fail(&mut self, reason: impl Into<String>) {
        let reason = reason.into();
        self.current_stage = PipelineStage::Failed(reason.clone());
        self.stages.push(PipelineStage::Failed(reason));
        self.completed_at = Some(chrono::Utc::now().timestamp());
    }

    /// Check if the pipeline is complete.
    pub fn is_complete(&self) -> bool {
        matches!(
            self.current_stage,
            PipelineStage::Completed | PipelineStage::Failed(_)
        )
    }

    /// Get the number of stages completed.
    pub fn stages_completed(&self) -> usize {
        self.stages.len()
    }

    /// Get pipeline duration in seconds.
    pub fn duration_secs(&self) -> Option<i64> {
        self.completed_at.map(|end| end - self.started_at)
    }
}
