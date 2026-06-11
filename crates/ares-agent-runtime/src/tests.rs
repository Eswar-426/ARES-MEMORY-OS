use super::*;
use agents::*;
use evaluation::engine::SelfEvaluationEngine;
use evaluation::models::{grade_from_score, MissionGrade};
use learning::engine::LearningEngine;
use learning::models::MissionOutcome;
use models::*;
use reflection::mission_reflection::{
    AgentEffectivenessScore, MissionReflection, MissionReflector, ToolUsageStats,
};
use replanning::autonomous::AutonomousReplanner;
use replanning::*;
use resources::*;
use scheduler::*;
use self_improvement::engine::SelfImprovementEngine;
use workflow::*;

use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════
// Core Runtime Tests (preserved & enhanced from Week 15)
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_mission_lifecycle() {
    let mut dag = MissionDag::new();
    let node = MissionNode {
        id: TaskId::new(),
        name: "test".into(),
        role: AgentRole::Architect,
        payload: "test".into(),
    };
    dag.add_node(node);

    let manager = mission::MissionManager::new();
    let mission_id = manager.create_mission(dag).await;

    assert_eq!(
        manager.track_mission(&mission_id).await.unwrap(),
        MissionState::Pending
    );

    manager.resume_mission(&mission_id).await.unwrap();
    assert_eq!(
        manager.track_mission(&mission_id).await.unwrap(),
        MissionState::Executing
    );

    manager.pause_mission(&mission_id).await.unwrap();
    assert_eq!(
        manager.track_mission(&mission_id).await.unwrap(),
        MissionState::Waiting
    );
}

#[test]
fn test_dag_execution() {
    let mut dag = MissionDag::new();
    let id1 = TaskId::new();
    let id2 = TaskId::new();
    dag.add_node(MissionNode {
        id: id1,
        name: "1".into(),
        role: AgentRole::Architect,
        payload: "".into(),
    });
    dag.add_node(MissionNode {
        id: id2,
        name: "2".into(),
        role: AgentRole::Coder,
        payload: "".into(),
    });
    dag.add_edge(MissionEdge {
        from: id1,
        to: id2,
        condition: None,
    });

    let roots = dag.get_roots();
    assert_eq!(roots.len(), 1);
    assert_eq!(roots[0], id1);

    let mut executor = MissionExecutor::new(dag);
    let ready = executor.get_ready_nodes();
    assert_eq!(ready.len(), 1);
    assert_eq!(ready[0].id, id1);

    executor.mark_completed(id1);
    let ready2 = executor.get_ready_nodes();
    assert_eq!(ready2.len(), 1);
    assert_eq!(ready2[0].id, id2);
}

#[test]
fn test_agent_allocation() {
    let mut registry = AgentRegistry::new();
    registry.register_template(AgentConfig {
        role: AgentRole::Coder,
        capabilities: CapabilitySet::default(),
        provider: ProviderSelection::Local("test".into()),
        system_prompt: "test".into(),
        temperature: 0.0,
    });

    let id = registry.allocate_agent(&AgentRole::Coder).unwrap();
    assert!(registry.health_check(&id));
    registry.release_agent(&id).unwrap();
    assert!(!registry.health_check(&id));
}

#[test]
fn test_scheduler() {
    let mut scheduler = AgentScheduler::new(SchedulingStrategy::LeastLoaded);
    let id1 = AgentId::new();
    let id2 = AgentId::new();
    scheduler.update_agent_load(id1, 5);
    scheduler.update_agent_load(id2, 2);

    let selected = scheduler.select_agent(&[id1, id2]).unwrap();
    assert_eq!(selected, id2);
}

#[test]
fn test_resource_budget() {
    let manager = ResourceManager::new(Budgets {
        cpu_cores: 4,
        memory_mb: 1024,
        tokens: 1000,
        api_calls: 10,
        concurrency: 2,
    });

    assert!(manager.acquire_concurrency().is_ok());
    assert!(manager.acquire_concurrency().is_ok());
    assert!(manager.acquire_concurrency().is_err());

    manager.release_concurrency();
    assert!(manager.acquire_concurrency().is_ok());
}

