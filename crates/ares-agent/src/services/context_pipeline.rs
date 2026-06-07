use crate::services::context_builder::{ContextBudget, ContextBuilder, ContextSnapshot};
use crate::services::retrieval::SemanticRetrievalLayer;
use ares_core::{AresError, Project};
use ares_store::repositories::decision::SqliteDecisionRepository;
use ares_store::repositories::graph::SqliteGraphRepository;
use std::sync::Arc;

pub struct ContextPipeline {
    retrieval_layer: Arc<SemanticRetrievalLayer>,
    decision_repo: Arc<SqliteDecisionRepository>,
    _graph_repo: Arc<SqliteGraphRepository>,
    context_builder: Arc<ContextBuilder>,
}

impl ContextPipeline {
    pub fn new(
        retrieval_layer: Arc<SemanticRetrievalLayer>,
        decision_repo: Arc<SqliteDecisionRepository>,
        graph_repo: Arc<SqliteGraphRepository>,
        context_builder: Arc<ContextBuilder>,
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
    ) -> Result<ContextSnapshot, AresError> {
        let project_id = &project.id;

        // 1. Memory Retrieval
        let memories =
            self.retrieval_layer
                .retrieve(project_id, query, budget.max_memories as u32)?;

        // 2. Decision Retrieval (Mocked as taking recent decisions for now, or based on query)
        // In a real system, decisions would also be full-text searched.
        // We'll just fetch a page of recent active decisions.
        let decisions = self
            .decision_repo
            .list(project_id, ares_core::DecisionFilter::default())?;

        // 3. Graph Expansion
        // We fetch nodes related to the retrieved memories and decisions
        let graph_nodes = Vec::new();
        let graph_edges = Vec::new();

        // As a simple placeholder, just fetch recent edges for the project
        // In a real graph expansion, we'd traverse outwards from retrieved memory/decision node IDs.
        // For Week 3, returning empty graph slices or basic lists suffices if we don't implement full graph traversal here.

        // 4. Context Builder
        let snapshot = self.context_builder.build(
            project,
            query,
            memories,
            decisions,
            graph_nodes,
            graph_edges,
            budget,
        );

        Ok(snapshot)
    }
}
