use ares_core::{PredictionId, ScenarioId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Predicted outcome for a scenario or goal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutcomePrediction {
    pub id: PredictionId,
    pub goal_id: String,
    pub scenario_id: Option<ScenarioId>,
    pub success_probability: f64,
    pub estimated_cost: f64,
    pub estimated_duration_secs: f64,
    pub confidence: f64,
    pub confidence_reasons: Vec<String>,
    pub similar_mission_count: u32,
    pub prediction_method: PredictionMethod,
    pub predicted_at: DateTime<Utc>,
}

impl OutcomePrediction {
    /// Whether confidence is high enough to be actionable (>= 0.6).
    pub fn is_actionable(&self) -> bool {
        self.confidence >= 0.6
    }

    /// Whether the prediction suggests likely success.
    pub fn predicts_success(&self) -> bool {
        self.success_probability >= 0.5
    }
}

/// Method used for prediction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PredictionMethod {
    Deterministic,
    HistoricalAverage,
    WeightedHistory,
    SimilarityBased,
    Blended,
}

impl PredictionMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Deterministic => "deterministic",
            Self::HistoricalAverage => "historical_average",
            Self::WeightedHistory => "weighted_history",
            Self::SimilarityBased => "similarity_based",
            Self::Blended => "blended",
        }
    }

    pub fn from_str_val(s: &str) -> Self {
        match s {
            "deterministic" => Self::Deterministic,
            "historical_average" => Self::HistoricalAverage,
            "weighted_history" => Self::WeightedHistory,
            "similarity_based" => Self::SimilarityBased,
            "blended" => Self::Blended,
            _ => Self::Deterministic,
        }
    }
}

/// Ranked strategy with composite scoring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyRanking {
    pub scenario_id: ScenarioId,
    pub rank: u32,
    pub composite_score: f64,
    pub speed_score: f64,
    pub quality_score: f64,
    pub cost_score: f64,
    pub risk_score: f64,
    pub success_score: f64,
    pub explanation: String,
}

/// Configurable weights for strategy ranking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankingWeights {
    pub speed: f64,
    pub quality: f64,
    pub cost: f64,
    pub risk: f64,
    pub success: f64,
}

impl Default for RankingWeights {
    fn default() -> Self {
        Self {
            speed: 0.15,
            quality: 0.2,
            cost: 0.2,
            risk: 0.2,
            success: 0.25,
        }
    }
}

impl RankingWeights {
    /// Weights optimized for speed.
    pub fn speed_optimized() -> Self {
        Self {
            speed: 0.35,
            quality: 0.1,
            cost: 0.15,
            risk: 0.15,
            success: 0.25,
        }
    }

    /// Weights optimized for cost.
    pub fn cost_optimized() -> Self {
        Self {
            speed: 0.1,
            quality: 0.15,
            cost: 0.35,
            risk: 0.15,
            success: 0.25,
        }
    }

    /// Weights optimized for quality.
    pub fn quality_optimized() -> Self {
        Self {
            speed: 0.1,
            quality: 0.35,
            cost: 0.15,
            risk: 0.15,
            success: 0.25,
        }
    }

    /// Total weight (should sum to 1.0).
    pub fn total(&self) -> f64 {
        self.speed + self.quality + self.cost + self.risk + self.success
    }
}

/// Record of predicted vs actual outcomes for learning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastDeviation {
    pub prediction_id: PredictionId,
    pub predicted_cost: f64,
    pub actual_cost: f64,
    pub predicted_duration_secs: f64,
    pub actual_duration_secs: f64,
    pub predicted_success: f64,
    pub actual_success: bool,
    pub deviation_score: f64,
    pub recorded_at: DateTime<Utc>,
}

impl ForecastDeviation {
    /// Calculate the deviation score from predicted vs actual values.
    pub fn calculate_deviation(
        predicted_cost: f64,
        actual_cost: f64,
        predicted_duration: f64,
        actual_duration: f64,
        predicted_success: f64,
        actual_success: bool,
    ) -> f64 {
        let cost_error = if predicted_cost > 0.0 {
            ((actual_cost - predicted_cost) / predicted_cost).abs()
        } else {
            0.0
        };

        let duration_error = if predicted_duration > 0.0 {
            ((actual_duration - predicted_duration) / predicted_duration).abs()
        } else {
            0.0
        };

        let success_error = if actual_success {
            1.0 - predicted_success
        } else {
            predicted_success
        };

        // Weighted combination
        (cost_error * 0.3 + duration_error * 0.3 + success_error * 0.4).clamp(0.0, 1.0)
    }
}

/// A historical mission record used for similarity matching.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalMission {
    pub id: String,
    pub title: String,
    pub keywords: Vec<String>,
    pub cost: f64,
    pub duration_secs: f64,
    pub success: bool,
    pub agent_count: u32,
    pub step_count: u32,
    pub completed_at: i64,
}

/// Result of finding similar missions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityMatch {
    pub mission: HistoricalMission,
    pub similarity_score: f64,
    pub matching_keywords: Vec<String>,
}
