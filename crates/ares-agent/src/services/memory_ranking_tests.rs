#[cfg(test)]
mod tests {
    use crate::services::memory_ranking::MemoryRankingEngine;
    use ares_core::{
        ImportanceLevel, Memory, MemoryId, MemorySource, MemoryStatus, MemoryType, ProjectId,
    };
    use std::time::{SystemTime, UNIX_EPOCH};

    fn make_memory(importance: ImportanceLevel, age_days: i64) -> Memory {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as i64;
        let age_micros = age_days * 24 * 60 * 60 * 1_000_000;
        Memory {
            id: MemoryId::new(),
            project_id: ProjectId::new(),
            memory_type: MemoryType::Feature,
            title: "Test".to_string(),
            content: serde_json::Value::Null,
            status: MemoryStatus::Active,
            version: 1,
            parent_id: None,
            confidence: 1.0,
            importance,
            source: MemorySource::Human,
            ai_assisted: false,
            created_at: now - age_micros,
            updated_at: now - age_micros,
            deleted_at: None,
        }
    }

    #[test]
    fn test_memory_ranking_importance_decay() {
        let engine = MemoryRankingEngine::new();

        // Critical memory: 200 days old
        let mem_critical = make_memory(ImportanceLevel::Critical, 200);
        // Low importance memory: 10 days old
        let mem_low = make_memory(ImportanceLevel::Low, 10);

        let candidate_memories = vec![mem_low.clone(), mem_critical.clone()];
        let relevance_scores = vec![(mem_low.id.clone(), 0.5), (mem_critical.id.clone(), 0.5)];
        let access_counts = vec![];

        let start = std::time::Instant::now();
        let ranked = engine.rank_memories(&candidate_memories, &relevance_scores, &access_counts);
        let duration = start.elapsed();

        println!("Memory ranking took: {:?}", duration);
        assert!(duration.as_millis() < 100, "Performance target: < 100 ms");

        // Despite being much older, the Critical memory should rank higher because of slower decay and higher base importance.
        assert_eq!(ranked[0].0.id, mem_critical.id);
        assert_eq!(ranked[1].0.id, mem_low.id);

        assert!(ranked[0].1.final_score > ranked[1].1.final_score);
    }
}
