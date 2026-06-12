use serde::{Deserialize, Serialize};

/// Type of counterfactual scenario ("what if" question).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CounterfactualType {
    AgentFailure,
    ProviderUnavailable,
    BudgetReduction,
    ToolAccessLost,
    DeadlineTightened,
    ResourceReduction,
    QualityIncrease,
    Custom(String),
}

impl CounterfactualType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::AgentFailure => "agent_failure",
            Self::ProviderUnavailable => "provider_unavailable",
            Self::BudgetReduction => "budget_reduction",
            Self::ToolAccessLost => "tool_access_lost",
            Self::DeadlineTightened => "deadline_tightened",
            Self::ResourceReduction => "resource_reduction",
            Self::QualityIncrease => "quality_increase",
            Self::Custom(s) => s.as_str(),
        }
    }

    pub fn from_str_val(s: &str) -> Self {
        match s {
            "agent_failure" => Self::AgentFailure,
            "provider_unavailable" => Self::ProviderUnavailable,
            "budget_reduction" => Self::BudgetReduction,
            "tool_access_lost" => Self::ToolAccessLost,
            "deadline_tightened" => Self::DeadlineTightened,
            "resource_reduction" => Self::ResourceReduction,
            "quality_increase" => Self::QualityIncrease,
            other => Self::Custom(other.to_string()),
        }
    }

    /// Returns the standard "what if" scenarios.
    pub fn standard_counterfactuals() -> Vec<CounterfactualType> {
        vec![
            CounterfactualType::AgentFailure,
            CounterfactualType::ProviderUnavailable,
            CounterfactualType::BudgetReduction,
            CounterfactualType::ToolAccessLost,
            CounterfactualType::DeadlineTightened,
        ]
    }
}

/// A counterfactual question — "What if X happens?"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Counterfactual {
    pub id: String,
    pub counterfactual_type: CounterfactualType,
    pub description: String,
    /// Parameter controlling the severity (e.g., 0.8 = 80% budget reduction).
    pub parameter: f64,
}

/// Result of evaluating a counterfactual against a simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterfactualResult {
    pub counterfactual: Counterfactual,
    pub original_success_probability: f64,
    pub adjusted_success_probability: f64,
    pub original_cost: f64,
    pub adjusted_cost: f64,
    pub original_duration_secs: f64,
    pub adjusted_duration_secs: f64,
    pub impact_score: f64,
    pub mitigation_suggestions: Vec<String>,
}

impl CounterfactualResult {
    /// How much the success probability dropped (positive = worse).
    pub fn success_delta(&self) -> f64 {
        self.original_success_probability - self.adjusted_success_probability
    }

    /// Whether this counterfactual has significant impact (>10% success drop).
    pub fn is_significant(&self) -> bool {
        self.impact_score > 0.1
    }

    /// Whether this counterfactual is critical (>30% success drop).
    pub fn is_critical(&self) -> bool {
        self.impact_score > 0.3
    }
}

/// Summary of all counterfactual evaluations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterfactualSummary {
    pub total_evaluated: usize,
    pub significant_count: usize,
    pub critical_count: usize,
    pub most_impactful: Option<String>,
    pub average_impact: f64,
}
