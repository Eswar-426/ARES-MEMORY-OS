use crate::services::context_builder::{ContextBudget, ReasoningContext, ReasoningContextBuilder};
use crate::services::context_intelligence::ContextIntelligenceEngine;
use crate::services::contradiction_detector::ContradictionReasoner;
use crate::services::dependency_analysis::DependencyAnalyzer;
use crate::services::evolution_engine::EvolutionEngine;
use crate::services::intent_analysis::IntentAnalyzer;
use crate::services::retrieval::SemanticRetrievalLayer;

use ares_core::types::reasoning::{ReasoningDiagnostics, ReasoningEvidence, ReasoningGraph};
use ares_core::{AresError, NodeId, Project};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningOutput {
    pub context: ReasoningContext,
    pub evidence: ReasoningEvidence,
    pub graph: ReasoningGraph,
    pub diagnostics: ReasoningDiagnostics,
}

pub struct ReasoningPipeline {
    intent_analyzer: Arc<IntentAnalyzer>,
    retrieval_layer: Arc<SemanticRetrievalLayer>,
    dependency_analyzer: Arc<DependencyAnalyzer>,
    contradiction_reasoner: Arc<ContradictionReasoner>,
    evolution_engine: Arc<EvolutionEngine>,
    context_intelligence: Arc<ContextIntelligenceEngine>,
    context_builder: Arc<ReasoningContextBuilder>,
}

impl ReasoningPipeline {
    pub fn new(
        intent_analyzer: Arc<IntentAnalyzer>,
        retrieval_layer: Arc<SemanticRetrievalLayer>,
        dependency_analyzer: Arc<DependencyAnalyzer>,
        contradiction_reasoner: Arc<ContradictionReasoner>,
        evolution_engine: Arc<EvolutionEngine>,
        context_intelligence: Arc<ContextIntelligenceEngine>,
        context_builder: Arc<ReasoningContextBuilder>,
    ) -> Self {
        Self {
            intent_analyzer,
            retrieval_layer,
            dependency_analyzer,
            contradiction_reasoner,
            evolution_engine,
            context_intelligence,
            context_builder,
        }
    }

    pub fn reason(
        &self,
        project: &Project,
        query: &str,
        budget: ContextBudget,
    ) -> Result<ReasoningOutput, AresError> {
        let start_time = Instant::now();
        let project_id = &project.id;

        let mut diagnostics = ReasoningDiagnostics::default();

        // 1. Intent Analysis
        let intent_result = self.intent_analyzer.analyze(query);

        // 2. Semantic Retrieval
        let retrieval_start = Instant::now();
        let memories = self.retrieval_layer.retrieve(project_id, query, 10)?;
        diagnostics.retrieval_ms = retrieval_start.elapsed().as_millis() as u64;

        // Collect memory IDs for evidence
        let memory_ids: Vec<String> = memories.iter().map(|m| m.id.as_str().to_string()).collect();
        // Decisions should be fetched here too, mocked empty
        let decisions = vec![];

        // 3. Dependency Analysis
        let dependency_start = Instant::now();
        // If we had memory graph nodes we'd use them, let's just pick the first memory id as a dummy node for this logic
        if let Some(first_mem) = memories.first() {
            let node_id = NodeId::from(first_mem.id.as_str().to_string());
            let _dep_analysis = self.dependency_analyzer.impacts(&node_id, Some(3))?;
        }
        diagnostics.dependency_ms = dependency_start.elapsed().as_millis() as u64;

        // 4. Decision Analysis (Skipped/mocked inside Context Intelligence)

        // 5. Contradiction Analysis
        let contradiction_start = Instant::now();
        let _contradiction_analysis = self.contradiction_reasoner.analyze(project_id, &[]);
        diagnostics.contradiction_ms = contradiction_start.elapsed().as_millis() as u64;

        // 6. Evolution Analysis
        let evolution_start = Instant::now();
        let timeline = if let Some(first_mem) = memories.first() {
            Some(
                self.evolution_engine
                    .memory_timeline(first_mem.id.as_str())?,
            )
        } else {
            None
        };
        diagnostics.evolution_ms = evolution_start.elapsed().as_millis() as u64;

        // 7. Context Intelligence
        let context_analysis = self
            .context_intelligence
            .analyze_context(&memory_ids, &[], &[])?;

        // 8. Context Assembly
        let assembly_start = Instant::now();
        let reasoning_context = self.context_builder.build(
            project,
            query,
            memories,
            decisions,
            context_analysis,
            timeline,
            budget,
        );
        diagnostics.assembly_ms = assembly_start.elapsed().as_millis() as u64;

        diagnostics.total_ms = start_time.elapsed().as_millis() as u64;

        // Build ReasoningEvidence
        let evidence = ReasoningEvidence {
            memory_ids,
            decision_ids: vec![],
            contradiction_ids: vec![],
            graph_paths: vec![],
            confidence: reasoning_context.confidence * intent_result.confidence,
        };

        // Build mock ReasoningGraph
        let graph = ReasoningGraph {
            nodes: vec![],
            edges: vec![],
            confidence: 0.9,
        };

        Ok(ReasoningOutput {
            context: reasoning_context,
            evidence,
            graph,
            diagnostics,
        })
    }
}
