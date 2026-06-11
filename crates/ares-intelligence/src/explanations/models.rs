use crate::models::capability::ModelCapability;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionExplanation {
    pub task_id: String,
    pub selected_model_id: String,
    pub required_capabilities: Vec<ModelCapability>,
    pub reasoning: String,
    pub rejected_models: Vec<RejectedModel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RejectedModel {
    pub model_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingExplanation {
    pub task_id: String,
    pub primary_model_id: String,
    pub successful_model_id: String,
    pub attempts: Vec<RoutingAttempt>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingAttempt {
    pub model_id: String,
    pub error: Option<String>,
}
