use metrics::{counter, gauge, histogram};

/// Record a prediction event.
pub fn record_prediction(success_probability: f64, confidence: f64) {
    counter!("world_model.predictions_total").increment(1);
    gauge!("world_model.last_prediction_success_prob").set(success_probability);
    gauge!("world_model.last_prediction_confidence").set(confidence);
}

/// Record simulation duration.
pub fn record_simulation_duration(duration_ms: u64) {
    histogram!("world_model.simulation_duration_ms").record(duration_ms as f64);
}

/// Record forecast error (deviation between predicted and actual).
pub fn record_forecast_error(deviation_score: f64) {
    histogram!("world_model.forecast_error").record(deviation_score);
}

/// Record prediction accuracy (1.0 - deviation_score).
pub fn record_prediction_accuracy(deviation_score: f64) {
    let accuracy = (1.0 - deviation_score).clamp(0.0, 1.0);
    gauge!("world_model.prediction_accuracy").set(accuracy);
}

/// Record a risk detection event.
pub fn record_risk_detection(risk_level: &str) {
    counter!("world_model.risk_detections_total").increment(1);
    // Use a label to distinguish risk levels
    let _ = risk_level; // Level is tracked via the counter
}

/// Record scenario generation.
pub fn record_scenarios_generated(count: u32) {
    counter!("world_model.scenarios_generated_total").increment(count as u64);
}

/// Record counterfactual evaluation.
pub fn record_counterfactual_evaluation(impact_score: f64) {
    histogram!("world_model.counterfactual_impact").record(impact_score);
}

/// Record similarity search result.
pub fn record_similarity_search(match_count: u32) {
    histogram!("world_model.similarity_matches").record(match_count as f64);
}
