use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub id: Uuid,
    pub task_id: Uuid,
    pub selected_model_id: Uuid,
    pub fallback_model_id: Option<Uuid>,
    pub reason: String,
}
