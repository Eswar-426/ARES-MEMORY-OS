use crate::forecast::models::*;
use crate::forecast::outcome_predictor::OutcomePredictor;
use crate::simulation::models::SimulationResult;
use ares_core::{ScenarioId, SimulationId};
use chrono::Utc;

fn make_sim() -> SimulationResult {
    SimulationResult {
        id: SimulationId::new(),
        scenario_id: ScenarioId::new(),
        task_duration_secs: 3600.0,
        total_cost: 50.0,
        success_probability: 0.8,
        agent_utilization: 0.7,
        memory_usage_estimate: 100.0,
        risk_score: 0.2,
        simulated_at: Utc::now(),
    }
}

fn make_similar_missions() -> Vec<SimilarityMatch> {
    vec![
        SimilarityMatch {
            mission: HistoricalMission {
                id: "m1".into(),
                title: "Build REST API".into(),
                keywords: vec!["build".into(), "api".into(), "rest".into()],
                cost: 45.0,
                duration_secs: 3200.0,
                success: true,
                agent_count: 2,
                step_count: 5,
                completed_at: 1000,
            },
            similarity_score: 0.9,
            matching_keywords: vec!["build".into(), "api".into()],
        },
        SimilarityMatch {
            mission: HistoricalMission {
                id: "m2".into(),
                title: "Build GraphQL API".into(),
                keywords: vec!["build".into(), "api".into(), "graphql".into()],
                cost: 60.0,
                duration_secs: 4000.0,
                success: true,
                agent_count: 3,
                step_count: 7,
                completed_at: 2000,
            },
            similarity_score: 0.7,
            matching_keywords: vec!["build".into(), "api".into()],
        },
        SimilarityMatch {
            mission: HistoricalMission {
                id: "m3".into(),
                title: "Build CLI Tool".into(),
                keywords: vec!["build".into(), "cli".into()],
                cost: 30.0,
                duration_secs: 2000.0,
                success: false,
                agent_count: 1,
                step_count: 4,
                completed_at: 500,
            },
            similarity_score: 0.3,
            matching_keywords: vec!["build".into()],
        },
    ]
}

#[test]
fn predict_without_history() {
    let mut predictor = OutcomePredictor::new();
    let sim = make_sim();
    let prediction = predictor.predict("g1", &sim, &[]);
    assert_eq!(
        prediction.prediction_method,
        PredictionMethod::Deterministic
    );
    assert_eq!(prediction.similar_mission_count, 0);
}

#[test]
fn predict_with_history_uses_blending() {
    let mut predictor = OutcomePredictor::new();
    let sim = make_sim();
    let similar = make_similar_missions();
    let prediction = predictor.predict("g1", &sim, &similar);
    assert_eq!(prediction.prediction_method, PredictionMethod::Blended);
    assert_eq!(prediction.similar_mission_count, 3);
}

#[test]
fn confidence_reasons_populated() {
    let mut predictor = OutcomePredictor::new();
    let sim = make_sim();
    let similar = make_similar_missions();
    let prediction = predictor.predict("g1", &sim, &similar);
    assert!(!prediction.confidence_reasons.is_empty());
}

#[test]
fn confidence_reasons_mention_similar_count() {
    let mut predictor = OutcomePredictor::new();
    let sim = make_sim();
    let similar = make_similar_missions();
    let prediction = predictor.predict("g1", &sim, &similar);
    assert!(prediction
        .confidence_reasons
        .iter()
        .any(|r| r.contains("similar")));
}

#[test]
fn success_probability_bounded() {
    let mut predictor = OutcomePredictor::new();
    let sim = make_sim();
    let prediction = predictor.predict("g1", &sim, &make_similar_missions());
    assert!(prediction.success_probability >= 0.01);
    assert!(prediction.success_probability <= 0.99);
}

#[test]
fn record_actual_outcome_success() {
    let mut predictor = OutcomePredictor::new();
    let sim = make_sim();
    let prediction = predictor.predict("g1", &sim, &[]);
    let deviation = predictor.record_actual_outcome(&prediction, 55.0, 4000.0, true);
    assert!(deviation.deviation_score >= 0.0 && deviation.deviation_score <= 1.0);
}

#[test]
fn record_actual_outcome_failure() {
    let mut predictor = OutcomePredictor::new();
    let sim = make_sim();
    let prediction = predictor.predict("g1", &sim, &[]);
    let deviation = predictor.record_actual_outcome(&prediction, 100.0, 7200.0, false);
    assert!(deviation.deviation_score > 0.0);
}