// ═══════════════════════════════════════════════════════════════════
// Mission Reflection Tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_reflector_lifecycle() {
    let mut reflector = MissionReflector::new();
    let mid = MissionId::new();
    reflector.start_mission(mid, 5);
    reflector.track_agent_result(&mid, AgentId::new(), true, 0.9, 100);
    reflector.track_agent_result(&mid, AgentId::new(), false, 0.2, 500);
    reflector.track_tool_usage(&mid, "grep", true, 50);
    reflector.track_retry(&mid);
    reflector.track_cost(&mid, 15.0);

    let reflection = reflector.finalize(&mid).unwrap();
    assert_eq!(reflection.completed_tasks, 1);
    assert_eq!(reflection.failed_tasks, 1);
    assert_eq!(reflection.retries, 1);
    assert!((reflection.total_cost - 15.0).abs() < f64::EPSILON);
}

#[test]
fn test_reflection_report_generation() {
    let reflector = MissionReflector::new();
    let reflection = MissionReflection {
        mission_id: MissionId::new(),
        total_tasks: 10,
        completed_tasks: 10,
        failed_tasks: 0,
        retries: 0,
        tool_usage: HashMap::new(),
        agent_effectiveness: HashMap::new(),
        total_cost: 5.0,
        total_latency_ms: 2000,
        started_at: chrono::Utc::now(),
        completed_at: Some(chrono::Utc::now()),
    };

    let report = reflector.reflect_on_mission(&reflection);
    assert!(report.success);
    assert_eq!(report.quality_score, 100);
}

#[test]
fn test_tool_usage_stats() {
    let stats = ToolUsageStats {
        invocations: 10,
        successes: 7,
        failures: 3,
        total_latency_ms: 5000,
    };
    assert!((stats.success_rate() - 0.7).abs() < f64::EPSILON);
    assert!((stats.avg_latency_ms() - 500.0).abs() < f64::EPSILON);
}

#[test]
fn test_agent_effectiveness_score() {
    let score = AgentEffectivenessScore {
        tasks_completed: 8,
        tasks_failed: 2,
        total_quality: 7.2,
        total_latency_ms: 10000,
        task_count: 10,
    };
    assert!((score.success_rate() - 0.8).abs() < f64::EPSILON);
    assert!((score.avg_quality() - 0.72).abs() < f64::EPSILON);
    assert!((score.avg_latency_ms() - 1000.0).abs() < f64::EPSILON);
}

// ═══════════════════════════════════════════════════════════════════
// Evaluation Engine Tests
// ═══════════════════════════════════════════════════════════════════

fn make_test_reflection(completed: u32, failed: u32, retries: u32, cost: f64) -> MissionReflection {
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
        started_at: chrono::Utc::now(),
        completed_at: Some(chrono::Utc::now()),
    }
}

#[test]
fn test_evaluation_perfect_score() {
    let engine = SelfEvaluationEngine::new();
    let r = make_test_reflection(10, 0, 0, 5.0);
    let score = engine.evaluate_mission(&r);
    assert!(score.overall_score > 0.85);
    assert!(score.grade >= MissionGrade::Good);
}

#[test]
fn test_evaluation_failed_mission() {
    let engine = SelfEvaluationEngine::new();
    let r = make_test_reflection(0, 10, 5, 100.0);
    let score = engine.evaluate_mission(&r);
    assert!(score.overall_score < 0.4);
    assert!(score.grade <= MissionGrade::Poor);
}

#[test]
fn test_evaluation_all_metrics_present() {
    let engine = SelfEvaluationEngine::new();
    let r = make_test_reflection(5, 3, 2, 20.0);
    let score = engine.evaluate_mission(&r);
    assert_eq!(score.metric_scores.len(), 6);
}

