use crate::services::context_builder::{ContextBudget, ReasoningContext, ReasoningContextBuilder};
use crate::services::context_intelligence::ContextAnalysis;
use crate::services::retrieval::SemanticRetrievalLayer;
use ares_core::{AresError, Project};
use ares_store::repositories::decision::SqliteDecisionRepository;
use ares_store::repositories::graph::SqliteGraphRepository;
use std::sync::Arc;

pub struct ContextPipeline {
    retrieval_layer: Arc<SemanticRetrievalLayer>,
    decision_repo: Arc<SqliteDecisionRepository>,
    _graph_repo: Arc<SqliteGraphRepository>,
    context_builder: Arc<ReasoningContextBuilder>,
}

impl ContextPipeline {
    pub fn new(
        retrieval_layer: Arc<SemanticRetrievalLayer>,
        decision_repo: Arc<SqliteDecisionRepository>,
        graph_repo: Arc<SqliteGraphRepository>,
        context_builder: Arc<ReasoningContextBuilder>,
    ) -> Self {
        Self {
            retrieval_layer,
            decision_repo,
            _graph_repo: graph_repo,
            context_builder,
        }
    }

    pub fn assemble_context(
        &self,
        project: &Project,
        query: &str,
        budget: ContextBudget,
    ) -> Result<ReasoningContext, AresError> {
        let project_id = &project.id;

        // 1. Memory Retrieval
        let memories = self.retrieval_layer.retrieve(project_id, query, 20)?;

        // 2. Decision Retrieval (Mocked as taking recent decisions for now, or based on query)
        // In a real system, decisions would also be full-text searched.
        // We'll just fetch a page of recent active decisions.
        let decisions = self
            .decision_repo
            .list(project_id, ares_core::DecisionFilter::default())?;

        // 3. Graph Expansion
        // As a simple placeholder, just fetch recent edges for the project

        // 4. Context Builder
        let snapshot = self.context_builder.build(
            project,
            query,
            memories,
            decisions,
            context_builder_analysis(),
            None,
            budget,
        );

        Ok(snapshot)
    }
}

fn context_builder_analysis() -> ContextAnalysis {
    ContextAnalysis {
        relevant_memories: vec![],
        related_decisions: vec![],
        contradictions: vec![],
        dependency_chain: vec![],
        reasoning_summary: "".into(),
        confidence: 1.0,
    }
}
