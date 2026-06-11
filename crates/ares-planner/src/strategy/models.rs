use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Execution strategy for a mission.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExecutionStrategy {
    Fastest,
    Cheapest,
    HighestQuality,
    Balanced,
    ParallelFirst,
    ReliabilityFirst,
}

impl ExecutionStrategy {
    /// Returns all strategy variants for enumeration.
    pub fn all() -> &'static [ExecutionStrategy] {
        &[
            ExecutionStrategy::Fastest,
            ExecutionStrategy::Cheapest,
            ExecutionStrategy::HighestQuality,
            ExecutionStrategy::Balanced,
            ExecutionStrategy::ParallelFirst,
            ExecutionStrategy::ReliabilityFirst,
        ]
    }
}

impl std::fmt::Display for ExecutionStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fastest => write!(f, "fastest"),
            Self::Cheapest => write!(f, "cheapest"),
            Self::HighestQuality => write!(f, "highest_quality"),
            Self::Balanced => write!(f, "balanced"),
            Self::ParallelFirst => write!(f, "parallel_first"),
            Self::ReliabilityFirst => write!(f, "reliability_first"),
        }
    }
}

/// Constraints that influence strategy selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConstraints {
    /// Maximum budget in cost units. `None` means no limit.
    pub budget: Option<f64>,
    /// Maximum wall-clock time in seconds. `None` means no deadline.
    pub deadline_secs: Option<f64>,
    /// Minimum acceptable quality score (0.0..=1.0).
    pub min_quality: Option<f64>,
    /// Maximum retry count before escalation.
    pub max_retries: u32,
}

impl Default for StrategyConstraints {
    fn default() -> Self {
        Self {
            budget: None,
            deadline_secs: None,
            min_quality: None,
            max_retries: 3,
        }
    }
}

/// The result of strategy selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategySelection {
    pub strategy: ExecutionStrategy,
    /// Confidence in the choice (0.0..=1.0).
    pub confidence: f64,
    /// Human-readable reasoning for the choice.
    pub reasoning: String,
    /// The constraints that influenced the decision.
    pub constraints_applied: StrategyConstraints,
}

/// Historical performance data for a strategy, used to inform future selections.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalPerformance {
    pub strategy: ExecutionStrategy,
    pub avg_success_rate: f64,
    pub avg_cost: f64,
    pub avg_duration_secs: f64,
    pub sample_count: u32,
    pub last_updated: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_strategies_returns_six() {
        assert_eq!(ExecutionStrategy::all().len(), 6);
    }

    #[test]
    fn strategy_display() {
        assert_eq!(ExecutionStrategy::Fastest.to_string(), "fastest");
        assert_eq!(ExecutionStrategy::Balanced.to_string(), "balanced");
    }

    #[test]
    fn default_constraints() {
        let c = StrategyConstraints::default();
        assert!(c.budget.is_none());
        assert!(c.deadline_secs.is_none());
        assert!(c.min_quality.is_none());
        assert_eq!(c.max_retries, 3);
    }

    #[test]
    fn strategy_serialization_roundtrip() {
        let s = ExecutionStrategy::ParallelFirst;
        let json = serde_json::to_string(&s).unwrap();
        let back: ExecutionStrategy = serde_json::from_str(&json).unwrap();
        assert_eq!(back, ExecutionStrategy::ParallelFirst);
    }

    #[test]
    fn selection_serialization() {
        let sel = StrategySelection {
            strategy: ExecutionStrategy::Cheapest,
            confidence: 0.85,
            reasoning: "Budget is tight".to_string(),
            constraints_applied: StrategyConstraints::default(),
        };
        let json = serde_json::to_string(&sel).unwrap();
        let back: StrategySelection = serde_json::from_str(&json).unwrap();
        assert_eq!(back.strategy, ExecutionStrategy::Cheapest);
        assert!((back.confidence - 0.85).abs() < f64::EPSILON);
    }

    #[test]
    fn historical_performance_serialization() {
        let hp = HistoricalPerformance {
            strategy: ExecutionStrategy::Fastest,
            avg_success_rate: 0.9,
            avg_cost: 50.0,
            avg_duration_secs: 120.0,
            sample_count: 10,
            last_updated: Utc::now(),
        };
        let json = serde_json::to_string(&hp).unwrap();
        let back: HistoricalPerformance = serde_json::from_str(&json).unwrap();
        assert_eq!(back.sample_count, 10);
    }
}
