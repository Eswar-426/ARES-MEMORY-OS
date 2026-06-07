#[cfg(test)]
mod tests {
    use crate::services::memory_ranking::MemoryRankingEngine;
    use crate::services::retrieval::SemanticRetrievalLayer;
    use ares_core::{CreateMemoryInput, MemoryType, ProjectId};
    use ares_store::repositories::intelligence::SqliteIntelligenceRepository;
    use ares_store::repositories::memory::SqliteMemoryRepository;
    use ares_store::repositories::project::SqliteProjectRepository;
    use std::sync::Arc;
    use tempfile::TempDir;

    fn test_store() -> (ares_store::db::Store, TempDir) {
        let dir = TempDir::new().unwrap();
        let store = ares_store::db::Store::open(&dir.path().join("test.db")).unwrap();
        (store, dir)
    }

    #[test]
    fn test_retrieval_performance() {
        let (store, _dir) = test_store();

        let memory_repo = Arc::new(SqliteMemoryRepository::new(store.clone()));
        let intell_repo = Arc::new(SqliteIntelligenceRepository::new(store.clone()));
        let ranking_engine = Arc::new(MemoryRankingEngine::new());
        let retrieval = SemanticRetrievalLayer::new(
            memory_repo.clone(),
            intell_repo.clone(),
            ranking_engine.clone(),
        );

        let project_id = ProjectId::new();
        let project_repo = SqliteProjectRepository::new(store.clone());
        project_repo
            .create(&ares_core::Project {
                id: project_id.clone(),
                name: "test".to_string(),
                description: "".to_string(),
                root_path: "/tmp/test".to_string(),
                primary_language: "rust".to_string(),
                domain: "".to_string(),
                maturity: ares_core::ProjectMaturity::Greenfield,
                created_at: 0,
                updated_at: 0,
                deleted_at: None,
            })
            .unwrap();

        // Seed some memories
        memory_repo
            .create(CreateMemoryInput {
                project_id: project_id.clone(),
                memory_type: MemoryType::Feature,
                title: "Auth JWT".to_string(),
                content: serde_json::json!({"desc": "JWT based authentication"}),
                confidence: None,
                importance: None,
                source: None,
                ai_assisted: None,
            })
            .unwrap();

        let start = std::time::Instant::now();
        let _results = retrieval
            .retrieve(&project_id, "authentication", 10)
            .unwrap();
        let duration = start.elapsed();

        println!("Retrieval took: {:?}", duration);
        assert!(duration.as_millis() < 500, "Performance target: < 500 ms");

        // The query "authentication" matches "Auth JWT" because "auth" is a prefix
        // actually FTS might need exact or porter stemmer matches.
        // Even if results are empty, the pipeline execution time is what we test here.
        // The test passes if it returns under 500ms.
    }
}