#[test]
fn test_grade_thresholds() {
    assert_eq!(grade_from_score(0.95), MissionGrade::Excellent);
    assert_eq!(grade_from_score(0.80), MissionGrade::Good);
    assert_eq!(grade_from_score(0.65), MissionGrade::Acceptable);
    assert_eq!(grade_from_score(0.45), MissionGrade::Poor);
    assert_eq!(grade_from_score(0.20), MissionGrade::Failed);
}

#[test]
fn test_higher_completion_higher_score() {
    let engine = SelfEvaluationEngine::new();
    let low = make_test_reflection(3, 7, 0, 10.0);
    let high = make_test_reflection(9, 1, 0, 10.0);
    assert!(
        engine.evaluate_mission(&high).overall_score > engine.evaluate_mission(&low).overall_score
    );
}

// ═══════════════════════════════════════════════════════════════════
// Learning Engine Tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_learning_first_outcome() {
    let mut engine = LearningEngine::new();
    let outcome = MissionOutcome {
        mission_id: MissionId::new(),
        strategy_used: "balanced".to_string(),
        success: true,
        score: 0.9,
        cost: 10.0,
        duration_secs: 60.0,
        completed_at: chrono::Utc::now(),
    };
    engine.record_outcome(outcome);

    let perf = engine.get_strategy_performance("balanced").unwrap();
    assert_eq!(perf.sample_count, 1);
    assert!((perf.ema_success_rate - 1.0).abs() < f64::EPSILON);
}

#[test]
fn test_learning_ema_convergence() {
    let mut engine = LearningEngine::new();
    for _ in 0..20 {
        engine.record_outcome(MissionOutcome {
            mission_id: MissionId::new(),
            strategy_used: "test".to_string(),
            success: false,
            score: 0.1,
            cost: 50.0,
            duration_secs: 300.0,
            completed_at: chrono::Utc::now(),
        });
    }
    let perf = engine.get_strategy_performance("test").unwrap();
    assert!(perf.ema_success_rate < 0.1);
}

#[test]
fn test_learning_best_strategy() {
    let mut engine = LearningEngine::new();
    for _ in 0..5 {
        engine.record_outcome(MissionOutcome {
            mission_id: MissionId::new(),
            strategy_used: "good".to_string(),
            success: true,
            score: 0.9,
            cost: 10.0,
            duration_secs: 30.0,
            completed_at: chrono::Utc::now(),
        });
    }
    for _ in 0..5 {
        engine.record_outcome(MissionOutcome {
            mission_id: MissionId::new(),
            strategy_used: "bad".to_string(),
            success: false,
            score: 0.2,
            cost: 80.0,
            duration_secs: 300.0,
            completed_at: chrono::Utc::now(),
        });
    }
    assert_eq!(engine.get_best_strategy().as_deref(), Some("good"));
}

#[test]
fn test_learning_agent_ema() {
    let mut engine = LearningEngine::new();
    engine.update_agent_ema("Coder", 0.9, 100.0);
    engine.update_agent_ema("Coder", 0.7, 200.0);
    let rec = engine.get_agent_effectiveness("Coder").unwrap();
    assert_eq!(rec.task_count, 2);
    assert!(rec.ema_quality > 0.7 && rec.ema_quality < 0.9);
}

#[test]
fn test_learning_export_performance() {
    let mut engine = LearningEngine::new();
    engine.record_outcome(MissionOutcome {
        mission_id: MissionId::new(),
        strategy_used: "fastest".to_string(),
        success: true,
        score: 0.8,
        cost: 15.0,
        duration_secs: 45.0,
        completed_at: chrono::Utc::now(),
    });
    let exports = engine.export_historical_performance();
    assert_eq!(exports.len(), 1);
    assert_eq!(exports[0].strategy, "fastest");
}

