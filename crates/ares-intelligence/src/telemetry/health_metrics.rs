use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Default)]
pub struct HealthMetrics {
    provider_health_scores: Mutex<HashMap<String, u32>>,
}

impl HealthMetrics {
    pub fn new() -> Self {
        Self {
            provider_health_scores: Mutex::new(HashMap::new()),
        }
    }

    pub fn record_health_score(&self, provider_id: &str, score: u32) {
        if let Ok(mut map) = self.provider_health_scores.lock() {
            map.insert(provider_id.to_string(), score);
        }
    }

    pub fn get_health_score(&self, provider_id: &str) -> Option<u32> {
        self.provider_health_scores
            .lock()
            .unwrap()
            .get(provider_id)
            .copied()
    }
}
