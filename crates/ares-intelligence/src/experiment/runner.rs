use super::ab_testing::ExperimentConfig;
use crate::models::model::Model;

pub struct ExperimentRunner;

impl Default for ExperimentRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl ExperimentRunner {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self
    }

    #[allow(dead_code)]
    pub fn select_model_for_request(&self, config: &ExperimentConfig) -> Model {
        // Pseudo-random split using system time to avoid adding rand dependency
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos();

        let roll = (nanos % 100) as f64 / 100.0;

        if roll < config.traffic_split {
            config.model_a.clone()
        } else {
            config.model_b.clone()
        }
    }
}
