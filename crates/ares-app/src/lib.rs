use ares_agent::config::AgentConfig;
use ares_agent::services::hybrid_ranking::HybridRankingConfig;
use ares_agent::services::semantic_retrieval::SemanticSearchService;
use ares_agent::services::{
    context_builder::ContextBuilder, context_pipeline::ContextPipeline,
    contradiction_detector::ContradictionDetector,
    decision_intelligence::DecisionIntelligenceEngine, memory_ranking::MemoryRankingEngine,
    retrieval::SemanticRetrievalLayer,
};
use ares_core::vector::traits::EmbeddingProvider;
use ares_core::AresError;
use ares_embeddings::MockEmbeddingProvider;
use ares_store::db::Store;
use ares_store::repositories::{
    decision::SqliteDecisionRepository, graph::SqliteGraphRepository,
    intelligence::SqliteIntelligenceRepository, memory::SqliteMemoryRepository,
    vector::SqliteVectorRepository,
};
use std::sync::Arc;
use tracing::info;

/// The shared application state containing all intelligence engines and repositories.
#[derive(Clone)]
pub struct AppState {
    pub config: AgentConfig,
    pub store: Store,
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
    // Week 5 - Semantic Search
    pub vector_repo: Arc<SqliteVectorRepository>,
    pub embedding_provider: Arc<dyn EmbeddingProvider>,
    pub semantic_search: Arc<SemanticSearchService>,
}

impl AppState {
    /// Initialize the shared application state from a configuration.
    pub async fn new(config: AgentConfig) -> Result<Self, AresError> {
        info!(project = %config.project_path, "Initializing AppState");

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
        let vector_repo = Arc::new(SqliteVectorRepository::new(store.clone()));

        // Initialize Embedding Provider
        // Defaulting to Mock for safety. Users can configure OpenAI/Ollama via env vars later.
        let embedding_provider = Arc::new(MockEmbeddingProvider::default());

        // Initialize Intelligence Engines
        let ranking_engine = Arc::new(MemoryRankingEngine::new());
        let retrieval_layer = Arc::new(SemanticRetrievalLayer::new(
            memory_repo.clone(),
            intelligence_repo.clone(),
            ranking_engine.clone(),
        ));
        let semantic_search = Arc::new(SemanticSearchService::new(
            embedding_provider.clone(),
            vector_repo.clone(),
            memory_repo.clone(),
            HybridRankingConfig::default(),
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
            vector_repo,
            embedding_provider,
            semantic_search,
        })
    }
}
