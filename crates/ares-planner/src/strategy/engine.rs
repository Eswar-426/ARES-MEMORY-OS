use crate::meta_planner::models::ComplexityEstimate;
use crate::strategy::models::{
    ExecutionStrategy, HistoricalPerformance, StrategyConstraints, StrategySelection,
};

/// Selects the optimal execution strategy based on constraints,
/// complexity, and historical performance data.
pub struct StrategyEngine;

impl Default for StrategyEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl StrategyEngine {
    pub fn new() -> Self {
        Self
    }

    /// Select the best strategy by scoring all candidates and returning the winner.
    pub fn select_strategy(
        &self,
        constraints: &StrategyConstraints,
        complexity: &ComplexityEstimate,
        history: &[HistoricalPerformance],
    ) -> StrategySelection {
        let mut best_strategy = ExecutionStrategy::Balanced;
        let mut best_score: f64 = f64::NEG_INFINITY;
        let mut best_reasoning = String::new();

        for strategy in ExecutionStrategy::all() {
            let hist = history.iter().find(|h| h.strategy == *strategy);
            let score = self.score_strategy(strategy, constraints, complexity, hist);

            if score > best_score {
                best_score = score;
                best_strategy = strategy.clone();
                best_reasoning = self.explain_selection(strategy, constraints, hist);
            }
        }

        // Normalize confidence to 0.0..=1.0
        let confidence = (best_score / 10.0).clamp(0.0, 1.0);

        StrategySelection {
            strategy: best_strategy,
            confidence,
            reasoning: best_reasoning,
            constraints_applied: constraints.clone(),
        }
    }

    /// Score an individual strategy. Higher is better.
    pub fn score_strategy(
        &self,
        strategy: &ExecutionStrategy,
        constraints: &StrategyConstraints,
        complexity: &ComplexityEstimate,
        history: Option<&HistoricalPerformance>,
    ) -> f64 {
        let mut score = self.base_score(strategy, constraints);

        // Complexity adjustments
        score += self.complexity_bonus(strategy, complexity);

        // Historical performance adjustment
        if let Some(hist) = history {
            score = self.adjust_for_history(score, hist);
        }

        score
    }

    /// Adjust base score using EMA-weighted historical data.
    pub fn adjust_for_history(&self, base_score: f64, history: &HistoricalPerformance) -> f64 {
        if history.sample_count == 0 {
            return base_score;
        }

        // Weight historical data more heavily with more samples (up to 50% influence)
        let confidence_factor = (history.sample_count as f64 / 20.0).min(0.5);
        let historical_score = history.avg_success_rate * 10.0; // normalize to same scale

        base_score * (1.0 - confidence_factor) + historical_score * confidence_factor
    }

    // ── Private helpers ──────────────────────────────────────────

    fn base_score(&self, strategy: &ExecutionStrategy, constraints: &StrategyConstraints) -> f64 {
        match strategy {
            ExecutionStrategy::Fastest => {
                let mut s = 5.0;
                if constraints.deadline_secs.is_some() {
                    s += 3.0; // strong bonus when deadline exists
                }
                s
            }
            ExecutionStrategy::Cheapest => {
                let mut s = 5.0;
                if constraints.budget.is_some() {
                    s += 3.0;
                }
                s
            }
            ExecutionStrategy::HighestQuality => {
                let mut s = 5.0;
                if let Some(q) = constraints.min_quality {
                    if q > 0.8 {
                        s += 3.0;
                    }
                }
                s
            }
            ExecutionStrategy::Balanced => 6.0, // slight baseline advantage
            ExecutionStrategy::ParallelFirst => {
                let mut s = 4.0;
                if constraints.deadline_secs.is_some() {
                    s += 2.0;
                }
                s
            }
            ExecutionStrategy::ReliabilityFirst => {
                let mut s = 5.0;
                if constraints.max_retries < 2 {
                    s += 2.0; // when retries are limited, reliability matters more
                }
                s
            }
        }
    }

    fn complexity_bonus(
        &self,
        strategy: &ExecutionStrategy,
        complexity: &ComplexityEstimate,
    ) -> f64 {
        match (strategy, complexity) {
            // Parallel shines with complex tasks
            (
                ExecutionStrategy::ParallelFirst,
                ComplexityEstimate::Complex | ComplexityEstimate::Epic,
            ) => 2.0,
            // Reliability is more valuable for complex tasks
            (
                ExecutionStrategy::ReliabilityFirst,
                ComplexityEstimate::Complex | ComplexityEstimate::Epic,
            ) => 1.5,
            // Fastest is penalized for epic tasks (risky)
            (ExecutionStrategy::Fastest, ComplexityEstimate::Epic) => -1.0,
            // Cheapest is penalized for complex tasks (false economy)
            (ExecutionStrategy::Cheapest, ComplexityEstimate::Epic) => -1.0,
            // Quality is bonus for moderate+
            (
                ExecutionStrategy::HighestQuality,
                ComplexityEstimate::Moderate
                | ComplexityEstimate::Complex
                | ComplexityEstimate::Epic,
            ) => 1.0,
            _ => 0.0,
        }
    }

