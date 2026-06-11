use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Clone, Debug)]
pub struct ProviderQuotaConfig {
    pub provider_id: String,
    pub max_requests_per_day: u64,
    pub max_tokens_per_day: u64,
    pub max_spend_per_day: f64,
}

struct QuotaState {
    requests_today: u64,
    tokens_today: u64,
    spend_today: f64,
}

pub struct ProviderQuotaManager {
    configs: HashMap<String, ProviderQuotaConfig>,
    states: Mutex<HashMap<String, QuotaState>>,
}

impl ProviderQuotaManager {
    pub fn new(configs: Vec<ProviderQuotaConfig>) -> Self {
        let mut map = HashMap::new();
        let mut states = HashMap::new();
        for config in configs {
            states.insert(
                config.provider_id.clone(),
                QuotaState {
                    requests_today: 0,
                    tokens_today: 0,
                    spend_today: 0.0,
                },
            );
            map.insert(config.provider_id.clone(), config);
        }
        Self {
            configs: map,
            states: Mutex::new(states),
        }
    }

    pub fn check_quota(&self, provider_id: &str) -> Result<()> {
        let config = self
            .configs
            .get(provider_id)
            .ok_or_else(|| anyhow!("No quota config for provider"))?;
        let states = self.states.lock().unwrap();
        let state = states.get(provider_id).unwrap();

        if state.requests_today >= config.max_requests_per_day {
            return Err(anyhow!(
                "Provider {} exceeded daily requests quota",
                provider_id
            ));
        }
        if state.tokens_today >= config.max_tokens_per_day {
            return Err(anyhow!(
                "Provider {} exceeded daily tokens quota",
                provider_id
            ));
        }
        if state.spend_today >= config.max_spend_per_day {
            return Err(anyhow!(
                "Provider {} exceeded daily spend quota",
                provider_id
            ));
        }
        Ok(())
    }

    pub fn record_usage(&self, provider_id: &str, tokens: u64, cost: f64) {
        if let Ok(mut states) = self.states.lock() {
            if let Some(state) = states.get_mut(provider_id) {
                state.requests_today += 1;
                state.tokens_today += tokens;
                state.spend_today += cost;
            }
        }
    }
}
