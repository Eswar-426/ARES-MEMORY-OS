use super::models::*;
use ares_core::AresError;
use ares_store::db::Store;
use rusqlite::params;

/// SQLite-backed repository for retrieval log entries.
pub struct RetrievalRepository {
    store: Store,
}

impl RetrievalRepository {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    /// Log a retrieval query.
    pub fn log_retrieval(&self, entry: &RetrievalLogEntry) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        let result_ids_json = serde_json::to_string(&entry.result_ids)
            .map_err(|e| AresError::Serialization(e.to_string()))?;

        conn.execute(
            "INSERT INTO memory_retrieval_log (id, query_text, query_type, results_count,
             result_ids, relevance_score, retrieval_ms, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                entry.id,
                entry.query_text,
                entry.query_type,
                entry.results_count,
                result_ids_json,
                entry.relevance_score,
                entry.retrieval_ms,
                entry.created_at,
            ],
        )
        .map_err(AresError::db)?;
        Ok(())
    }

    /// Get recent retrieval logs.
    pub fn recent_logs(&self, limit: u32) -> Result<Vec<RetrievalLogEntry>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, query_text, query_type, results_count, result_ids,
                        relevance_score, retrieval_ms, created_at
                 FROM memory_retrieval_log ORDER BY created_at DESC LIMIT ?1",
            )
            .map_err(AresError::db)?;

        let rows = stmt
            .query_map(params![limit], |row| {
                let ids_str: String = row.get(4)?;
                Ok(RetrievalLogEntry {
                    id: row.get(0)?,
                    query_text: row.get(1)?,
                    query_type: row.get(2)?,
                    results_count: row.get(3)?,
                    result_ids: serde_json::from_str(&ids_str).unwrap_or_default(),
                    relevance_score: row.get(5)?,
                    retrieval_ms: row.get(6)?,
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

    /// Count retrieval logs.
    pub fn count(&self) -> Result<u64, AresError> {
        let conn = self.store.get_conn()?;
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM memory_retrieval_log", [], |row| {
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

    fn make_log(id: &str) -> RetrievalLogEntry {
        RetrievalLogEntry {
            id: id.into(),
            query_text: "deploy failures".into(),
            query_type: "failure_search".into(),
            results_count: 3,
            result_ids: vec!["r_1".into(), "r_2".into()],
            relevance_score: 0.75,
            retrieval_ms: 12,
            created_at: Utc::now().timestamp_micros(),
        }
    }

    #[test]
    fn log_and_retrieve() {
        let (store, _dir) = test_store();
        let repo = RetrievalRepository::new(store);
        repo.log_retrieval(&make_log("rl_1")).unwrap();

        let logs = repo.recent_logs(10).unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].query_text, "deploy failures");
        assert_eq!(logs[0].result_ids.len(), 2);
    }

    #[test]
    fn recent_logs_respects_limit() {
        let (store, _dir) = test_store();
        let repo = RetrievalRepository::new(store);
        for i in 0..5 {
            repo.log_retrieval(&make_log(&format!("rl_{}", i))).unwrap();
        }

        let logs = repo.recent_logs(3).unwrap();
        assert_eq!(logs.len(), 3);
    }

    #[test]
    fn count_logs() {
        let (store, _dir) = test_store();
        let repo = RetrievalRepository::new(store);
        assert_eq!(repo.count().unwrap(), 0);
        repo.log_retrieval(&make_log("rl_cnt")).unwrap();
        assert_eq!(repo.count().unwrap(), 1);
    }
}
