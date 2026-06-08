use crate::services::context_builder::{ContextBudget, ReasoningContextBuilder};
use ares_core::{Project, ProjectId, ProjectMaturity};
use std::time::Instant;

#[test]
fn test_intent_analysis_performance() {
    let intent = crate::services::intent_analysis::IntentAnalyzer::new();
    let start = Instant::now();
    let _ = intent.analyze("why is auth failing");
    let elapsed = start.elapsed();
    assert!(
        elapsed.as_millis() < 10,
        "Intent analysis took {}ms (limit 10ms)",
        elapsed.as_millis()
    );
}

#[test]
fn test_context_assembly_performance() {
    let builder = ReasoningContextBuilder::new();
    let project = Project {
        id: ProjectId::new(),
        name: "".into(),
        description: "".into(),
        root_path: "".into(),
        primary_language: "".into(),
        domain: "".into(),
        maturity: ProjectMaturity::Greenfield,
        created_at: 0,
        updated_at: 0,
        deleted_at: None,
    };

    let start = Instant::now();
    let _ = builder.build(
        &project,
        "test query",
        vec![],
        vec![],
        crate::services::context_intelligence::ContextAnalysis {
            relevant_memories: vec![],
            related_decisions: vec![],
            contradictions: vec![],
            dependency_chain: vec![],
            reasoning_summary: "test".into(),
            confidence: 1.0,
        },
        None,
        ContextBudget::budget_8k(),
    );
    let elapsed = start.elapsed();
    assert!(
        elapsed.as_millis() < 300,
        "Context assembly took {}ms (limit 300ms)",
        elapsed.as_millis()
    );
}

#[test]
fn test_evolution_analysis_performance() {
    let engine = crate::services::evolution_engine::EvolutionEngine::new();
    let start = Instant::now();
    let _ = engine.memory_timeline("dummy_id");
    let elapsed = start.elapsed();
    assert!(
        elapsed.as_millis() < 150,
        "Evolution analysis took {}ms (limit 150ms)",
        elapsed.as_millis()
    );
}

// In a real environment with populated repos, we'd also run:
// test_dependency_analysis_performance (< 100ms)
// test_full_pipeline_performance (< 750ms)
