use super::models::*;
use ares_core::AresError;
use ares_store::db::Store;
use rusqlite::params;

/// SQLite-backed repository for semantic memory persistence.
pub struct SemanticRepository {
    store: Store,
}

impl SemanticRepository {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    /// Insert a new semantic memory.
    pub fn insert(&self, mem: &SemanticMemory) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        let tags_json = serde_json::to_string(&mem.tags)
            .map_err(|e| AresError::Serialization(e.to_string()))?;
        conn.execute(
            "INSERT INTO semantic_memories (id, source_episode_id, memory_type, subject, predicate,
             object, confidence, evidence_count, tags, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                mem.id,
                mem.source_episode_id,
                mem.memory_type.as_str(),
                mem.subject,
                mem.predicate,
                mem.object,
                mem.confidence,
                mem.evidence_count,
                tags_json,
                mem.created_at,
                mem.updated_at,
            ],
        )
        .map_err(AresError::db)?;
        Ok(())
    }

    /// Get a semantic memory by ID.
    pub fn get(&self, id: &str) -> Result<Option<SemanticMemory>, AresError> {
        let conn = self.store.get_conn()?;
        let result = conn.query_row(
            "SELECT id, source_episode_id, memory_type, subject, predicate, object,
                    confidence, evidence_count, tags, created_at, updated_at
             FROM semantic_memories WHERE id = ?1",
            params![id],
            |row| Self::row_to_memory(row),
        );
        match result {
            Ok(m) => Ok(Some(m)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AresError::db(e)),
        }
    }

    /// Query semantic memories with filters.
    pub fn query(&self, q: &SemanticQuery) -> Result<Vec<SemanticMemory>, AresError> {
        let conn = self.store.get_conn()?;
        let mut sql = String::from(
            "SELECT id, source_episode_id, memory_type, subject, predicate, object,
                    confidence, evidence_count, tags, created_at, updated_at
             FROM semantic_memories WHERE 1=1",
        );
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        let mut idx = 1;

        if let Some(ref mt) = q.memory_type {
            sql.push_str(&format!(" AND memory_type = ?{}", idx));
            param_values.push(Box::new(mt.as_str().to_string()));
            idx += 1;
        }
        if let Some(ref subj) = q.subject {
            sql.push_str(&format!(" AND subject LIKE ?{}", idx));
            param_values.push(Box::new(format!("%{}%", subj)));
            idx += 1;
        }
        if let Some(min_conf) = q.min_confidence {
            sql.push_str(&format!(" AND confidence >= ?{}", idx));
            param_values.push(Box::new(min_conf));
            idx += 1;
        }
        let _ = idx;

        sql.push_str(" ORDER BY confidence DESC");
        let limit = q.limit.unwrap_or(100);
        sql.push_str(&format!(" LIMIT {}", limit));

        let params_ref: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();
        let mut stmt = conn.prepare(&sql).map_err(AresError::db)?;
        let rows = stmt
            .query_map(params_ref.as_slice(), |row| Self::row_to_memory(row))
            .map_err(AresError::db)?;

        let mut memories = Vec::new();
        for row in rows {
            memories.push(row.map_err(AresError::db)?);
        }
        Ok(memories)
    }

    /// Update confidence and evidence count for a semantic memory.
    pub fn update_confidence(
        &self,
        id: &str,
        new_confidence: f64,
        new_evidence_count: u32,
    ) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        let now = chrono::Utc::now().timestamp_micros();
        let rows = conn
            .execute(
                "UPDATE semantic_memories SET confidence = ?1, evidence_count = ?2, updated_at = ?3
                 WHERE id = ?4",
                params![new_confidence, new_evidence_count, now, id],
            )
            .map_err(AresError::db)?;
        if rows == 0 {
            return Err(AresError::not_found("semantic_memory", id));
        }
        Ok(())
    }

    /// Find memories by subject match.
    pub fn find_by_subject(&self, subject: &str) -> Result<Vec<SemanticMemory>, AresError> {
        let query = SemanticQuery {
            subject: Some(subject.into()),
            ..Default::default()
        };
        self.query(&query)
    }

    /// Delete a semantic memory by ID.
    pub fn delete(&self, id: &str) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        conn.execute("DELETE FROM semantic_memories WHERE id = ?1", params![id])
            .map_err(AresError::db)?;
        Ok(())
    }

    /// Count total semantic memories.
    pub fn count(&self) -> Result<u64, AresError> {
        let conn = self.store.get_conn()?;
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM semantic_memories", [], |row| {
                row.get(0)
            })
            .map_err(AresError::db)?;
        Ok(count as u64)
    }

    fn row_to_memory(row: &rusqlite::Row<'_>) -> Result<SemanticMemory, rusqlite::Error> {
        let tags_str: String = row.get(8)?;
        Ok(SemanticMemory {
            id: row.get(0)?,
            source_episode_id: row.get(1)?,
            memory_type: SemanticMemoryType::from_str_val(&row.get::<_, String>(2)?),
            subject: row.get(3)?,
            predicate: row.get(4)?,
            object: row.get(5)?,
            confidence: row.get(6)?,
            evidence_count: row.get(7)?,
            tags: serde_json::from_str(&tags_str).unwrap_or_default(),
            created_at: row.get(9)?,
            updated_at: row.get(10)?,
        })
    }
}

