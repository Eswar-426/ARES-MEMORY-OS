use ares_core::id::GoalId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The type of mission inferred from the user's goal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissionType {
    Research,
    Coding,
    Refactoring,
    Analysis,
    Debugging,
    Deployment,
    MultiStepProject,
}

/// Estimated complexity of the goal.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ComplexityEstimate {
    Trivial,
    Simple,
    Moderate,
    Complex,
    Epic,
}

impl ComplexityEstimate {
    /// Returns the recommended maximum decomposition depth for this complexity.
    pub fn max_decomposition_depth(&self) -> u32 {
        match self {
            Self::Trivial => 1,
            Self::Simple => 2,
            Self::Moderate => 3,
            Self::Complex => 4,
            Self::Epic => 5,
        }
    }

    /// Returns a numeric weight (1–5) for scoring calculations.
    pub fn weight(&self) -> f64 {
        match self {
            Self::Trivial => 1.0,
            Self::Simple => 2.0,
            Self::Moderate => 3.0,
            Self::Complex => 4.0,
            Self::Epic => 5.0,
        }
    }
}

/// High-level planning strategy to use for decomposition and execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlanningStrategy {
    /// Execute tasks one after another.
    Sequential,
    /// Execute independent tasks concurrently.
    Parallel,
    /// Repeat refine cycles until quality threshold met.
    Iterative,
    /// Decompose into a multi-level hierarchy.
    Hierarchical,
    /// Dynamically switch strategies during execution.
    Adaptive,
}

/// The output of the Meta Planner: a structured intent that drives
/// downstream decomposition and strategy selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningIntent {
    pub goal_id: GoalId,
    pub mission_type: MissionType,
    pub complexity: ComplexityEstimate,
    pub strategy: PlanningStrategy,
    pub constraints: Vec<String>,
    pub estimated_steps: u32,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complexity_ordering() {
        assert!(ComplexityEstimate::Trivial < ComplexityEstimate::Simple);
        assert!(ComplexityEstimate::Simple < ComplexityEstimate::Moderate);
        assert!(ComplexityEstimate::Moderate < ComplexityEstimate::Complex);
        assert!(ComplexityEstimate::Complex < ComplexityEstimate::Epic);
    }

    #[test]
    fn complexity_depth_increases() {
        let depths: Vec<u32> = [
            ComplexityEstimate::Trivial,
            ComplexityEstimate::Simple,
            ComplexityEstimate::Moderate,
            ComplexityEstimate::Complex,
            ComplexityEstimate::Epic,
        ]
        .iter()
        .map(|c| c.max_decomposition_depth())
        .collect();

        for window in depths.windows(2) {
            assert!(window[0] < window[1]);
        }
    }

    #[test]
    fn complexity_weight_values() {
        assert!((ComplexityEstimate::Trivial.weight() - 1.0).abs() < f64::EPSILON);
        assert!((ComplexityEstimate::Epic.weight() - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn planning_intent_serialization() {
        let intent = PlanningIntent {
            goal_id: GoalId::new(),
            mission_type: MissionType::Coding,
            complexity: ComplexityEstimate::Moderate,
            strategy: PlanningStrategy::Sequential,
            constraints: vec!["budget < 100".to_string()],
            estimated_steps: 5,
            created_at: Utc::now(),
        };

        let json = serde_json::to_string(&intent).unwrap();
        let back: PlanningIntent = serde_json::from_str(&json).unwrap();
        assert_eq!(back.mission_type, MissionType::Coding);
        assert_eq!(back.complexity, ComplexityEstimate::Moderate);
        assert_eq!(back.estimated_steps, 5);
    }

    #[test]
    fn mission_type_equality() {
        assert_eq!(MissionType::Research, MissionType::Research);
        assert_ne!(MissionType::Coding, MissionType::Debugging);
    }

    #[test]
    fn planning_strategy_equality() {
        assert_eq!(PlanningStrategy::Parallel, PlanningStrategy::Parallel);
        assert_ne!(PlanningStrategy::Sequential, PlanningStrategy::Adaptive);
    }
}
