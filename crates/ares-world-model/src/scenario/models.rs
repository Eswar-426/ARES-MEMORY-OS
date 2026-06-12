use ares_core::ScenarioId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Type of scenario generated.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScenarioType {
    Fastest,
    Cheapest,
    HighestQuality,
    Balanced,
    Custom(String),
}

impl ScenarioType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Fastest => "fastest",
            Self::Cheapest => "cheapest",
            Self::HighestQuality => "highest_quality",
            Self::Balanced => "balanced",
            Self::Custom(s) => s.as_str(),
        }
    }

    pub fn from_str_val(s: &str) -> Self {
        match s {
            "fastest" => Self::Fastest,
            "cheapest" => Self::Cheapest,
            "highest_quality" => Self::HighestQuality,
            "balanced" => Self::Balanced,
            other => Self::Custom(other.to_string()),
        }
    }

    /// Returns the four standard scenario types.
    pub fn standard_types() -> Vec<ScenarioType> {
        vec![
            ScenarioType::Fastest,
            ScenarioType::Cheapest,
            ScenarioType::HighestQuality,
            ScenarioType::Balanced,
        ]
    }
}

impl std::fmt::Display for ScenarioType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A generated scenario representing one possible future.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    pub id: ScenarioId,
    pub goal_id: String,
    pub scenario_type: ScenarioType,
    pub description: String,
    pub estimated_cost: f64,
    pub estimated_duration_secs: f64,
    pub estimated_quality: f64,
    pub agent_assignments: Vec<String>,
    pub steps: Vec<ScenarioStep>,
    pub created_at: DateTime<Utc>,
}

impl Scenario {
    /// Total cost across all steps.
    pub fn total_step_cost(&self) -> f64 {
        self.steps.iter().map(|s| s.cost).sum()
    }

    /// Total duration across all steps (sequential sum).
    pub fn total_step_duration(&self) -> f64 {
        self.steps.iter().map(|s| s.duration_secs).sum()
    }

    /// Number of steps.
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }
}

/// A single step within a scenario.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioStep {
    pub order: u32,
    pub title: String,
    pub duration_secs: f64,
    pub cost: f64,
    pub agent: Option<String>,
}

/// Configuration for scenario generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioGenerationConfig {
    /// Whether to generate all standard scenario types.
    pub generate_all_standard: bool,
    /// Additional custom scenario types to generate.
    pub custom_types: Vec<String>,
    /// Maximum number of steps per scenario.
    pub max_steps: u32,
    /// Base cost multiplier (e.g., for different environments).
    pub cost_multiplier: f64,
    /// Base duration multiplier.
    pub duration_multiplier: f64,
}

impl Default for ScenarioGenerationConfig {
    fn default() -> Self {
        Self {
            generate_all_standard: true,
            custom_types: Vec::new(),
            max_steps: 20,
            cost_multiplier: 1.0,
            duration_multiplier: 1.0,
        }
    }
}
