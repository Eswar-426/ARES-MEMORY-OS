use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyRule {
    MaxCostPerRequest(f64),
    AllowedProviders(Vec<String>),
    ForbiddenModels(Vec<String>),
    RequiredConfidence(f64),
    MaxLatencyMs(u64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicySet {
    pub name: String,
    pub rules: Vec<PolicyRule>,
}
