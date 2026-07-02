use crate::core::context::RepositoryContext;
use crate::core::metadata::{PlannerTrace, PlannerTraceEvent};
use crate::core::response::{
    KnowledgeBundle, RepositoryResponse, KNOWLEDGE_VERSION, PLANNER_VERSION, SCHEMA_VERSION,
};
use crate::planner::aggregator::EvidenceAggregator;
use crate::planner::builder::PlanBuilder;
use crate::planner::executor::EngineExecutor;
use crate::planner::expander::CapabilityExpander;
use crate::planner::intent::IntentRouter;
use crate::planner::knowledge::KnowledgePipeline;
use crate::planner::optimizer::CapabilityOptimizer;
use crate::planner::registry::EngineRegistry;
use crate::planner::resolver::DependencyResolver;
use crate::planner::scheduler::Scheduler;
use crate::planner::validator::EvidenceValidator;

// ═══════════════════════════════════════════════════════════════════
// Frozen Pipeline: The complete ARES execution pipeline.
//
// Layer 1: RepositoryContext
// Layer 2: Planning (Intent → Plan → Expand → Optimize → Resolve)
// Layer 3: Execution (Schedule → Execute)
// Layer 4: Evidence (Aggregate → Validate)
// Layer 5: Knowledge (Canonicalize → Deduplicate → Confidence → Rank)
// Layer 6: RepositoryResponse
// ═══════════════════════════════════════════════════════════════════

pub struct ExecutionPlanner<'a> {
    registry: &'a EngineRegistry,
}

impl<'a> ExecutionPlanner<'a> {
    pub fn new(registry: &'a EngineRegistry) -> Self {
        Self { registry }
    }

    #[tracing::instrument(skip(self, context), fields(execution_id = %context.execution.execution_id))]
    pub async fn execute(&self, context: &RepositoryContext) -> RepositoryResponse {
        let planner_start = std::time::Instant::now();
        let mut trace = PlannerTrace::new();

        // ── Layer 2: Planning ──────────────────────────────────────

        trace.push(PlannerTraceEvent::PlanningStarted {
            execution_id: context.execution.execution_id.clone(),
            timestamp_ms: context.execution.started_at,
        });
        tracing::info!(
            "Starting execution pipeline for {}",
            context.execution.execution_id
        );

        // Step 1: Intent
        let intent = IntentRouter::parse(&context.request.query);
        trace.push(PlannerTraceEvent::IntentDetected {
            intent: format!("{:?}", intent),
            duration_ms: planner_start.elapsed().as_millis() as u64,
        });

        // Step 2: Plan
        let mut plan = PlanBuilder::build(intent);

        // Step 3: Expand
        let expanded = CapabilityExpander::expand(&plan.intent, plan.requested_capabilities);
        trace.push(PlannerTraceEvent::CapabilityExpanded {
            added: expanded.iter().map(|c| format!("{:?}", c)).collect(),
            duration_ms: planner_start.elapsed().as_millis() as u64,
        });

        // Step 4: Optimize
        let (optimized, removed) = CapabilityOptimizer::optimize(&plan.intent, expanded);
        trace.push(PlannerTraceEvent::CapabilityOptimized {
            removed,
            final_count: optimized.len(),
            duration_ms: planner_start.elapsed().as_millis() as u64,
        });
        plan.requested_capabilities = optimized;

        trace.push(PlannerTraceEvent::PlanBuilt {
            plan_id: plan.plan_id.clone(),
            capabilities: plan
                .requested_capabilities
                .iter()
                .map(|c| format!("{:?}", c))
                .collect(),
            duration_ms: planner_start.elapsed().as_millis() as u64,
        });

        // Step 5: Resolve
        let graph = DependencyResolver::resolve(&plan);
        trace.push(PlannerTraceEvent::DependencyResolved {
            node_count: graph.nodes.len(),
            duration_ms: planner_start.elapsed().as_millis() as u64,
        });

        // ── Layer 3: Execution ─────────────────────────────────────

        let executor = EngineExecutor::new(self.registry);

        let results = match Scheduler::execute_graph(&graph, &executor, context).await {
            Ok(res) => res,
            Err(e) => {
                tracing::error!("Scheduler failed: {}", e);
                Vec::new()
            }
        };

        // ── Layer 4: Evidence ──────────────────────────────────────

        trace.push(PlannerTraceEvent::AggregationStarted);
        let mut raw_evidence = EvidenceAggregator::aggregate(results);
        trace.push(PlannerTraceEvent::AggregationFinished {
            duration_ms: planner_start.elapsed().as_millis() as u64,
        });

        trace.push(PlannerTraceEvent::ValidationStarted);
        let validation = EvidenceValidator::validate(&mut raw_evidence);
        trace.push(PlannerTraceEvent::ValidationFinished {
            issues: validation.issues.len(),
            duration_ms: planner_start.elapsed().as_millis() as u64,
        });

        // ── Layer 5: Knowledge ─────────────────────────────────────

        trace.push(PlannerTraceEvent::KnowledgeStarted);
        let processed_evidence = KnowledgePipeline::process(raw_evidence);
        trace.push(PlannerTraceEvent::KnowledgeFinished {
            duration_ms: planner_start.elapsed().as_millis() as u64,
        });

        // ── Layer 6: RepositoryResponse ────────────────────────────

        let duration = planner_start.elapsed();
        trace.push(PlannerTraceEvent::Completed {
            total_duration_ms: duration.as_millis() as u64,
        });
        tracing::info!("Planner Total duration: {} ms", duration.as_millis());

        RepositoryResponse {
            schema_version: SCHEMA_VERSION,
            planner_version: PLANNER_VERSION.to_string(),
            knowledge_version: KNOWLEDGE_VERSION.to_string(),
            response_id: uuid::Uuid::new_v4().to_string(),
            execution_id: context.execution.execution_id.clone(),
            answer: None,
            evidence: processed_evidence,
            knowledge: KnowledgeBundle::default(),
            metadata: crate::core::metadata::ExecutionMetadata {
                engine: "ExecutionPlanner".to_string(),
                duration_ms: duration.as_millis() as u64,
                cache_hit: false,
                confidence: 1.0,
                errors: vec![],
                warnings: vec![],
                retry_count: 0,
                sources_used: vec![],
            },
            diagnostics: crate::core::response::DiagnosticsBundle {
                health_score: 100.0,
                warnings: vec![],
                errors: vec![],
                missing_requirements: vec![],
            },
            planner_trace: trace,
            replay_id: None,
            artifacts: vec![],
            actions: vec![],
            citations: vec![],
        }
    }
}
