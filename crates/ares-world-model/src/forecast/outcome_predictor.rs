use super::models::{ForecastDeviation, OutcomePrediction, PredictionMethod, SimilarityMatch};
use crate::simulation::models::SimulationResult;
use ares_core::PredictionId;
use chrono::Utc;

/// Predicts outcomes using historical data, simulation results,
/// and similarity matching. No LLM dependency.
pub struct OutcomePredictor {
    /// Running average of forecast error for self-calibration.
    avg_forecast_error: f64,
    /// Total predictions made (for EMA weighting).
    total_predictions: u64,
}

impl Default for OutcomePredictor {
    fn default() -> Self {
        Self::new()
    }
}

impl OutcomePredictor {
    pub fn new() -> Self {
        Self {
            avg_forecast_error: 0.5, // start pessimistic
            total_predictions: 0,
        }
    }

    /// Predict outcome from simulation results and historical data.
    pub fn predict(
        &mut self,
        goal_id: &str,
        simulation: &SimulationResult,
        similar_missions: &[SimilarityMatch],
    ) -> OutcomePrediction {
        self.total_predictions += 1;

        let (hist_success, hist_cost, hist_duration) = self.aggregate_historical(similar_missions);

        let similar_count = similar_missions.len() as u32;
        let has_history = !similar_missions.is_empty();

        // Determine prediction method and blend
        let (method, success_probability, estimated_cost, estimated_duration_secs) = if has_history
        {
            let hist_weight = (similar_count as f64 / 30.0).min(0.5);
            let sim_weight = 1.0 - hist_weight;

            let success = simulation.success_probability * sim_weight + hist_success * hist_weight;
            let cost = simulation.total_cost * sim_weight + hist_cost * hist_weight;
            let duration = simulation.task_duration_secs * sim_weight + hist_duration * hist_weight;

            (PredictionMethod::Blended, success, cost, duration)
        } else {
            (
                PredictionMethod::Deterministic,
                simulation.success_probability,
                simulation.total_cost,
                simulation.task_duration_secs,
            )
        };

        let (confidence, confidence_reasons) =
            self.calculate_confidence(similar_count, has_history, simulation);

        OutcomePrediction {
            id: PredictionId::new(),
            goal_id: goal_id.to_string(),
            scenario_id: Some(simulation.scenario_id.clone()),
            success_probability: success_probability.clamp(0.01, 0.99),
            estimated_cost,
            estimated_duration_secs,
            confidence,
            confidence_reasons,
            similar_mission_count: similar_count,
            prediction_method: method,
            predicted_at: Utc::now(),
        }
    }

    /// Record actual outcome and update self-calibration.
    pub fn record_actual_outcome(
        &mut self,
        prediction: &OutcomePrediction,
        actual_cost: f64,
        actual_duration_secs: f64,
        actual_success: bool,
    ) -> ForecastDeviation {
        let deviation_score = ForecastDeviation::calculate_deviation(
            prediction.estimated_cost,
            actual_cost,
            prediction.estimated_duration_secs,
            actual_duration_secs,
            prediction.success_probability,
            actual_success,
        );

        // Update running average forecast error using EMA
        let alpha = 0.1; // smoothing factor
        self.avg_forecast_error = self.avg_forecast_error * (1.0 - alpha) + deviation_score * alpha;

        ForecastDeviation {
            prediction_id: prediction.id.clone(),
            predicted_cost: prediction.estimated_cost,
            actual_cost,
            predicted_duration_secs: prediction.estimated_duration_secs,
            actual_duration_secs,
            predicted_success: prediction.success_probability,
            actual_success,
            deviation_score,
            recorded_at: Utc::now(),
        }
    }

    /// Current average forecast error (for telemetry/calibration).
    pub fn average_forecast_error(&self) -> f64 {
        self.avg_forecast_error
    }

    /// Total predictions made.
    pub fn total_predictions(&self) -> u64 {
        self.total_predictions
    }

    /// Aggregate historical data from similar missions using weighted averages.
    fn aggregate_historical(&self, matches: &[SimilarityMatch]) -> (f64, f64, f64) {
        if matches.is_empty() {
            return (0.5, 0.0, 0.0);
        }

        let total_weight: f64 = matches.iter().map(|m| m.similarity_score).sum();
        if total_weight <= 0.0 {
            return (0.5, 0.0, 0.0);
        }

        let success_rate = matches
            .iter()
            .map(|m| {
                let s = if m.mission.success { 1.0 } else { 0.0 };
                s * m.similarity_score
            })
            .sum::<f64>()
            / total_weight;

        let avg_cost = matches
            .iter()
            .map(|m| m.mission.cost * m.similarity_score)
            .sum::<f64>()
            / total_weight;

        let avg_duration = matches
            .iter()
            .map(|m| m.mission.duration_secs * m.similarity_score)
            .sum::<f64>()
            / total_weight;

        (success_rate, avg_cost, avg_duration)
    }

    /// Calculate confidence and the reasons behind it.
    fn calculate_confidence(
        &self,
        similar_count: u32,
        has_history: bool,
        simulation: &SimulationResult,
    ) -> (f64, Vec<String>) {
        let mut confidence: f64 = 0.5; // start at moderate
        let mut reasons = Vec::new();

        // Historical data availability
        if similar_count >= 20 {
            confidence += 0.25;
            reasons.push(format!("{} similar missions found", similar_count));
        } else if similar_count >= 5 {
            confidence += 0.15;
            reasons.push(format!("{} similar missions found", similar_count));
        } else if has_history {
            confidence += 0.05;
            reasons.push(format!(
                "Limited history: {} similar missions",
                similar_count
            ));
        } else {
            confidence -= 0.1;
            reasons.push("Novel goal — no similar missions found".to_string());
        }

        // Forecast accuracy calibration
        if self.avg_forecast_error < 0.2 {
            confidence += 0.15;
            reasons.push("Low historical forecast error".to_string());
        } else if self.avg_forecast_error > 0.5 {
            confidence -= 0.1;
            reasons.push("High historical forecast error — calibrating".to_string());
        }

        // Simulation quality indicators
        if simulation.risk_score < 0.2 {
            confidence += 0.05;
            reasons.push("Low simulation risk".to_string());
        } else if simulation.risk_score > 0.6 {
            confidence -= 0.1;
            reasons.push("High simulation risk reduces confidence".to_string());
        }

        if simulation.success_probability > 0.8 {
            confidence += 0.05;
            reasons.push("High simulation success probability".to_string());
        }

        (confidence.clamp(0.1, 0.95), reasons)
    }
}
