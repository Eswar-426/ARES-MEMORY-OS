use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionExplanation {
    pub id: Uuid,
    pub task_id: String,
    pub decision_type: String,
    pub model_id: String,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub id: Uuid,
    pub task_id: String,
    pub selected_model_id: String,
    pub fallback_model_id: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTrace {
    pub id: Uuid,
    pub task_id: String,
    pub model_id: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub latency_ms: i64,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostEvent {
    pub id: Uuid,
    pub model_id: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub total_cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderHealthEvent {
    pub provider_id: String,
    pub status: String, // "healthy", "degraded", "down"
    pub last_checked_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerState {
    pub provider_id: String,
    pub state: String, // "closed", "open", "half_open"
    pub failure_count: i64,
    pub opened_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningEvent {
    pub id: Uuid,
    pub model_id: String,
    pub success_rate: f64,
    pub latency_ms: i64,
    pub updated_at: DateTime<Utc>,
}

#[async_trait::async_trait]
pub trait IntelligenceRepository: Send + Sync {
    // P0
    async fn save_selection_explanation(&self, explanation: SelectionExplanation) -> Result<()>;
    async fn save_routing_decision(&self, decision: RoutingDecision) -> Result<()>;
    async fn save_execution_trace(&self, trace: ExecutionTrace) -> Result<()>;

    // P1
    async fn save_cost_event(&self, event: CostEvent) -> Result<()>;
    async fn save_learning_event(&self, event: LearningEvent) -> Result<()>;

    // Reliability state
    async fn save_provider_health(&self, health: ProviderHealthEvent) -> Result<()>;
    async fn get_provider_health(&self, provider_id: &str) -> Result<Option<ProviderHealthEvent>>;

    async fn save_circuit_breaker_state(&self, state: CircuitBreakerState) -> Result<()>;
    async fn get_circuit_breaker_state(
        &self,
        provider_id: &str,
    ) -> Result<Option<CircuitBreakerState>>;
}
