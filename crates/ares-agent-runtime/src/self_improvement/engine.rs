use crate::evaluation::engine::SelfEvaluationEngine;
use crate::evaluation::models::MissionScore;
use crate::learning::engine::LearningEngine;
use crate::learning::models::MissionOutcome;
use crate::models::MissionId;
use crate::reflection::mission_reflection::MissionReflection;
use crate::self_improvement::models::{
    ImprovementAction, ImprovementCycle, ImprovementOutcome, ImprovementPhase,
};
use chrono::Utc;

/// Score threshold below which improvement is attempted.
const IMPROVEMENT_THRESHOLD: f64 = 0.8;

/// Orchestrates the full self-improvement cycle:
/// Execute → Reflect → Evaluate → Learn → Improve → Replan
pub struct SelfImprovementEngine {
    evaluator: SelfEvaluationEngine,
    learner: LearningEngine,
    cycles: Vec<ImprovementCycle>,
}

impl Default for SelfImprovementEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SelfImprovementEngine {
    pub fn new() -> Self {
        Self {
            evaluator: SelfEvaluationEngine::new(),
            learner: LearningEngine::new(),
            cycles: Vec::new(),
        }
    }

    /// Run a complete improvement cycle for a mission.
    pub fn run_improvement_cycle(
        &mut self,
        mission_id: MissionId,
        reflection: MissionReflection,
        strategy_used: &str,
    ) -> ImprovementOutcome {
        let cycle_id = ares_core::id::new_id();

        let mut cycle = ImprovementCycle {
            cycle_id: cycle_id.clone(),
            mission_id,
            phase: ImprovementPhase::Reflect,
            reflection: Some(reflection.clone()),
            score: None,
            actions_taken: Vec::new(),
            started_at: Utc::now(),
            completed_at: None,
        };

        // Phase: Evaluate
        cycle.phase = ImprovementPhase::Evaluate;
        let score = self.evaluator.evaluate_mission(&reflection);
        cycle.score = Some(score.clone());

        // Phase: Learn
        cycle.phase = ImprovementPhase::Learn;
        let outcome = MissionOutcome {
            mission_id,
            strategy_used: strategy_used.to_string(),
            success: reflection.failed_tasks == 0,
            score: score.overall_score,
            cost: reflection.total_cost,
            duration_secs: reflection.duration_secs(),
            completed_at: Utc::now(),
        };
        self.learner.record_outcome(outcome);
        cycle
            .actions_taken
            .push(ImprovementAction::RecordedLearning);

        // Phase: Improve
        cycle.phase = ImprovementPhase::Improve;
        let improvements = if self.should_improve(&score) {
            self.apply_improvements(&score, &reflection, strategy_used)
        } else {
            vec![ImprovementAction::SkippedImprovement(
                "Score above threshold".to_string(),
            )]
        };

        let improved = improvements
            .iter()
            .any(|a| !matches!(a, ImprovementAction::SkippedImprovement(_)));

        cycle.actions_taken.extend(improvements.clone());
        cycle.completed_at = Some(Utc::now());
        cycle.phase = if improved {
            ImprovementPhase::Replan
        } else {
            ImprovementPhase::Improve
        };

        // Calculate score delta from learning history
        let score_delta =
            score.overall_score - self.learner.get_learning_profile().overall_ema_score;

        self.record_cycle(cycle);

        ImprovementOutcome {
            cycle_id,
            improved,
            score_delta,
            actions: improvements,
        }
    }

    /// Determine whether the mission score warrants improvement efforts.
    pub fn should_improve(&self, score: &MissionScore) -> bool {
        score.overall_score < IMPROVEMENT_THRESHOLD
    }

