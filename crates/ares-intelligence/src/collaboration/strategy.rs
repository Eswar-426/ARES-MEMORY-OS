use crate::models::model::Model;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollaborationStrategy {
    SingleModel,
    ReasonAndVerify,
    Debate,
    Consensus,
    SpecialistPipeline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationConfig {
    pub strategy: CollaborationStrategy,
    pub primary_model: Model,
    pub secondary_models: Vec<Model>,
    pub max_rounds: usize,
}
