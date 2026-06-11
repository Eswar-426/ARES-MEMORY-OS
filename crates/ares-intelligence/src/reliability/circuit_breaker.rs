use crate::repository::intelligence_repository::{CircuitBreakerState, IntelligenceRepository};
use anyhow::Result;
use chrono::Utc;
use std::sync::Arc;

pub enum BreakerState {
    Closed,
    Open,
    HalfOpen,
}

pub struct CircuitBreaker {
    repo: Arc<dyn IntelligenceRepository>,
    max_failures: i64,
    reset_timeout_ms: i64,
}

impl CircuitBreaker {
    pub fn new(
        repo: Arc<dyn IntelligenceRepository>,
        max_failures: i64,
        reset_timeout_ms: i64,
    ) -> Self {
        Self {
            repo,
            max_failures,
            reset_timeout_ms,
        }
    }

    pub async fn check(&self, provider_id: &str) -> Result<BreakerState> {
        let state = self.repo.get_circuit_breaker_state(provider_id).await?;
        if let Some(state) = state {
            if state.state == "open" {
                if let Some(opened_at) = state.opened_at {
                    let now = Utc::now();
                    if now.signed_duration_since(opened_at).num_milliseconds()
                        > self.reset_timeout_ms
                    {
                        // Transition to half-open
                        self.repo
                            .save_circuit_breaker_state(CircuitBreakerState {
                                provider_id: provider_id.to_string(),
                                state: "half_open".to_string(),
                                failure_count: state.failure_count,
                                opened_at: Some(opened_at),
                            })
                            .await?;
                        return Ok(BreakerState::HalfOpen);
                    }
                }
                return Ok(BreakerState::Open);
            } else if state.state == "half_open" {
                return Ok(BreakerState::HalfOpen);
            }
        }
        Ok(BreakerState::Closed)
    }

    pub async fn record_success(&self, provider_id: &str) -> Result<()> {
        self.repo
            .save_circuit_breaker_state(CircuitBreakerState {
                provider_id: provider_id.to_string(),
                state: "closed".to_string(),
                failure_count: 0,
                opened_at: None,
            })
            .await
    }

    pub async fn record_failure(&self, provider_id: &str) -> Result<()> {
        let current = self.repo.get_circuit_breaker_state(provider_id).await?;
        let mut failures = 1;
        let mut state = "closed".to_string();
        let mut opened_at = None;

        if let Some(c) = current {
            if c.state == "half_open" {
                // Immediate open
                state = "open".to_string();
                opened_at = Some(Utc::now());
            } else {
                failures = c.failure_count + 1;
                if failures >= self.max_failures {
                    state = "open".to_string();
                    opened_at = Some(Utc::now());
                }
            }
        }

        self.repo
            .save_circuit_breaker_state(CircuitBreakerState {
                provider_id: provider_id.to_string(),
                state,
                failure_count: failures,
                opened_at,
            })
            .await
    }
}
