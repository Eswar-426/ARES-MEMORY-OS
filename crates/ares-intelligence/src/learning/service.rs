use super::profiler::ModelProfiler;
use crate::models::capability::TaskType;
use crate::models::profile::ModelProfile;
use uuid::Uuid;

pub struct LearningService {
    profiler: ModelProfiler,
}

impl Default for LearningService {
    fn default() -> Self {
        Self::new(ModelProfiler::default())
    }
}

impl LearningService {
    pub fn new(profiler: ModelProfiler) -> Self {
        Self { profiler }
    }

    pub fn get_profile(&self, model_id: Uuid) -> Option<ModelProfile> {
        self.profiler.get_profile(model_id)
    }

    pub fn process_execution_result(
        &self,
        model_id: Uuid,
        task_type: TaskType,
        success: bool,
        latency_ms: u64,
    ) {
        self.profiler
            .update_success_rate(model_id, task_type, success);
        self.profiler.record_latency(model_id, latency_ms);
    }
}