// ═══════════════════════════════════════════════════════════════════
// Autonomous Replanning Tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_replanning_threshold() {
    let rp = AutonomousReplanner::new();
    let low_score = evaluation::models::MissionScore {
        mission_id: MissionId::new(),
        overall_score: 0.3,
        metric_scores: vec![],
        evaluated_at: chrono::Utc::now(),
        grade: MissionGrade::Failed,
    };
    assert!(rp.should_replan(&low_score));

    let high_score = evaluation::models::MissionScore {
        mission_id: MissionId::new(),
        overall_score: 0.9,
        metric_scores: vec![],
        evaluated_at: chrono::Utc::now(),
        grade: MissionGrade::Excellent,
    };
    assert!(!rp.should_replan(&high_score));
}

#[test]
fn test_replanning_poor_agent_replacement() {
    let rp = AutonomousReplanner::new();
    let dag = MissionDag::new();
    let score = evaluation::models::MissionScore {
        mission_id: MissionId::new(),
        overall_score: 0.3,
        metric_scores: vec![],
        evaluated_at: chrono::Utc::now(),
        grade: MissionGrade::Failed,
    };
    let agent = AgentId::new();
    let mut reflection = make_test_reflection(2, 8, 0, 10.0);
    reflection.agent_effectiveness.insert(
        agent,
        AgentEffectivenessScore {
            tasks_completed: 1,
            tasks_failed: 9,
            total_quality: 1.0,
            total_latency_ms: 10000,
            task_count: 10,
        },
    );

    let actions = rp
        .evaluate_and_replan(MissionId::new(), &dag, &score, &reflection)
        .unwrap();
    assert!(actions
        .iter()
        .any(|a| matches!(a, ReplanningAction::ReplaceAgent(_, _))));
}

#[test]
fn test_replanning_task_split() {
    let rp = AutonomousReplanner::new();
    let dag = MissionDag::new();
    let reflection = make_test_reflection(3, 7, 5, 10.0);
    let trigger = ReplanningTrigger::TaskFailure(TaskId::new());

    let actions = rp.determine_actions(trigger, &dag, &reflection);
    assert!(actions
        .iter()
        .any(|a| matches!(a, ReplanningAction::SplitTask(_, _))));
}

// ═══════════════════════════════════════════════════════════════════
// Self Improvement Loop Tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_improvement_cycle_good_mission() {
    let mut engine = SelfImprovementEngine::new();
    let reflection = make_test_reflection(10, 0, 0, 5.0);
    let outcome = engine.run_improvement_cycle(MissionId::new(), reflection, "balanced");

    assert_eq!(engine.cycle_count(), 1);
    // Good mission may or may not need improvement
    assert!(!outcome.actions.is_empty());
}

#[test]
fn test_improvement_cycle_poor_mission() {
    let mut engine = SelfImprovementEngine::new();
    let reflection = make_test_reflection(2, 8, 5, 80.0);
    let outcome = engine.run_improvement_cycle(MissionId::new(), reflection, "cheapest");

    assert!(outcome.improved || !outcome.actions.is_empty());
}

#[test]
fn test_improvement_multiple_cycles() {
    let mut engine = SelfImprovementEngine::new();
    for i in 0..10 {
        let reflection = make_test_reflection(5 + i, i, 0, 10.0);
        engine.run_improvement_cycle(MissionId::new(), reflection, "balanced");
    }
    assert_eq!(engine.cycle_count(), 10);
    assert_eq!(
        engine.get_learner().get_learning_profile().total_missions,
        10
    );
}

#[test]
fn test_improvement_strategy_suggestion() {
    let mut engine = SelfImprovementEngine::new();
    // Record good outcomes for "fastest"
    for _ in 0..5 {
        let r = make_test_reflection(10, 0, 0, 5.0);
        engine.run_improvement_cycle(MissionId::new(), r, "fastest");
    }
    // Record bad outcomes for "cheapest"
    for _ in 0..5 {
        let r = make_test_reflection(2, 8, 3, 50.0);
        engine.run_improvement_cycle(MissionId::new(), r, "cheapest");
    }
    assert_eq!(
        engine.get_learner().get_best_strategy().as_deref(),
        Some("fastest")
    );
}

