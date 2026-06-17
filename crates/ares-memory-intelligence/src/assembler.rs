use std::sync::Arc;
use ares_core::AresError;
use ares_knowledge_graph::queries::CanonicalQueries as GraphQueries;
use ares_memory_evolution::queries::TemporalQueries;
use ares_gap_engine::engine::GapEngine;
use ares_resolution_engine::engine::ResolutionEngine;
use crate::retrieval::engine::RetrievalEngine;
use serde_json::json;

/// The MemoryContextAssembler is the unified intelligence layer
/// that routes and merges requests across bounded contexts without cyclic dependencies.
pub struct MemoryContextAssembler {
    pub graph: Arc<GraphQueries>,
    pub evolution: Arc<TemporalQueries>,
    pub gap_engine: Arc<GapEngine>,
    pub resolution_engine: Arc<ResolutionEngine>,
    pub retrieval_engine: Arc<RetrievalEngine>,
}

impl MemoryContextAssembler {
    pub fn new(
        graph: Arc<GraphQueries>,
        evolution: Arc<TemporalQueries>,
        gap_engine: Arc<GapEngine>,
        resolution_engine: Arc<ResolutionEngine>,
        retrieval_engine: Arc<RetrievalEngine>,
    ) -> Self {
        Self {
            graph,
            evolution,
            gap_engine,
            resolution_engine,
            retrieval_engine,
        }
    }

    pub fn default_from_store(store: ares_store::Store) -> Self {
        let arc_store = Arc::new(store.clone());
        let kg_store = Arc::new(ares_knowledge_graph::store::KnowledgeGraphStore::new(arc_store.clone()));
        let evo_store = Arc::new(ares_memory_evolution::store::MemoryEvolutionStore::new(arc_store.clone()));

        let traversal = Arc::new(ares_knowledge_graph::traversal::TraversalEngine::new(kg_store));
        let impact = Arc::new(ares_knowledge_graph::impact::ImpactEngine::new(traversal.clone()));
        let graph = Arc::new(GraphQueries::new(traversal, impact));

        let evo_engine = Arc::new(ares_memory_evolution::engine::MemoryEvolutionEngine::new(evo_store));
        let supersession = Arc::new(ares_memory_evolution::supersession::SupersessionEngine::new(arc_store.clone()));
        let evolution = Arc::new(TemporalQueries::new(evo_engine, supersession));

        let gap_engine = Arc::new(GapEngine::new(arc_store.clone()));
        let resolution_engine = Arc::new(ResolutionEngine::new());
        let retrieval_engine = Arc::new(RetrievalEngine::new(store.clone()));

        Self::new(graph, evolution, gap_engine, resolution_engine, retrieval_engine)
    }

    /// Example: Unified query to get the full story of an entity.
    /// It queries current state (Graph) and historical state (Evolution).
    pub fn get_entity_full_context(&self, entity_id: &str) -> Result<serde_json::Value, AresError> {
        // 1. Get current state from Graph (using arbitrary downstream lookup as a proxy for "find in graph")
        // Normally this would be a direct node lookup in the graph, but we'll use existing queries.
        let downstream = match self.graph.what_breaks_if_changed(entity_id) {
            Ok(report) => report.impacted_nodes.len(),
            Err(_) => 0,
        };
        
        // 2. Get history from Evolution Engine
        let timeline = self.evolution.how_has_this_evolved(entity_id)?;
        
        // 3. Assemble and merge
        Ok(json!({
            "entity_id": entity_id,
            "current_graph_downstream": downstream,
            "history": timeline,
        }))
    }
}