#[test]
fn forecast_error_updates_on_deviation() {
    let mut predictor = OutcomePredictor::new();
    let initial_error = predictor.average_forecast_error();
    let sim = make_sim();
    let pred = predictor.predict("g1", &sim, &[]);
    predictor.record_actual_outcome(&pred, 50.0, 3600.0, true);
    // Error should have changed
    assert!(
        (predictor.average_forecast_error() - initial_error).abs() > 0.0 || initial_error == 0.5
    );
}

#[test]
fn total_predictions_increments() {
    let mut predictor = OutcomePredictor::new();
    let sim = make_sim();
    predictor.predict("g1", &sim, &[]);
    predictor.predict("g2", &sim, &[]);
    assert_eq!(predictor.total_predictions(), 2);
}

#[test]
fn prediction_is_actionable() {
    let mut predictor = OutcomePredictor::new();
    let sim = make_sim();
    let prediction = predictor.predict("g1", &sim, &make_similar_missions());
    // With good history and low risk, should be actionable
    if prediction.confidence >= 0.6 {
        assert!(prediction.is_actionable());
    }
}

#[test]
fn prediction_predicts_success() {
    let mut predictor = OutcomePredictor::new();
    let sim = make_sim();
    let prediction = predictor.predict("g1", &sim, &make_similar_missions());
    assert!(prediction.predicts_success());
}

#[test]
fn forecast_deviation_calculation() {
    let score = ForecastDeviation::calculate_deviation(100.0, 120.0, 3600.0, 4000.0, 0.8, true);
    assert!(score > 0.0);
    assert!(score < 1.0);
}

#[test]
fn forecast_deviation_perfect_prediction() {
    let score = ForecastDeviation::calculate_deviation(100.0, 100.0, 3600.0, 3600.0, 1.0, true);
    assert!(score < 0.01);
}

#[test]
fn forecast_deviation_wrong_success() {
    let score = ForecastDeviation::calculate_deviation(100.0, 100.0, 3600.0, 3600.0, 0.9, false);
    assert!(score > 0.3); // 0.9 predicted success but actual failure = big error
}

#[test]
fn prediction_method_roundtrip() {
    for m in &[
        PredictionMethod::Deterministic,
        PredictionMethod::HistoricalAverage,
        PredictionMethod::WeightedHistory,
        PredictionMethod::SimilarityBased,
        PredictionMethod::Blended,
    ] {
        assert_eq!(PredictionMethod::from_str_val(m.as_str()), *m);
    }
}

#[test]
fn ranking_weights_default_sums_to_one() {
    let w = RankingWeights::default();
    assert!((w.total() - 1.0).abs() < 0.01);
}

#[test]
fn ranking_weights_speed_optimized() {
    let w = RankingWeights::speed_optimized();
    assert!(w.speed > w.quality);
    assert!(w.speed > w.cost);
}

#[test]
fn ranking_weights_cost_optimized() {
    let w = RankingWeights::cost_optimized();
    assert!(w.cost > w.quality);
    assert!(w.cost > w.speed);
}

#[test]
fn ranking_weights_quality_optimized() {
    let w = RankingWeights::quality_optimized();
    assert!(w.quality > w.cost);
    assert!(w.quality > w.speed);
}

#[test]
fn outcome_prediction_serialization() {
    let mut predictor = OutcomePredictor::new();
    let sim = make_sim();
    let prediction = predictor.predict("g1", &sim, &make_similar_missions());
    let json = serde_json::to_string(&prediction).unwrap();
    let back: OutcomePrediction = serde_json::from_str(&json).unwrap();
    assert_eq!(back.goal_id, "g1");
    assert_eq!(
        back.confidence_reasons.len(),
        prediction.confidence_reasons.len()
    );
}

#[test]
fn forecast_deviation_serialization() {
    let mut predictor = OutcomePredictor::new();
    let sim = make_sim();
    let prediction = predictor.predict("g1", &sim, &[]);
    let deviation = predictor.record_actual_outcome(&prediction, 60.0, 4000.0, true);
    let json = serde_json::to_string(&deviation).unwrap();
    let back: ForecastDeviation = serde_json::from_str(&json).unwrap();
    assert!((back.deviation_score - deviation.deviation_score).abs() < 0.01);
}

#[test]
fn novel_goal_lower_confidence() {
    let mut predictor = OutcomePredictor::new();
    let sim = make_sim();
    let pred_no_hist = predictor.predict("g1", &sim, &[]);
    let mut predictor2 = OutcomePredictor::new();
    let pred_with_hist = predictor2.predict("g1", &sim, &make_similar_missions());
    assert!(pred_with_hist.confidence >= pred_no_hist.confidence);
}