/// Create a test semantic memory.
#[cfg(test)]
pub fn make_test_semantic(id: &str, mem_type: SemanticMemoryType) -> SemanticMemory {
    let now = chrono::Utc::now().timestamp_micros();
    SemanticMemory {
        id: id.into(),
        source_episode_id: None,
        memory_type: mem_type,
        subject: "TestSubject".into(),
        predicate: "USES".into(),
        object: "TestObject".into(),
        confidence: 0.8,
        evidence_count: 1,
        tags: vec!["test".into()],
        created_at: now,
        updated_at: now,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::test_store;

    #[test]
    fn insert_and_get_semantic_memory() {
        let (store, _dir) = test_store();
        let repo = SemanticRepository::new(store);
        let mem = make_test_semantic("sm_1", SemanticMemoryType::Entity);
        repo.insert(&mem).unwrap();

        let fetched = repo.get("sm_1").unwrap().unwrap();
        assert_eq!(fetched.subject, "TestSubject");
        assert_eq!(fetched.memory_type, SemanticMemoryType::Entity);
    }

    #[test]
    fn get_nonexistent_returns_none() {
        let (store, _dir) = test_store();
        let repo = SemanticRepository::new(store);
        assert!(repo.get("nope").unwrap().is_none());
    }

    #[test]
    fn query_by_type() {
        let (store, _dir) = test_store();
        let repo = SemanticRepository::new(store);
        repo.insert(&make_test_semantic("sm_e", SemanticMemoryType::Entity))
            .unwrap();
        repo.insert(&make_test_semantic(
            "sm_r",
            SemanticMemoryType::Relationship,
        ))
        .unwrap();

        let q = SemanticQuery {
            memory_type: Some(SemanticMemoryType::Entity),
            ..Default::default()
        };
        let results = repo.query(&q).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "sm_e");
    }

    #[test]
    fn query_by_subject() {
        let (store, _dir) = test_store();
        let repo = SemanticRepository::new(store);
        let mut mem = make_test_semantic("sm_sub", SemanticMemoryType::Entity);
        mem.subject = "Authentication".into();
        repo.insert(&mem).unwrap();

        let results = repo.find_by_subject("Auth").unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn query_by_min_confidence() {
        let (store, _dir) = test_store();
        let repo = SemanticRepository::new(store);

        let mut high = make_test_semantic("sm_hi", SemanticMemoryType::Fact);
        high.confidence = 0.95;
        repo.insert(&high).unwrap();

        let mut low = make_test_semantic("sm_lo", SemanticMemoryType::Fact);
        low.confidence = 0.3;
        repo.insert(&low).unwrap();

        let q = SemanticQuery {
            min_confidence: Some(0.5),
            ..Default::default()
        };
        let results = repo.query(&q).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "sm_hi");
    }

    #[test]
    fn update_confidence() {
        let (store, _dir) = test_store();
        let repo = SemanticRepository::new(store);
        repo.insert(&make_test_semantic("sm_upd", SemanticMemoryType::Entity))
            .unwrap();

        repo.update_confidence("sm_upd", 0.99, 5).unwrap();
        let fetched = repo.get("sm_upd").unwrap().unwrap();
        assert!((fetched.confidence - 0.99).abs() < f64::EPSILON);
        assert_eq!(fetched.evidence_count, 5);
    }

    #[test]
    fn update_nonexistent_fails() {
        let (store, _dir) = test_store();
        let repo = SemanticRepository::new(store);
        let result = repo.update_confidence("nope", 0.5, 1);
        assert!(result.is_err());
    }

    #[test]
    fn delete_semantic_memory() {
        let (store, _dir) = test_store();
        let repo = SemanticRepository::new(store);
        repo.insert(&make_test_semantic("sm_del", SemanticMemoryType::Concept))
            .unwrap();
        assert_eq!(repo.count().unwrap(), 1);

        repo.delete("sm_del").unwrap();
        assert_eq!(repo.count().unwrap(), 0);
    }

    #[test]
    fn count_semantic_memories() {
        let (store, _dir) = test_store();
        let repo = SemanticRepository::new(store);
        assert_eq!(repo.count().unwrap(), 0);

        repo.insert(&make_test_semantic("sm_c1", SemanticMemoryType::Entity))
            .unwrap();
        repo.insert(&make_test_semantic("sm_c2", SemanticMemoryType::Fact))
            .unwrap();
        assert_eq!(repo.count().unwrap(), 2);
    }
}
