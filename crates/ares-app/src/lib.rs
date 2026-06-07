use ares_agent::config::AgentConfig;
use ares_agent::services::{
    context_pipeline::ContextPipeline,
    contradiction_detector::ContradictionDetector,
    decision_intelligence::DecisionIntelligenceEngine,
    memory_ranking::MemoryRankingEngine,
    retrieval::SemanticRetrievalLayer,
    context_builder::ContextBuilder,
};
use ares_core::AresError;
use ares_store::db::Store;
use ares_store::repositories::{
    decision::SqliteDecisionRepository,
    graph::SqliteGraphRepository,
    intelligence::SqliteIntelligenceRepository,
    memory::SqliteMemoryRepository,
};
use std::sync::Arc;
use tracing::info;

/// The shared application state containing all intelligence engines and repositories.
#[derive(Clone)]
pub struct AppState {
    pub config: AgentConfig,
    pub store: Arc<Store>,
    pub memory_repo: Arc<SqliteMemoryRepository>,
    pub intelligence_repo: Arc<SqliteIntelligenceRepository>,
    pub decision_repo: Arc<SqliteDecisionRepository>,
    pub graph_repo: Arc<SqliteGraphRepository>,
    pub ranking_engine: Arc<MemoryRankingEngine>,
    pub retrieval_layer: Arc<SemanticRetrievalLayer>,
    pub context_builder: Arc<ContextBuilder>,
    pub decision_intelligence: Arc<DecisionIntelligenceEngine>,
    pub context_pipeline: Arc<ContextPipeline>,
    pub contradiction_detector: Arc<ContradictionDetector>,
}

impl AppState {
    /// Initialize the shared application state from a configuration.
    pub async fn new(config: AgentConfig) -> Result<Self, AresError> {
        info!(project = %config.project_path, "Initializing AppState");

        // Initialize Store
        let db_path = std::path::Path::new(&config.project_path)
            .join(".ares")
            .join("ares.db");
        let store = Arc::new(Store::open(&db_path).map_err(|_| AresError::Database("Failed to open DB".into()))?);

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
            config,
            store,
            memory_repo,
            intelligence_repo,
            decision_repo,
            graph_repo,
            ranking_engine,
            retrieval_layer,
            context_builder,
            decision_intelligence,
            context_pipeline,
            contradiction_detector,
        })
    }
}
