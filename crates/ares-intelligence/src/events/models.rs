use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntelligenceEvent {
    ModelSelected(String),
    ModelRejected(String),
    RoutingCompleted(String),
    ProviderFailed(String),
    FallbackTriggered(String),
    ResponseEvaluated(String),
    LearningUpdated(String),
    BenchmarkCompleted(String),
    CollaborationCompleted(String),
}
