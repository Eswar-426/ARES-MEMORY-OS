use ares_core::AresError;
use ares_store::db::Store;
use rusqlite::params;

pub struct CacheService {
    db: Store,
}

impl CacheService {
    pub fn new(db: Store) -> Self {
        Self { db }
    }

    pub async fn set(&self, key: &str, data: &str) -> Result<(), AresError> {
        let conn = self.db.get_conn()?;
        conn.execute(
            "INSERT INTO knowledge_cache (id, cache_key, data, created_at) 
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(cache_key) DO UPDATE SET data = excluded.data",
            params![
                uuid::Uuid::now_v7().to_string(),
                key,
                data,
                chrono::Utc::now().to_rfc3339()
            ],
        )
        .map_err(|e| AresError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn get(&self, key: &str) -> Result<Option<String>, AresError> {
        let conn = self.db.get_conn()?;
        let mut stmt = conn
            .prepare("SELECT data FROM knowledge_cache WHERE cache_key = ?1")
            .map_err(|e| AresError::Database(e.to_string()))?;
        let mut rows = stmt
            .query(params![key])
            .map_err(|e| AresError::Database(e.to_string()))?;

        if let Some(row) = rows
            .next()
            .map_err(|e| AresError::Database(e.to_string()))?
        {
            Ok(Some(row.get(0).unwrap()))
        } else {
            Ok(None)
        }
    }
}
