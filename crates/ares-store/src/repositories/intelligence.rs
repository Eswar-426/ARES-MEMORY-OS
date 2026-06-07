use crate::db::Store;
use ares_core::types::event::now_micros;
use ares_core::{
    AccessContext, AresError, ContradictionRecord, MemoryAccessLog, MemoryId, NodeId, ProjectId,
    RankingCache,
};
use rusqlite::params;
use uuid::Uuid;

pub struct SqliteIntelligenceRepository {
    store: Store,
}

impl SqliteIntelligenceRepository {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    // ----------------------------------------------------------------
    // Memory Access Logs
    // ----------------------------------------------------------------
    pub fn log_access(
        &self,
        project_id: &ProjectId,
        memory_id: &MemoryId,
        context: AccessContext,
    ) -> Result<MemoryAccessLog, AresError> {
        let now = now_micros();
        let id = Uuid::now_v7().to_string();
        let conn = self.store.get_conn()?;

        conn.execute(
            "INSERT INTO memory_access_log (id, memory_id, project_id, accessed_at, context)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                id,
                memory_id.as_str(),
                project_id.as_str(),
                now,
                context.as_str()
            ],
        )
        .map_err(AresError::db)?;

        Ok(MemoryAccessLog {
            id,
            memory_id: memory_id.clone(),
            project_id: project_id.clone(),
            accessed_at: now,
            context,
        })
    }

    pub fn get_access_count(&self, memory_id: &MemoryId) -> Result<u32, AresError> {
        let conn = self.store.get_conn()?;
        let count: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM memory_access_log WHERE memory_id = ?1",
                params![memory_id.as_str()],
                |row| row.get(0),
            )
            .unwrap_or(0);
        Ok(count)
    }

    // ----------------------------------------------------------------
    // Ranking Cache
    // ----------------------------------------------------------------
    pub fn set_ranking(
        &self,
        project_id: &ProjectId,
        memory_id: &MemoryId,
        score: f32,
    ) -> Result<RankingCache, AresError> {
        let now = now_micros();
        let conn = self.store.get_conn()?;

        conn.execute(
            "INSERT INTO ranking_cache (memory_id, project_id, score, updated_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(memory_id) DO UPDATE SET score=excluded.score, updated_at=excluded.updated_at",
            params![
                memory_id.as_str(),
                project_id.as_str(),
                score,
                now,
            ],
        ).map_err(AresError::db)?;

        Ok(RankingCache {
            memory_id: memory_id.clone(),
            project_id: project_id.clone(),
            score,
            updated_at: now,
        })
    }

    pub fn get_ranking(&self, memory_id: &MemoryId) -> Result<Option<RankingCache>, AresError> {
        let conn = self.store.get_conn()?;
        let result = conn.query_row(
            "SELECT memory_id, project_id, score, updated_at FROM ranking_cache WHERE memory_id = ?1",
            params![memory_id.as_str()],
            |row| {
                Ok(RankingCache {
                    memory_id: MemoryId::from(row.get::<_, String>(0)?),
                    project_id: ProjectId::from(row.get::<_, String>(1)?),
                    score: row.get(2)?,
                    updated_at: row.get(3)?,
                })
            },
        );
        match result {
            Ok(c) => Ok(Some(c)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AresError::db(e)),
        }
    }

    // ----------------------------------------------------------------
    // Contradiction Records
    // ----------------------------------------------------------------
    pub fn record_contradiction(
        &self,
        project_id: &ProjectId,
        source_id: &NodeId,
        target_id: &NodeId,
        reason: &str,
        confidence: f32,
    ) -> Result<ContradictionRecord, AresError> {
        let now = now_micros();
        let id = Uuid::now_v7().to_string();
        let conn = self.store.get_conn()?;

        conn.execute(
            "INSERT INTO contradiction_records (id, project_id, source_id, target_id, reason, confidence, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                id,
                project_id.as_str(),
                source_id.as_str(),
                target_id.as_str(),
                reason,
                confidence,
                now,
            ],
        ).map_err(AresError::db)?;

        Ok(ContradictionRecord {
            id,
            project_id: project_id.clone(),
            source_id: source_id.clone(),
            target_id: target_id.clone(),
            reason: reason.to_string(),
            confidence,
            created_at: now,
            resolved_at: None,
        })
    }

    pub fn resolve_contradiction(&self, id: &str) -> Result<(), AresError> {
        let now = now_micros();
        let conn = self.store.get_conn()?;
        let rows = conn.execute(
            "UPDATE contradiction_records SET resolved_at = ?1 WHERE id = ?2 AND resolved_at IS NULL",
            params![now, id],
        ).map_err(AresError::db)?;

        if rows == 0 {
            return Err(AresError::not_found("contradiction_record", id));
        }
        Ok(())
    }
}
