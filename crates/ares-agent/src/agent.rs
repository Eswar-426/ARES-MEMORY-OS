use crate::config::AgentConfig;
use ares_core::AresError;
use tracing::info;

use crate::services::context_builder::ContextBuilder;
use crate::services::context_pipeline::ContextPipeline;
use crate::services::contradiction_detector::ContradictionDetector;
use crate::services::decision_intelligence::DecisionIntelligenceEngine;
use crate::services::memory_ranking::MemoryRankingEngine;
use crate::services::retrieval::SemanticRetrievalLayer;

use ares_store::db::Store;
use ares_store::repositories::decision::SqliteDecisionRepository;
use ares_store::repositories::graph::SqliteGraphRepository;
use ares_store::repositories::intelligence::SqliteIntelligenceRepository;
use ares_store::repositories::memory::SqliteMemoryRepository;

use std::sync::Arc;

/// The ARES Local Agent — manages lifecycle of IPC server + scanner.
/// Implemented progressively across Weeks 4–7.
pub struct Agent {
    _config: AgentConfig,
    context_pipeline: Arc<ContextPipeline>,
    contradiction_detector: Arc<ContradictionDetector>,
    decision_intelligence: Arc<DecisionIntelligenceEngine>,
}

impl Agent {
    pub async fn new(config: AgentConfig) -> Result<Self, AresError> {
        info!(project = %config.project_path, "Agent initialized");

        // Initialize Store
        let db_path = std::path::Path::new(&config.project_path)
            .join(".ares")
            .join("ares.db");
        let store =
            Store::open(&db_path).map_err(|_| AresError::Database("Failed to open DB".into()))?;

        // Initialize Repositories
        let memory_repo = Arc::new(SqliteMemoryRepository::new(store.clone()));
        let intelligence_repo = Arc::new(SqliteIntelligenceRepository::new(store.clone()));
        let decision_repo = Arc::new(SqliteDecisionRepository::new(store.clone()));
        let graph_repo = Arc::new(SqliteGraphRepository::new(store.clone()));

        // Initialize Intelligence Engines
        let ranking_engine = Arc::new(MemoryRankingEngine::new());
        let retrieval_layer = Arc::new(SemanticRetrievalLayer::new(
            memory_repo.clone(),
            intelligence_repo.clone(),
            ranking_engine.clone(),
        ));
        let context_builder = Arc::new(ContextBuilder::new());

        let decision_intelligence = Arc::new(DecisionIntelligenceEngine::new(
            decision_repo.clone(),
            graph_repo.clone(),
        ));

        let context_pipeline = Arc::new(ContextPipeline::new(
            retrieval_layer.clone(),
            decision_repo.clone(),
            graph_repo.clone(),
            context_builder.clone(),
        ));

        let contradiction_detector = Arc::new(ContradictionDetector::new(
            graph_repo.clone(),
            intelligence_repo.clone(),
        ));

        Ok(Self {
            _config: config,
            context_pipeline,
            contradiction_detector,
            decision_intelligence,
        })
    }

    /// Retrieve an AI-ready context snapshot for a query.
    pub fn get_context(
        &self,
        project: &ares_core::Project,
        query: &str,
        budget: crate::services::context_builder::ContextBudget,
    ) -> Result<crate::services::context_builder::ContextSnapshot, AresError> {
        self.context_pipeline
            .assemble_context(project, query, budget)
    }

    /// Perform a scan of the project to find contradictions.
    pub fn detect_contradictions(
        &self,
        project: &ares_core::Project,
        nodes_to_check: &[ares_core::NodeId],
    ) -> Result<Vec<ares_core::ContradictionRecord>, AresError> {
        self.contradiction_detector
            .detect_contradictions(&project.id, nodes_to_check)
    }

    /// Ask why a decision was made.
    pub fn why_was_this_done(
        &self,
        project: &ares_core::Project,
        decision_id: &ares_core::id::DecisionId,
    ) -> Result<Vec<(ares_core::GraphEdge, ares_core::GraphNode)>, AresError> {
        self.decision_intelligence
            .why_was_this_done(&project.id, decision_id)
    }

    /// Find what replaced a decision.
    pub fn what_replaced_this(
        &self,
        project: &ares_core::Project,
        decision_id: &ares_core::id::DecisionId,
    ) -> Result<Option<ares_core::Decision>, AresError> {
        self.decision_intelligence
            .what_replaced_this(&project.id, decision_id)
    }

    /// Find decisions derived from this one.
    pub fn what_evolved_from_this(
        &self,
        project: &ares_core::Project,
        decision_id: &ares_core::id::DecisionId,
    ) -> Result<Vec<ares_core::Decision>, AresError> {
        self.decision_intelligence
            .what_evolved_from_this(&project.id, decision_id)
    }

    /// Get decision history.
    pub fn decision_history(
        &self,
        project: &ares_core::Project,
        decision_id: &ares_core::id::DecisionId,
    ) -> Result<Vec<ares_core::Decision>, AresError> {
        self.decision_intelligence
            .decision_history(&project.id, decision_id)
    }

    /// Main event loop — starts IPC server and waits for shutdown signal.
    pub async fn run(&mut self) -> Result<(), AresError> {
        // TODO Week 4: Start IPC server
        // TODO Week 4: Register project
        // TODO Week 6: Start scanner
        // TODO Week 7: Start file watcher

        info!("Agent running — press Ctrl+C to stop");

        // Graceful shutdown on Ctrl+C
        tokio::signal::ctrl_c().await.map_err(AresError::Io)?;
        info!("Agent shutting down");
        Ok(())
    }
}