// ═══════════════════════════════════════════════════════════════════
// ID Bridge Tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_id_bridge_mission_roundtrip() {
    let original = MissionId::new();
    let core_str = id_bridge::mission_id_to_core(&original);
    let back = id_bridge::core_to_mission_id(&core_str).unwrap();
    assert_eq!(original, back);
}

#[test]
fn test_id_bridge_task_roundtrip() {
    let original = TaskId::new();
    let core_str = id_bridge::task_id_to_core(&original);
    let back = id_bridge::core_to_task_id(&core_str).unwrap();
    assert_eq!(original, back);
}

#[test]
fn test_id_bridge_agent_roundtrip() {
    let original = AgentId::new();
    let core_str = id_bridge::agent_id_to_core(&original);
    let back = id_bridge::core_to_agent_id(&core_str).unwrap();
    assert_eq!(original, back);
}

#[test]
fn test_id_bridge_invalid_returns_none() {
    assert!(id_bridge::core_to_mission_id("not-a-uuid").is_none());
    assert!(id_bridge::core_to_task_id("").is_none());
}

// ═══════════════════════════════════════════════════════════════════
// DAG & Workflow Tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_dag_parallel_nodes() {
    let mut dag = MissionDag::new();
    let root = TaskId::new();
    let a = TaskId::new();
    let b = TaskId::new();

    dag.add_node(MissionNode {
        id: root,
        name: "root".into(),
        role: AgentRole::Architect,
        payload: "".into(),
    });
    dag.add_node(MissionNode {
        id: a,
        name: "A".into(),
        role: AgentRole::Coder,
        payload: "".into(),
    });
    dag.add_node(MissionNode {
        id: b,
        name: "B".into(),
        role: AgentRole::Coder,
        payload: "".into(),
    });
    dag.add_edge(MissionEdge {
        from: root,
        to: a,
        condition: None,
    });
    dag.add_edge(MissionEdge {
        from: root,
        to: b,
        condition: None,
    });

    let mut executor = MissionExecutor::new(dag);
    let ready = executor.get_ready_nodes();
    assert_eq!(ready.len(), 1); // only root

    executor.mark_completed(root);
    let ready2 = executor.get_ready_nodes();
    assert_eq!(ready2.len(), 2); // A and B in parallel
}

#[test]
fn test_dag_diamond_pattern() {
    let mut dag = MissionDag::new();
    let a = TaskId::new();
    let b = TaskId::new();
    let c = TaskId::new();
    let d = TaskId::new();

    dag.add_node(MissionNode {
        id: a,
        name: "A".into(),
        role: AgentRole::Architect,
        payload: "".into(),
    });
    dag.add_node(MissionNode {
        id: b,
        name: "B".into(),
        role: AgentRole::Coder,
        payload: "".into(),
    });
    dag.add_node(MissionNode {
        id: c,
        name: "C".into(),
        role: AgentRole::Coder,
        payload: "".into(),
    });
    dag.add_node(MissionNode {
        id: d,
        name: "D".into(),
        role: AgentRole::Tester,
        payload: "".into(),
    });
    dag.add_edge(MissionEdge {
        from: a,
        to: b,
        condition: None,
    });
    dag.add_edge(MissionEdge {
        from: a,
        to: c,
        condition: None,
    });
    dag.add_edge(MissionEdge {
        from: b,
        to: d,
        condition: None,
    });
    dag.add_edge(MissionEdge {
        from: c,
        to: d,
        condition: None,
    });

    let mut executor = MissionExecutor::new(dag);
    executor.mark_completed(a);
    let ready = executor.get_ready_nodes();
    assert_eq!(ready.len(), 2);

    executor.mark_completed(b);
    let ready2 = executor.get_ready_nodes();
    assert_eq!(ready2.len(), 1); // C is still ready, D is waiting for C

    executor.mark_completed(c);
    let ready3 = executor.get_ready_nodes();
    assert_eq!(ready3.len(), 1); // D now ready
    assert_eq!(ready3[0].id, d);
}