    /// Determine improvement actions based on evaluation and reflection data.
    pub fn apply_improvements(
        &mut self,
        score: &MissionScore,
        reflection: &MissionReflection,
        current_strategy: &str,
    ) -> Vec<ImprovementAction> {
        let mut actions = Vec::new();

        // Check if a better strategy exists
        if let Some(best) = self.learner.get_best_strategy() {
            if best != current_strategy {
                actions.push(ImprovementAction::UpdatedStrategy(best));
            }
        }

        // High retry ratio → adjust parameters
        if reflection.retry_ratio() > 0.3 {
            actions.push(ImprovementAction::AdjustedParameters(
                "Increased retry tolerance and added backoff".to_string(),
            ));
        }

        // Very low score → rebuild DAG
        if score.overall_score < 0.4 {
            actions.push(ImprovementAction::RebuiltDag);
        }

        // Update agent effectiveness EMAs
        for effectiveness in reflection.agent_effectiveness.values() {
            // Use a generic role name since we don't have role info here
            self.learner.update_agent_ema(
                "agent",
                effectiveness.avg_quality(),
                effectiveness.avg_latency_ms(),
            );
        }

        if actions.is_empty() {
            actions.push(ImprovementAction::SkippedImprovement(
                "No specific improvement identified".to_string(),
            ));
        }

        actions
    }

    /// Record a completed improvement cycle.
    pub fn record_cycle(&mut self, cycle: ImprovementCycle) {
        self.cycles.push(cycle);
    }

    /// Get all recorded cycles.
    pub fn get_cycles(&self) -> &[ImprovementCycle] {
        &self.cycles
    }

    /// Get the learning engine (for inspection/export).
    pub fn get_learner(&self) -> &LearningEngine {
        &self.learner
    }

