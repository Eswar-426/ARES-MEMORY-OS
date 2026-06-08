use crate::services::hybrid_ranking::HybridRankingConfig;
use crate::services::semantic_retrieval::SemanticSearchService;
use ares_core::vector::traits::EmbeddingProvider;
use ares_core::{ImportanceLevel, MemorySource, MemoryType, ProjectId};
use ares_embeddings::MockEmbeddingProvider;
use ares_store::db::Store;
use ares_store::repositories::memory::SqliteMemoryRepository;
use ares_store::repositories::vector::SqliteVectorRepository;
use std::sync::Arc;
use std::time::Instant;
use tempfile::TempDir;

fn test_store() -> (Store, TempDir) {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = dir.path().join("test.db");
    let store = Store::open(&db_path).expect("Failed to open test store");
    (store, dir)
}

#[tokio::test]
async fn performance_benchmarks() {
    let (store, _dir) = test_store();
    let memory_repo = Arc::new(SqliteMemoryRepository::new(store.clone()));
    let vector_repo = Arc::new(SqliteVectorRepository::new(store.clone()));
    let embedding_provider = Arc::new(MockEmbeddingProvider::new(128));

    let semantic_search = SemanticSearchService::new(
        embedding_provider.clone(),
        vector_repo.clone(),
        memory_repo.clone(),
        HybridRankingConfig::default(),
    );

    let project_id = ProjectId::from("perf_proj");

    // Create the project to satisfy foreign key constraints
    store.get_conn().unwrap().execute(
        "INSERT INTO projects (id, name, description, root_path, primary_language, domain, maturity, created_at, updated_at)
         VALUES ('perf_proj', 'Perf', '', '/perf', 'rust', '', 'mature', 0, 0)",
        [],
    ).unwrap();

    // 1. Populate DB with 1000 items
    let mut memories = Vec::new();
    for i in 0..1000 {
        let input = ares_core::CreateMemoryInput {
            project_id: project_id.clone(),
            memory_type: MemoryType::Feature,
            title: format!("Performance test memory {}", i),
            content: serde_json::json!({ "details": format!("This is memory number {}", i) }),
            confidence: Some(1.0),
            importance: Some(ImportanceLevel::Medium),
            source: Some(MemorySource::Human),
            ai_assisted: Some(false),
        };
        let mem = memory_repo.create(input).unwrap();
        memories.push(mem);
    }

    // Benchmark Embedding Generation (Mock)
    let embed_start = Instant::now();
    let mut batch_texts = Vec::new();
    for m in &memories {
        batch_texts.push(m.title.as_str());
    }
    let _ = embedding_provider.embed_batch(&batch_texts).await.unwrap();
    let embed_latency = embed_start.elapsed().as_millis();

    // Check if it's less than 500ms
    println!("Embedding Generation (1000 items): {} ms", embed_latency);
    assert!(embed_latency < 500, "Embedding generation took too long");

    // Benchmark Reindex Throughput
    let reindex_start = Instant::now();
    let reindexed = semantic_search.reindex_project(&project_id).await.unwrap();
    let reindex_latency = reindex_start.elapsed().as_millis();

    println!("Reindex Throughput (1000 items): {} ms", reindex_latency);
    assert_eq!(reindexed, 1000);

    // Benchmark Semantic Search (Vector Search + Keyword + Hybrid)
    let search_start = Instant::now();
    let response = semantic_search
        .search(&project_id, "test memory 500", 10)
        .await
        .unwrap();
    let search_latency = search_start.elapsed().as_millis();

    println!("Semantic Search End-to-End: {} ms", search_latency);
    assert!(search_latency < 200, "Semantic search took too long");

    println!("Retrieval Diagnostics: {:#?}", response.diagnostics);
}
