use chrono::{DateTime, Utc};
use std::collections::HashMap;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct Quota {
    pub max_requests_per_minute: u32,
}

pub struct QuotaState {
    pub current_requests: u32,
    pub window_start: DateTime<Utc>,
}

pub struct QuotaManager {
    quotas: HashMap<String, Quota>,
    state: RwLock<HashMap<String, QuotaState>>,
}

impl QuotaManager {
    pub fn new() -> Self {
        let mut quotas = HashMap::new();
        // Example quotas based on user prompt
        quotas.insert(
            "openai".to_string(),
            Quota {
                max_requests_per_minute: 100,
            },
        );
        quotas.insert(
            "gemini".to_string(),
            Quota {
                max_requests_per_minute: 60,
            },
        );
        quotas.insert(
            "claude".to_string(),
            Quota {
                max_requests_per_minute: 50,
            },
        );

        Self {
            quotas,
            state: RwLock::new(HashMap::new()),
        }
    }

    pub async fn check_and_consume(&self, provider_id: &str) -> bool {
        let quota = match self.quotas.get(provider_id) {
            Some(q) => q,
            None => return true, // No quota means unlimited
        };

        let now = Utc::now();
        let mut w = self.state.write().await;

        let state = w
            .entry(provider_id.to_string())
            .or_insert_with(|| QuotaState {
                current_requests: 0,
                window_start: now,
            });

        // Reset window if more than a minute has passed
        if (now - state.window_start).num_seconds() >= 60 {
            state.current_requests = 0;
            state.window_start = now;
        }

        if state.current_requests < quota.max_requests_per_minute {
            state.current_requests += 1;
            true
        } else {
            false
        }
    }
}

impl Default for QuotaManager {
    fn default() -> Self {
        Self::new()
    }
}