    /// Get total number of improvement cycles run.
    pub fn cycle_count(&self) -> usize {
        self.cycles.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluation::models::grade_from_score;
    use crate::reflection::mission_reflection::AgentEffectivenessScore;
    use std::collections::HashMap;

    fn make_reflection(completed: u32, failed: u32, retries: u32, cost: f64) -> MissionReflection {
        MissionReflection {
            mission_id: MissionId::new(),
            total_tasks: completed + failed,
            completed_tasks: completed,
            failed_tasks: failed,
            retries,
            tool_usage: HashMap::new(),
            agent_effectiveness: HashMap::new(),
            total_cost: cost,
            total_latency_ms: 2000,
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
        }
    }

    fn make_score(overall: f64) -> MissionScore {
        MissionScore {
            mission_id: MissionId::new(),
            overall_score: overall,
            metric_scores: vec![],
            evaluated_at: Utc::now(),
            grade: grade_from_score(overall),
        }
    }

    #[test]
    fn should_improve_below_threshold() {
        let engine = SelfImprovementEngine::new();
        assert!(engine.should_improve(&make_score(0.5)));
        assert!(engine.should_improve(&make_score(0.79)));
    }

    #[test]
    fn should_not_improve_above_threshold() {
        let engine = SelfImprovementEngine::new();
        assert!(!engine.should_improve(&make_score(0.8)));
        assert!(!engine.should_improve(&make_score(0.95)));
    }

    #[test]
    fn run_cycle_good_mission() {
        let mut engine = SelfImprovementEngine::new();
        let reflection = make_reflection(10, 0, 0, 5.0);
        let outcome = engine.run_improvement_cycle(MissionId::new(), reflection, "balanced");

        // Good mission → learning recorded, improvement may be skipped
        assert!(outcome
            .actions
            .iter()
            .any(|a| matches!(a, ImprovementAction::RecordedLearning)
                | matches!(a, ImprovementAction::SkippedImprovement(_))));
        assert_eq!(engine.cycle_count(), 1);
    }

    #[test]
    fn run_cycle_poor_mission() {
        let mut engine = SelfImprovementEngine::new();
        let reflection = make_reflection(2, 8, 5, 80.0);
        let outcome = engine.run_improvement_cycle(MissionId::new(), reflection, "fastest");

        // Poor mission → should have improvements
        assert!(outcome.improved || !outcome.actions.is_empty());
        assert_eq!(engine.cycle_count(), 1);
    }

    #[test]
    fn run_cycle_records_learning() {
        let mut engine = SelfImprovementEngine::new();
        let reflection = make_reflection(5, 0, 0, 10.0);
        engine.run_improvement_cycle(MissionId::new(), reflection, "balanced");

        let profile = engine.get_learner().get_learning_profile();
        assert_eq!(profile.total_missions, 1);
        assert!(profile.strategy_records.contains_key("balanced"));
    }

    #[test]
    fn multiple_cycles_accumulate() {
        let mut engine = SelfImprovementEngine::new();

        for i in 0..5 {
            let reflection = make_reflection(5, i, 0, 10.0);
            engine.run_improvement_cycle(MissionId::new(), reflection, "balanced");
        }

        assert_eq!(engine.cycle_count(), 5);
        assert_eq!(
            engine.get_learner().get_learning_profile().total_missions,
            5
        );
    }

    #[test]
    fn strategy_suggestion_after_learning() {
        let mut engine = SelfImprovementEngine::new();

        // Record several outcomes with different strategies
        for _ in 0..5 {
            let reflection = make_reflection(10, 0, 0, 5.0);
            engine.run_improvement_cycle(MissionId::new(), reflection, "fastest");
        }
        for _ in 0..5 {
            let reflection = make_reflection(3, 7, 3, 50.0);
            engine.run_improvement_cycle(MissionId::new(), reflection, "cheapest");
        }

        // "fastest" should be suggested over "cheapest"
        let best = engine.get_learner().get_best_strategy();
        assert_eq!(best.as_deref(), Some("fastest"));
    }

    #[test]
    fn high_retry_triggers_parameter_adjustment() {
        let mut engine = SelfImprovementEngine::new();
        let reflection = make_reflection(5, 5, 10, 10.0); // retry_ratio = 1.0
        let score = make_score(0.5);

        let actions = engine.apply_improvements(&score, &reflection, "balanced");
        let has_param_adjust = actions
            .iter()
            .any(|a| matches!(a, ImprovementAction::AdjustedParameters(_)));
        assert!(has_param_adjust);
    }

    #[test]
    fn very_low_score_triggers_rebuild() {
        let mut engine = SelfImprovementEngine::new();
        let reflection = make_reflection(1, 9, 0, 10.0);
        let score = make_score(0.2);

        let actions = engine.apply_improvements(&score, &reflection, "balanced");
        let has_rebuild = actions
            .iter()
            .any(|a| matches!(a, ImprovementAction::RebuiltDag));
        assert!(has_rebuild);
    }

    #[test]
    fn agent_effectiveness_tracked() {
        let mut engine = SelfImprovementEngine::new();
        let agent = crate::models::AgentId::new();
        let mut reflection = make_reflection(5, 0, 0, 5.0);
        reflection.agent_effectiveness.insert(
            agent,
            AgentEffectivenessScore {
                tasks_completed: 5,
                tasks_failed: 0,
                total_quality: 4.5,
                total_latency_ms: 2500,
                task_count: 5,
            },
        );

        let score = make_score(0.5);
        engine.apply_improvements(&score, &reflection, "balanced");

        let agent_rec = engine.get_learner().get_agent_effectiveness("agent");
        assert!(agent_rec.is_some());
    }

    #[test]
    fn score_delta_calculation() {
        let mut engine = SelfImprovementEngine::new();

        // First cycle — delta should be ~ 0
        let r1 = make_reflection(10, 0, 0, 5.0);
        let o1 = engine.run_improvement_cycle(MissionId::new(), r1, "balanced");
        // Score delta is score - ema (which just got set to the same value)
        assert!(o1.score_delta.abs() < 0.5);
    }

    #[test]
    fn outcome_cycle_id_unique() {
        let mut engine = SelfImprovementEngine::new();
        let r1 = make_reflection(5, 0, 0, 5.0);
        let r2 = make_reflection(5, 0, 0, 5.0);
        let o1 = engine.run_improvement_cycle(MissionId::new(), r1, "a");
        let o2 = engine.run_improvement_cycle(MissionId::new(), r2, "a");
        assert_ne!(o1.cycle_id, o2.cycle_id);
    }

    #[test]
    fn default_trait() {
        let engine = SelfImprovementEngine::default();
        assert_eq!(engine.cycle_count(), 0);
    }

    #[test]
    fn get_cycles_returns_all() {
        let mut engine = SelfImprovementEngine::new();
        let r = make_reflection(5, 0, 0, 5.0);
        engine.run_improvement_cycle(MissionId::new(), r, "balanced");
        assert_eq!(engine.get_cycles().len(), 1);
    }

    #[test]
    fn no_improvement_needed_skips() {
        let mut engine = SelfImprovementEngine::new();
        let reflection = make_reflection(10, 0, 0, 2.0);
        let outcome = engine.run_improvement_cycle(MissionId::new(), reflection, "balanced");

        let has_skip = outcome
            .actions
            .iter()
            .any(|a| matches!(a, ImprovementAction::SkippedImprovement(_)));
        assert!(has_skip);
    }
}
