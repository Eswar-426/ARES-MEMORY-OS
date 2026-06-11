use crate::models::model::Model;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentConfig {
    pub experiment_id: String,
    pub model_a: Model,
    pub model_b: Model,
    pub traffic_split: f64, // 0.0 to 1.0 (e.g., 0.5 = 50% split)
}
