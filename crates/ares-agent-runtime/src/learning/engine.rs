use crate::learning::models::{
    AgentEffectivenessRecord, LearningProfile, MissionOutcome, StrategyPerformanceRecord,
};
use chrono::Utc;

/// EMA smoothing factor. Higher values give more weight to recent data.
const EMA_ALPHA: f64 = 0.3;

/// Learns from mission outcomes using Exponential Moving Average (EMA)
/// to track strategy and agent effectiveness over time.
pub struct LearningEngine {
    profile: LearningProfile,
}

impl Default for LearningEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl LearningEngine {
    pub fn new() -> Self {
        Self {
            profile: LearningProfile::default(),
        }
    }

    /// Record a mission outcome and update all EMA values.
    pub fn record_outcome(&mut self, outcome: MissionOutcome) {
        self.update_strategy_ema(&outcome.strategy_used, &outcome);
        self.update_overall_ema(outcome.score);
        self.profile.total_missions += 1;
    }

    /// Update EMA for a specific strategy.
    pub fn update_strategy_ema(&mut self, strategy: &str, outcome: &MissionOutcome) {
        let record = self
            .profile
            .strategy_records
            .entry(strategy.to_string())
            .or_insert_with(|| StrategyPerformanceRecord::new(strategy.to_string()));

        let success_val = if outcome.success { 1.0 } else { 0.0 };

        if record.sample_count == 0 {
            // First sample — initialise directly
            record.ema_success_rate = success_val;
            record.ema_cost = outcome.cost;
            record.ema_duration = outcome.duration_secs;
        } else {
            record.ema_success_rate = ema(record.ema_success_rate, success_val);
            record.ema_cost = ema(record.ema_cost, outcome.cost);
            record.ema_duration = ema(record.ema_duration, outcome.duration_secs);
        }

        record.sample_count += 1;
        record.last_updated = Utc::now();
    }

    /// Update EMA for a specific agent role.
    pub fn update_agent_ema(&mut self, role: &str, quality: f64, latency: f64) {
        let record = self
            .profile
            .agent_records
            .entry(role.to_string())
            .or_insert_with(|| AgentEffectivenessRecord::new(role.to_string()));

        if record.task_count == 0 {
            record.ema_quality = quality;
            record.ema_latency = latency;
        } else {
            record.ema_quality = ema(record.ema_quality, quality);
            record.ema_latency = ema(record.ema_latency, latency);
        }

        record.task_count += 1;
        record.last_updated = Utc::now();
    }

    /// Get the performance record for a strategy.
    pub fn get_strategy_performance(&self, strategy: &str) -> Option<&StrategyPerformanceRecord> {
        self.profile.strategy_records.get(strategy)
    }

    /// Get the effectiveness record for an agent role.
    pub fn get_agent_effectiveness(&self, role: &str) -> Option<&AgentEffectivenessRecord> {
        self.profile.agent_records.get(role)
    }

    /// Return the strategy with the highest EMA success rate.
    pub fn get_best_strategy(&self) -> Option<String> {
        self.profile
            .strategy_records
            .values()
            .filter(|r| r.sample_count > 0)
            .max_by(|a, b| {
                a.ema_success_rate
                    .partial_cmp(&b.ema_success_rate)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| a.sample_count.cmp(&b.sample_count))
            })
            .map(|r| r.strategy.clone())
    }

    /// Get a reference to the full learning profile.
    pub fn get_learning_profile(&self) -> &LearningProfile {
        &self.profile
    }

    /// Export historical performance data in a format suitable for the strategy engine.
    pub fn export_historical_performance(&self) -> Vec<HistoricalPerformanceExport> {
        self.profile
            .strategy_records
            .values()
            .map(|r| HistoricalPerformanceExport {
                strategy: r.strategy.clone(),
                avg_success_rate: r.ema_success_rate,
                avg_cost: r.ema_cost,
                avg_duration_secs: r.ema_duration,
                sample_count: r.sample_count,
            })
            .collect()
    }

    // ── Private ──────────────────────────────────────────────────

    fn update_overall_ema(&mut self, score: f64) {
        if self.profile.total_missions == 0 {
            self.profile.overall_ema_score = score;
        } else {
            self.profile.overall_ema_score = ema(self.profile.overall_ema_score, score);
        }
    }
}