// ═══════════════════════════════════════════════════════════════════
// Replanning Integration Tests
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_replanner_task_failure() {
    let rp = Replanner::new();
    let dag = MissionDag::new();
    let task_id = TaskId::new();
    let action = rp
        .handle_trigger(
            MissionId::new(),
            &dag,
            ReplanningTrigger::TaskFailure(task_id),
        )
        .await
        .unwrap();
    assert!(matches!(action, ReplanningAction::RetryStrategy(_)));
}

#[tokio::test]
async fn test_replanner_budget_exhaustion() {
    let rp = Replanner::new();
    let dag = MissionDag::new();
    let action = rp
        .handle_trigger(MissionId::new(), &dag, ReplanningTrigger::BudgetExhaustion)
        .await
        .unwrap();
    assert!(matches!(action, ReplanningAction::Escalate(_)));
}

#[tokio::test]
async fn test_replanner_timeout() {
    let rp = Replanner::new();
    let dag = MissionDag::new();
    let action = rp
        .handle_trigger(MissionId::new(), &dag, ReplanningTrigger::Timeout)
        .await
        .unwrap();
    assert!(matches!(action, ReplanningAction::Escalate(_)));
}

#[tokio::test]
async fn test_replanner_quality_below_threshold() {
    let rp = Replanner::new();
    let dag = MissionDag::new();
    let action = rp
        .handle_trigger(
            MissionId::new(),
            &dag,
            ReplanningTrigger::QualityBelowThreshold(40),
        )
        .await
        .unwrap();
    assert!(matches!(action, ReplanningAction::RebuildDag(_)));
}

// ═══════════════════════════════════════════════════════════════════
// Resource Management Tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_resource_token_budget() {
    let manager = ResourceManager::new(Budgets {
        cpu_cores: 4,
        memory_mb: 1024,
        tokens: 100,
        api_calls: 10,
        concurrency: 2,
    });
    assert!(manager.consume_tokens(50).is_ok());
    assert!(manager.consume_tokens(50).is_ok());
    assert!(manager.consume_tokens(1).is_err());
}

#[test]
fn test_resource_memory_alloc() {
    let manager = ResourceManager::new(Budgets {
        cpu_cores: 4,
        memory_mb: 512,
        tokens: 1000,
        api_calls: 10,
        concurrency: 2,
    });
    assert!(manager.allocate_memory(256).is_ok());
    assert!(manager.allocate_memory(256).is_ok());
    assert!(manager.allocate_memory(1).is_err());
    manager.release_memory(256);
    assert!(manager.allocate_memory(1).is_ok());
}

#[test]
fn test_resource_cpu_alloc() {
    let manager = ResourceManager::new(Budgets {
        cpu_cores: 2,
        memory_mb: 1024,
        tokens: 1000,
        api_calls: 10,
        concurrency: 2,
    });
    assert!(manager.allocate_cpu(1).is_ok());
    assert!(manager.allocate_cpu(1).is_ok());
    assert!(manager.allocate_cpu(1).is_err());
    manager.release_cpu(1);
    assert!(manager.allocate_cpu(1).is_ok());
}

// ═══════════════════════════════════════════════════════════════════
// Scheduler Tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_scheduler_priority_ordering() {
    let mut scheduler = AgentScheduler::new(SchedulingStrategy::Priority);
    let low_task = MissionNode {
        id: TaskId::new(),
        name: "low".into(),
        role: AgentRole::Documentation,
        payload: "".into(),
    };
    let high_task = MissionNode {
        id: TaskId::new(),
        name: "high".into(),
        role: AgentRole::Coder,
        payload: "".into(),
    };
    scheduler.enqueue_task(low_task, 1);
    scheduler.enqueue_task(high_task, 10);

    let first = scheduler.dequeue_task().unwrap();
    assert_eq!(first.name, "high");
}

#[test]
fn test_scheduler_round_robin() {
    let scheduler = AgentScheduler::new(SchedulingStrategy::RoundRobin);
    let agents = vec![AgentId::new(), AgentId::new(), AgentId::new()];
    let selected = scheduler.select_agent(&agents);
    assert!(selected.is_some());
}

