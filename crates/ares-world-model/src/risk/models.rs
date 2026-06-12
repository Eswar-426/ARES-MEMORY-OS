use ares_core::RiskReportId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Category of risk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskCategory {
    Failure,
    Budget,
    Resource,
    Dependency,
    Execution,
}

impl RiskCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Failure => "failure",
            Self::Budget => "budget",
            Self::Resource => "resource",
            Self::Dependency => "dependency",
            Self::Execution => "execution",
        }
    }

    pub fn from_str_val(s: &str) -> Self {
        match s {
            "failure" => Self::Failure,
            "budget" => Self::Budget,
            "resource" => Self::Resource,
            "dependency" => Self::Dependency,
            "execution" => Self::Execution,
            _ => Self::Execution,
        }
    }
}

/// Level of risk severity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RiskLevel {
    Negligible,
    Low,
    Moderate,
    High,
    Critical,
}

impl RiskLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Negligible => "negligible",
            Self::Low => "low",
            Self::Moderate => "moderate",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }

    pub fn from_str_val(s: &str) -> Self {
        match s {
            "negligible" => Self::Negligible,
            "low" => Self::Low,
            "moderate" => Self::Moderate,
            "high" => Self::High,
            "critical" => Self::Critical,
            _ => Self::Moderate,
        }
    }

    /// Numeric value for scoring (0.0..=1.0).
    pub fn numeric(&self) -> f64 {
        match self {
            Self::Negligible => 0.05,
            Self::Low => 0.2,
            Self::Moderate => 0.5,
            Self::High => 0.75,
            Self::Critical => 0.95,
        }
    }

    /// Derive risk level from a numeric score.
    pub fn from_score(score: f64) -> Self {
        if score < 0.1 {
            Self::Negligible
        } else if score < 0.3 {
            Self::Low
        } else if score < 0.6 {
            Self::Moderate
        } else if score < 0.8 {
            Self::High
        } else {
            Self::Critical
        }
    }
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A specific risk factor identified during analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    pub category: RiskCategory,
    pub description: String,
    pub severity: f64,
    pub likelihood: f64,
}

impl RiskFactor {
    /// Combined risk score = severity × likelihood.
    pub fn impact(&self) -> f64 {
        (self.severity * self.likelihood).clamp(0.0, 1.0)
    }
}

/// Comprehensive risk analysis report for a scenario.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskReport {
    pub id: RiskReportId,
    pub scenario_id: ares_core::ScenarioId,
    pub overall_risk: RiskLevel,
    pub failure_probability: f64,
    pub budget_overrun_probability: f64,
    pub resource_exhaustion_risk: f64,
    pub dependency_risk: f64,
    pub execution_risk: f64,
    pub risk_factors: Vec<RiskFactor>,
    pub mitigations: Vec<String>,
    pub analyzed_at: DateTime<Utc>,
}

impl RiskReport {
    /// Weighted overall risk score (0.0..=1.0).
    pub fn overall_score(&self) -> f64 {
        let weights = [0.3, 0.2, 0.2, 0.15, 0.15];
        let scores = [
            self.failure_probability,
            self.budget_overrun_probability,
            self.resource_exhaustion_risk,
            self.dependency_risk,
            self.execution_risk,
        ];
        weights.iter().zip(scores.iter()).map(|(w, s)| w * s).sum()
    }

    /// Whether the risk is acceptable (moderate or below).
    pub fn is_acceptable(&self) -> bool {
        self.overall_risk <= RiskLevel::Moderate
    }

    /// Count of high-impact risk factors.
    pub fn high_impact_count(&self) -> usize {
        self.risk_factors
            .iter()
            .filter(|f| f.impact() > 0.5)
            .count()
    }
}
