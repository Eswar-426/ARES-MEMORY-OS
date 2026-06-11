use super::models::*;
use ares_core::AresError;
use ares_store::db::Store;
use rusqlite::params;

/// SQLite-backed repository for knowledge evolution events.
pub struct EvolutionRepository {
    store: Store,
}

impl EvolutionRepository {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    /// Insert an evolution entry.
    pub fn insert(&self, entry: &KnowledgeEvolutionEntry) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        conn.execute(
            "INSERT INTO knowledge_evolution (id, semantic_memory_id, event_type,
             old_confidence, new_confidence, reason, source_episode_id, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                entry.id,
                entry.semantic_memory_id,
                entry.event_type.as_str(),
                entry.old_confidence,
                entry.new_confidence,
                entry.reason,
                entry.source_episode_id,
                entry.created_at,
            ],
        )
        .map_err(AresError::db)?;
        Ok(())
    }

    /// Get evolution history for a semantic memory.
    pub fn get_history(
        &self,
        semantic_memory_id: &str,
    ) -> Result<Vec<KnowledgeEvolutionEntry>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, semantic_memory_id, event_type, old_confidence, new_confidence,
                        reason, source_episode_id, created_at
                 FROM knowledge_evolution WHERE semantic_memory_id = ?1
                 ORDER BY created_at DESC LIMIT 100",
            )
            .map_err(AresError::db)?;

        let rows = stmt
            .query_map(params![semantic_memory_id], |row| {
                Ok(KnowledgeEvolutionEntry {
                    id: row.get(0)?,
                    semantic_memory_id: row.get(1)?,
                    event_type: EvolutionEventType::from_str_val(&row.get::<_, String>(2)?),
                    old_confidence: row.get(3)?,
                    new_confidence: row.get(4)?,
                    reason: row.get(5)?,
                    source_episode_id: row.get(6)?,
                    created_at: row.get(7)?,
                })
            })
            .map_err(AresError::db)?;

        let mut entries = Vec::new();
        for row in rows {
            entries.push(row.map_err(AresError::db)?);
        }
        Ok(entries)
    }

    /// Count evolution events.
    pub fn count(&self) -> Result<u64, AresError> {
        let conn = self.store.get_conn()?;
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM knowledge_evolution", [], |row| {
                row.get(0)
            })
            .map_err(AresError::db)?;
        Ok(count as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::test_store;
    use chrono::Utc;

    fn make_entry(id: &str, sm_id: &str) -> KnowledgeEvolutionEntry {
        KnowledgeEvolutionEntry {
            id: id.into(),
            semantic_memory_id: sm_id.into(),
            event_type: EvolutionEventType::Reinforcement,
            old_confidence: 0.7,
            new_confidence: 0.85,
            reason: "Test reason".into(),
            source_episode_id: None,
            created_at: Utc::now().timestamp_micros(),
        }
    }

    fn setup_semantic_memory(store: &Store, id: &str) {
        let conn = store.get_conn().unwrap();
        let now = Utc::now().timestamp_micros();
        conn.execute(
            "INSERT INTO semantic_memories (id, memory_type, subject, predicate, object,
             confidence, evidence_count, tags, created_at, updated_at)
             VALUES (?1, 'entity', 'Test', '', '', 0.8, 1, '[]', ?2, ?2)",
            params![id, now],
        )
        .unwrap();
    }

    #[test]
    fn insert_and_get_history() {
        let (store, _dir) = test_store();
        setup_semantic_memory(&store, "sm_hist");
        let repo = EvolutionRepository::new(store);

        repo.insert(&make_entry("ke_1", "sm_hist")).unwrap();
        repo.insert(&make_entry("ke_2", "sm_hist")).unwrap();

        let history = repo.get_history("sm_hist").unwrap();
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn get_history_empty() {
        let (store, _dir) = test_store();
        let repo = EvolutionRepository::new(store);
        let history = repo.get_history("nonexistent").unwrap();
        assert!(history.is_empty());
    }

    #[test]
    fn count_evolution_events() {
        let (store, _dir) = test_store();
        setup_semantic_memory(&store, "sm_cnt");
        let repo = EvolutionRepository::new(store);
        assert_eq!(repo.count().unwrap(), 0);

        repo.insert(&make_entry("ke_c1", "sm_cnt")).unwrap();
        assert_eq!(repo.count().unwrap(), 1);
    }
}
