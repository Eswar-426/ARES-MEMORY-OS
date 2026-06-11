use crate::models::capability::TaskType;
use crate::models::profile::ModelProfile;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

pub struct ModelProfiler {
    profiles: Arc<RwLock<HashMap<Uuid, ModelProfile>>>,
    alpha_success: f64,
    alpha_latency: f64,
}

impl Default for ModelProfiler {
    fn default() -> Self {
        Self::new(0.1, 0.2) // Defaults: slow decay for success, faster reaction for latency
    }
}

impl ModelProfiler {
    pub fn new(alpha_success: f64, alpha_latency: f64) -> Self {
        Self {
            profiles: Arc::new(RwLock::new(HashMap::new())),
            alpha_success,
            alpha_latency,
        }
    }

    pub fn get_profile(&self, model_id: Uuid) -> Option<ModelProfile> {
        self.profiles.read().unwrap().get(&model_id).cloned()
    }

    pub fn update_success_rate(&self, model_id: Uuid, _task_type: TaskType, success: bool) {
        let mut map = self.profiles.write().unwrap();
        let profile = map.entry(model_id).or_insert_with(|| ModelProfile {
            model_id,
            success_rate: 1.0, // Start optimistic
            average_latency_ms: 1000,
            total_executions: 0,
        });

        let current_rate = profile.success_rate;
        let incoming_value = if success { 1.0 } else { 0.0 };

        // Exponential Moving Average
        profile.success_rate =
            (self.alpha_success * incoming_value) + ((1.0 - self.alpha_success) * current_rate);
        // Bounds checking
        profile.success_rate = profile.success_rate.clamp(0.0, 1.0);

        profile.total_executions += 1;
    }

    pub fn record_latency(&self, model_id: Uuid, latency_ms: u64) {
        let mut map = self.profiles.write().unwrap();
        let profile = map.entry(model_id).or_insert_with(|| ModelProfile {
            model_id,
            success_rate: 1.0,
            average_latency_ms: 1000,
            total_executions: 0,
        });

        let current_latency = profile.average_latency_ms as f64;
        let incoming_value = latency_ms as f64;

        // Exponential Moving Average
        let new_latency =
            (self.alpha_latency * incoming_value) + ((1.0 - self.alpha_latency) * current_latency);
        profile.average_latency_ms = new_latency.round() as u64;

        // Ensure bounds
        if profile.average_latency_ms == 0 {
            profile.average_latency_ms = 1;
        }
    }
}