#[test]
fn test_scheduler_no_agents_available() {
    let scheduler = AgentScheduler::new(SchedulingStrategy::LeastLoaded);
    let selected = scheduler.select_agent(&[]);
    assert!(selected.is_none());
}

// ═══════════════════════════════════════════════════════════════════
// Agent Registry Tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_agent_capabilities() {
    let mut registry = AgentRegistry::new();
    let cap = CapabilitySet {
        capabilities: vec![Capability::CodeGeneration, Capability::TestGeneration],
    };
    registry.register_template(AgentConfig {
        role: AgentRole::Coder,
        capabilities: cap,
        provider: ProviderSelection::Local("test".into()),
        system_prompt: "code".into(),
        temperature: 0.5,
    });

    let id = registry.allocate_agent(&AgentRole::Coder).unwrap();
    let caps = registry.get_capabilities(&id).unwrap();
    assert_eq!(caps.capabilities.len(), 2);
}

#[test]
fn test_agent_missing_template() {
    let mut registry = AgentRegistry::new();
    let result = registry.allocate_agent(&AgentRole::Security);
    assert!(result.is_err());
}

#[test]
fn test_agent_double_release() {
    let mut registry = AgentRegistry::new();
    registry.register_template(AgentConfig {
        role: AgentRole::Tester,
        capabilities: CapabilitySet::default(),
        provider: ProviderSelection::Local("test".into()),
        system_prompt: "test".into(),
        temperature: 0.0,
    });
    let id = registry.allocate_agent(&AgentRole::Tester).unwrap();
    registry.release_agent(&id).unwrap();
    assert!(registry.release_agent(&id).is_err());
}

// ═══════════════════════════════════════════════════════════════════
// Recovery Manager Tests
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_recovery_diagnose() {
    let rm = recovery::RecoveryManager::new();
    let mid = MissionId::new();
    let action = rm.diagnose_failure(&mid).await.unwrap();
    assert!(matches!(action, recovery::RecoveryAction::ResumeMission(_)));
}

#[tokio::test]
async fn test_recovery_apply() {
    let rm = recovery::RecoveryManager::new();
    let dag = MissionDag::new();
    let mut mission = mission::Mission {
        id: MissionId::new(),
        state: MissionState::Failed,
        dag,
    };
    let action = recovery::RecoveryAction::ResumeMission(mission.id);
    assert!(rm.apply_recovery(action, &mut mission).await.is_ok());
}

// ═══════════════════════════════════════════════════════════════════
// Mission Manager Advanced Tests
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_mission_cancel() {
    let manager = mission::MissionManager::new();
    let mid = manager.create_mission(MissionDag::new()).await;
    assert!(manager.cancel_mission(&mid).await.is_ok());
    assert_eq!(
        manager.track_mission(&mid).await.unwrap(),
        MissionState::Cancelled
    );
}

#[tokio::test]
async fn test_mission_checkpoint() {
    let manager = mission::MissionManager::new();
    let mid = manager.create_mission(MissionDag::new()).await;
    assert!(manager.checkpoint_mission(&mid).await.is_ok());
}

#[tokio::test]
async fn test_mission_recover() {
    let manager = mission::MissionManager::new();
    let mid = manager.create_mission(MissionDag::new()).await;
    assert!(manager.recover_mission(&mid).await.is_ok());
    assert_eq!(
        manager.track_mission(&mid).await.unwrap(),
        MissionState::Recovering
    );
}

#[tokio::test]
async fn test_mission_not_found() {
    let manager = mission::MissionManager::new();
    let fake_id = MissionId::new();
    assert!(manager.track_mission(&fake_id).await.is_none());
    assert!(manager.pause_mission(&fake_id).await.is_err());
}

#[tokio::test]
async fn test_mission_pause_requires_executing() {
    let manager = mission::MissionManager::new();
    let mid = manager.create_mission(MissionDag::new()).await;
    // Mission is Pending, not Executing
    assert!(manager.pause_mission(&mid).await.is_err());
}
