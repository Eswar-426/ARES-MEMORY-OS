use super::ab_testing::ExperimentConfig;
use super::runner::ExperimentRunner;
use crate::models::model::Model;

pub struct ExperimentService {
    runner: ExperimentRunner,
}

impl Default for ExperimentService {
    fn default() -> Self {
        Self::new(ExperimentRunner)
    }
}

impl ExperimentService {
    #[allow(dead_code)]
    pub fn new(runner: ExperimentRunner) -> Self {
        Self { runner }
    }

    #[allow(dead_code)]
    pub fn route_experiment(&self, config: &ExperimentConfig) -> Model {
        self.runner.select_model_for_request(config)
    }
}