    fn explain_selection(
        &self,
        strategy: &ExecutionStrategy,
        constraints: &StrategyConstraints,
        history: Option<&HistoricalPerformance>,
    ) -> String {
        let mut reasons = Vec::new();

        match strategy {
            ExecutionStrategy::Fastest => reasons.push("Optimizing for speed".to_string()),
            ExecutionStrategy::Cheapest => reasons.push("Optimizing for cost".to_string()),
            ExecutionStrategy::HighestQuality => reasons.push("Optimizing for quality".to_string()),
            ExecutionStrategy::Balanced => reasons.push("Balanced trade-offs".to_string()),
            ExecutionStrategy::ParallelFirst => reasons.push("Maximizing parallelism".to_string()),
            ExecutionStrategy::ReliabilityFirst => {
                reasons.push("Prioritizing reliability".to_string())
            }
        }

        if constraints.budget.is_some() {
            reasons.push("Budget constraint active".to_string());
        }
        if constraints.deadline_secs.is_some() {
            reasons.push("Deadline constraint active".to_string());
        }
        if let Some(hist) = history {
            reasons.push(format!(
                "Historical success rate: {:.0}% ({} samples)",
                hist.avg_success_rate * 100.0,
                hist.sample_count
            ));
        }

        reasons.join("; ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_history(
        strategy: ExecutionStrategy,
        success: f64,
        cost: f64,
        duration: f64,
        samples: u32,
    ) -> HistoricalPerformance {
        HistoricalPerformance {
            strategy,
            avg_success_rate: success,
            avg_cost: cost,
            avg_duration_secs: duration,
            sample_count: samples,
            last_updated: Utc::now(),
        }
    }

    #[test]
    fn balanced_wins_with_no_constraints() {
        let engine = StrategyEngine::new();
        let constraints = StrategyConstraints::default();
        let sel = engine.select_strategy(&constraints, &ComplexityEstimate::Simple, &[]);
        assert_eq!(sel.strategy, ExecutionStrategy::Balanced);
    }

    #[test]
    fn fastest_wins_with_tight_deadline() {
        let engine = StrategyEngine::new();
        let constraints = StrategyConstraints {
            deadline_secs: Some(60.0),
            ..Default::default()
        };
        let sel = engine.select_strategy(&constraints, &ComplexityEstimate::Simple, &[]);
        assert_eq!(sel.strategy, ExecutionStrategy::Fastest);
    }

    #[test]
    fn cheapest_wins_with_tight_budget() {
        let engine = StrategyEngine::new();
        let constraints = StrategyConstraints {
            budget: Some(10.0),
            ..Default::default()
        };
        let sel = engine.select_strategy(&constraints, &ComplexityEstimate::Simple, &[]);
        assert_eq!(sel.strategy, ExecutionStrategy::Cheapest);
    }

    #[test]
    fn quality_wins_with_high_quality_requirement() {
        let engine = StrategyEngine::new();
        let constraints = StrategyConstraints {
            min_quality: Some(0.95),
            ..Default::default()
        };
        let sel = engine.select_strategy(&constraints, &ComplexityEstimate::Moderate, &[]);
        assert_eq!(sel.strategy, ExecutionStrategy::HighestQuality);
    }

    #[test]
    fn parallel_boosted_for_complex_with_deadline() {
        let engine = StrategyEngine::new();
        let constraints = StrategyConstraints {
            deadline_secs: Some(30.0),
            ..Default::default()
        };
        let sel = engine.select_strategy(&constraints, &ComplexityEstimate::Complex, &[]);
        // Should be Fastest or ParallelFirst
        assert!(
            sel.strategy == ExecutionStrategy::Fastest
                || sel.strategy == ExecutionStrategy::ParallelFirst
        );
    }

    #[test]
    fn reliability_wins_with_low_retries() {
        let engine = StrategyEngine::new();
        let constraints = StrategyConstraints {
            max_retries: 0,
            ..Default::default()
        };
        let sel = engine.select_strategy(&constraints, &ComplexityEstimate::Complex, &[]);
        assert_eq!(sel.strategy, ExecutionStrategy::ReliabilityFirst);
    }

    #[test]
    fn historical_data_boosts_score() {
        let engine = StrategyEngine::new();
        let base = 5.0;
        let hist = make_history(ExecutionStrategy::Fastest, 0.95, 10.0, 30.0, 20);
        let adjusted = engine.adjust_for_history(base, &hist);
        // Should be influenced towards 9.5 (0.95 * 10)
        assert!(adjusted > base);
    }

    #[test]
    fn zero_samples_no_adjustment() {
        let engine = StrategyEngine::new();
        let base = 5.0;
        let hist = make_history(ExecutionStrategy::Fastest, 0.0, 0.0, 0.0, 0);
        let adjusted = engine.adjust_for_history(base, &hist);
        assert!((adjusted - base).abs() < f64::EPSILON);
    }

    #[test]
    fn historical_data_can_shift_selection() {
        let engine = StrategyEngine::new();
        let constraints = StrategyConstraints::default();

        // Give "Cheapest" stellar historical performance
        let history = vec![make_history(
            ExecutionStrategy::Cheapest,
            0.99,
            5.0,
            60.0,
            50,
        )];

        let sel = engine.select_strategy(&constraints, &ComplexityEstimate::Simple, &history);
        // With such strong history, cheapest might beat balanced
        assert!(sel.confidence > 0.0);
    }

    #[test]
    fn confidence_is_bounded() {
        let engine = StrategyEngine::new();
        let constraints = StrategyConstraints::default();
        let sel = engine.select_strategy(&constraints, &ComplexityEstimate::Trivial, &[]);
        assert!(sel.confidence >= 0.0);
        assert!(sel.confidence <= 1.0);
    }

    #[test]
    fn reasoning_is_non_empty() {
        let engine = StrategyEngine::new();
        let constraints = StrategyConstraints::default();
        let sel = engine.select_strategy(&constraints, &ComplexityEstimate::Moderate, &[]);
        assert!(!sel.reasoning.is_empty());
    }

    #[test]
    fn reasoning_mentions_budget_when_set() {
        let engine = StrategyEngine::new();
        let constraints = StrategyConstraints {
            budget: Some(50.0),
            ..Default::default()
        };
        let sel = engine.select_strategy(&constraints, &ComplexityEstimate::Simple, &[]);
        assert!(sel.reasoning.contains("Budget"));
    }

    #[test]
    fn reasoning_mentions_deadline_when_set() {
        let engine = StrategyEngine::new();
        let constraints = StrategyConstraints {
            deadline_secs: Some(100.0),
            ..Default::default()
        };
        let sel = engine.select_strategy(&constraints, &ComplexityEstimate::Simple, &[]);
        assert!(sel.reasoning.contains("Deadline"));
    }

    #[test]
    fn fastest_penalized_for_epic() {
        let engine = StrategyEngine::new();
        let bonus_simple =
            engine.complexity_bonus(&ExecutionStrategy::Fastest, &ComplexityEstimate::Simple);
        let bonus_epic =
            engine.complexity_bonus(&ExecutionStrategy::Fastest, &ComplexityEstimate::Epic);
        assert!(bonus_epic < bonus_simple);
    }

    #[test]
    fn score_strategy_all_positive_defaults() {
        let engine = StrategyEngine::new();
        let constraints = StrategyConstraints::default();
        for s in ExecutionStrategy::all() {
            let score = engine.score_strategy(s, &constraints, &ComplexityEstimate::Moderate, None);
            assert!(score > 0.0, "Strategy {:?} had non-positive score", s);
        }
    }

    #[test]
    fn default_trait() {
        let engine = StrategyEngine::new();
        let sel = engine.select_strategy(
            &StrategyConstraints::default(),
            &ComplexityEstimate::Moderate,
            &[],
        );
        assert!(!sel.reasoning.is_empty());
    }

    #[test]
    fn constraints_serialization() {
        let c = StrategyConstraints {
            budget: Some(100.0),
            deadline_secs: Some(3600.0),
            min_quality: Some(0.9),
            max_retries: 5,
        };
        let json = serde_json::to_string(&c).unwrap();
        let back: StrategyConstraints = serde_json::from_str(&json).unwrap();
        assert_eq!(back.max_retries, 5);
        assert!((back.budget.unwrap() - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn historical_reasoning_included() {
        let engine = StrategyEngine::new();
        let constraints = StrategyConstraints::default();
        let history = vec![make_history(
            ExecutionStrategy::Balanced,
            0.88,
            20.0,
            120.0,
            15,
        )];
        let sel = engine.select_strategy(&constraints, &ComplexityEstimate::Moderate, &history);
        assert!(sel.reasoning.contains("Historical") || sel.reasoning.contains("success"));
    }

    #[test]
    fn parallel_first_bonus_for_complex() {
        let engine = StrategyEngine::new();
        let bonus = engine.complexity_bonus(
            &ExecutionStrategy::ParallelFirst,
            &ComplexityEstimate::Complex,
        );
        assert!(bonus > 0.0);
    }

    #[test]
    fn all_strategies_scored_for_epic() {
        let engine = StrategyEngine::new();
        let constraints = StrategyConstraints::default();
        let sel = engine.select_strategy(&constraints, &ComplexityEstimate::Epic, &[]);
        // Just verify it doesn't panic and returns a valid strategy
        assert!(ExecutionStrategy::all().contains(&sel.strategy));
    }
}