/// Lightweight export struct (avoids coupling to ares-planner types).
#[derive(Debug, Clone)]
pub struct HistoricalPerformanceExport {
    pub strategy: String,
    pub avg_success_rate: f64,
    pub avg_cost: f64,
    pub avg_duration_secs: f64,
    pub sample_count: u32,
}

/// Exponential Moving Average: `alpha * current + (1 - alpha) * previous`.
fn ema(previous: f64, current: f64) -> f64 {
    EMA_ALPHA * current + (1.0 - EMA_ALPHA) * previous
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::MissionId;

    fn make_outcome(
        strategy: &str,
        success: bool,
        score: f64,
        cost: f64,
        dur: f64,
    ) -> MissionOutcome {
        MissionOutcome {
            mission_id: MissionId::new(),
            strategy_used: strategy.to_string(),
            success,
            score,
            cost,
            duration_secs: dur,
            completed_at: Utc::now(),
        }
    }

    #[test]
    fn record_first_outcome() {
        let mut engine = LearningEngine::new();
        let outcome = make_outcome("balanced", true, 0.9, 10.0, 60.0);
        engine.record_outcome(outcome);

        let perf = engine.get_strategy_performance("balanced").unwrap();
        assert_eq!(perf.sample_count, 1);
        assert!((perf.ema_success_rate - 1.0).abs() < f64::EPSILON);
        assert!((perf.ema_cost - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn ema_update_multiple_outcomes() {
        let mut engine = LearningEngine::new();

        engine.record_outcome(make_outcome("fastest", true, 0.9, 20.0, 30.0));
        engine.record_outcome(make_outcome("fastest", false, 0.3, 50.0, 120.0));

        let perf = engine.get_strategy_performance("fastest").unwrap();
        assert_eq!(perf.sample_count, 2);
        // EMA of success: alpha*0 + (1-alpha)*1 = 0.7
        assert!((perf.ema_success_rate - 0.7).abs() < 0.01);
        // EMA of cost: alpha*50 + (1-alpha)*20 = 15 + 14 = 29
        assert!((perf.ema_cost - 29.0).abs() < 0.01);
    }

    #[test]
    fn ema_converges_towards_recent() {
        let mut engine = LearningEngine::new();
        // 10 successes then 10 failures
        for _ in 0..10 {
            engine.record_outcome(make_outcome("test", true, 0.9, 10.0, 10.0));
        }
        for _ in 0..10 {
            engine.record_outcome(make_outcome("test", false, 0.2, 10.0, 10.0));
        }

        let perf = engine.get_strategy_performance("test").unwrap();
        // After many failures, EMA should be well below 0.5
        assert!(perf.ema_success_rate < 0.3);
    }

    #[test]
    fn get_best_strategy() {
        let mut engine = LearningEngine::new();
        engine.record_outcome(make_outcome("slow", true, 0.6, 10.0, 100.0));
        engine.record_outcome(make_outcome("fast", true, 0.9, 20.0, 10.0));
        engine.record_outcome(make_outcome("fast", true, 0.95, 15.0, 8.0));

        // Both have 100% success, but "fast" has more samples at 100%
        let best = engine.get_best_strategy().unwrap();
        assert_eq!(best, "fast");
    }

    #[test]
    fn get_best_strategy_empty() {
        let engine = LearningEngine::new();
        assert!(engine.get_best_strategy().is_none());
    }

    #[test]
    fn agent_ema_first_sample() {
        let mut engine = LearningEngine::new();
        engine.update_agent_ema("Coder", 0.9, 200.0);

        let rec = engine.get_agent_effectiveness("Coder").unwrap();
        assert!((rec.ema_quality - 0.9).abs() < f64::EPSILON);
        assert!((rec.ema_latency - 200.0).abs() < f64::EPSILON);
        assert_eq!(rec.task_count, 1);
    }

    #[test]
    fn agent_ema_multiple_samples() {
        let mut engine = LearningEngine::new();
        engine.update_agent_ema("Tester", 0.8, 100.0);
        engine.update_agent_ema("Tester", 0.6, 300.0);

        let rec = engine.get_agent_effectiveness("Tester").unwrap();
        assert_eq!(rec.task_count, 2);
        // EMA quality: 0.3*0.6 + 0.7*0.8 = 0.74
        assert!((rec.ema_quality - 0.74).abs() < 0.01);
    }

    #[test]
    fn overall_ema_tracks_scores() {
        let mut engine = LearningEngine::new();
        engine.record_outcome(make_outcome("a", true, 0.9, 0.0, 0.0));
        assert!((engine.get_learning_profile().overall_ema_score - 0.9).abs() < f64::EPSILON);

        engine.record_outcome(make_outcome("a", false, 0.3, 0.0, 0.0));
        // EMA: 0.3*0.3 + 0.7*0.9 = 0.09 + 0.63 = 0.72
        assert!((engine.get_learning_profile().overall_ema_score - 0.72).abs() < 0.01);
    }

    #[test]
    fn total_missions_counter() {
        let mut engine = LearningEngine::new();
        assert_eq!(engine.get_learning_profile().total_missions, 0);

        engine.record_outcome(make_outcome("a", true, 0.9, 0.0, 0.0));
        assert_eq!(engine.get_learning_profile().total_missions, 1);

        engine.record_outcome(make_outcome("b", false, 0.3, 0.0, 0.0));
        assert_eq!(engine.get_learning_profile().total_missions, 2);
    }

    #[test]
    fn export_historical_performance() {
        let mut engine = LearningEngine::new();
        engine.record_outcome(make_outcome("fastest", true, 0.9, 10.0, 30.0));
        engine.record_outcome(make_outcome("balanced", true, 0.8, 20.0, 60.0));

        let exports = engine.export_historical_performance();
        assert_eq!(exports.len(), 2);
    }

    #[test]
    fn multiple_strategies_tracked_independently() {
        let mut engine = LearningEngine::new();
        engine.record_outcome(make_outcome("A", true, 0.9, 10.0, 10.0));
        engine.record_outcome(make_outcome("B", false, 0.2, 50.0, 100.0));

        let a = engine.get_strategy_performance("A").unwrap();
        let b = engine.get_strategy_performance("B").unwrap();
        assert!((a.ema_success_rate - 1.0).abs() < f64::EPSILON);
        assert!((b.ema_success_rate - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn ema_function_correctness() {
        assert!((ema(0.5, 1.0) - 0.65).abs() < f64::EPSILON);
        assert!((ema(1.0, 0.0) - 0.7).abs() < f64::EPSILON);
        assert!((ema(0.0, 0.0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn unknown_strategy_returns_none() {
        let engine = LearningEngine::new();
        assert!(engine.get_strategy_performance("nonexistent").is_none());
    }

    #[test]
    fn unknown_agent_returns_none() {
        let engine = LearningEngine::new();
        assert!(engine.get_agent_effectiveness("nonexistent").is_none());
    }

    #[test]
    fn default_trait() {
        let engine = LearningEngine::default();
        assert_eq!(engine.get_learning_profile().total_missions, 0);
    }

    #[test]
    fn high_ema_alpha_gives_more_recent_weight() {
        // This test validates the EMA formula semantics
        let prev = 0.5;
        let current = 1.0;
        let result = ema(prev, current);
        // result = 0.3 * 1.0 + 0.7 * 0.5 = 0.65
        // The result should be closer to prev than to current (alpha < 0.5)
        assert!(result > prev);
        assert!(result < current);
    }
}
