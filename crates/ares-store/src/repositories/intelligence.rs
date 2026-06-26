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

use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionExplanation {
    pub id: Uuid,
    pub task_id: String,
    pub decision_type: String,
    pub model_id: String,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub id: Uuid,
    pub task_id: String,
    pub selected_model_id: String,
    pub fallback_model_id: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTrace {
    pub id: Uuid,
    pub task_id: String,
    pub model_id: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub latency_ms: i64,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostEvent {
    pub id: Uuid,
    pub model_id: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub total_cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderHealthEvent {
    pub provider_id: String,
    pub status: String,
    pub last_checked_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerState {
    pub provider_id: String,
    pub state: String,
    pub failure_count: i64,
    pub opened_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningEvent {
    pub id: Uuid,
    pub model_id: String,
    pub success_rate: f64,
    pub latency_ms: i64,
    pub updated_at: DateTime<Utc>,
}

#[async_trait::async_trait]
pub trait IntelligenceRepository: Send + Sync {
    async fn save_selection_explanation(
        &self,
        explanation: SelectionExplanation,
    ) -> anyhow::Result<()>;
    async fn save_routing_decision(&self, decision: RoutingDecision) -> anyhow::Result<()>;
    async fn save_execution_trace(&self, trace: ExecutionTrace) -> anyhow::Result<()>;
    async fn save_cost_event(&self, event: CostEvent) -> anyhow::Result<()>;
    async fn save_learning_event(&self, event: LearningEvent) -> anyhow::Result<()>;
    async fn save_provider_health(&self, health: ProviderHealthEvent) -> anyhow::Result<()>;
    async fn get_provider_health(
        &self,
        provider_id: &str,
    ) -> anyhow::Result<Option<ProviderHealthEvent>>;
    async fn save_circuit_breaker_state(&self, state: CircuitBreakerState) -> anyhow::Result<()>;
    async fn get_circuit_breaker_state(
        &self,
        provider_id: &str,
    ) -> anyhow::Result<Option<CircuitBreakerState>>;
}

#[async_trait::async_trait]
impl IntelligenceRepository for SqliteIntelligenceRepository {
    async fn save_selection_explanation(
        &self,
        explanation: SelectionExplanation,
    ) -> anyhow::Result<()> {
        let conn = self.store.get_conn()?;
        conn.execute(
            "INSERT INTO model_explanations (id, decision_type, model_id, explanation) VALUES (?1, ?2, ?3, ?4)",
            params![
                explanation.id.to_string(),
                explanation.decision_type,
                explanation.model_id,
                explanation.explanation
            ],
        )?;
        Ok(())
    }

    async fn save_routing_decision(&self, decision: RoutingDecision) -> anyhow::Result<()> {
        let conn = self.store.get_conn()?;
        conn.execute(
            "INSERT INTO routing_history (id, task_id, selected_model_id, fallback_model_id, reason) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                decision.id.to_string(),
                decision.task_id,
                decision.selected_model_id,
                decision.fallback_model_id,
                decision.reason
            ],
        )?;
        Ok(())
    }

    async fn save_execution_trace(&self, trace: ExecutionTrace) -> anyhow::Result<()> {
        let conn = self.store.get_conn()?;
        conn.execute(
            "INSERT INTO execution_traces (id, task_id, model_id, start_time, end_time, latency_ms, success) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                trace.id.to_string(),
                trace.task_id,
                trace.model_id,
                trace.start_time.timestamp_millis(),
                trace.end_time.timestamp_millis(),
                trace.latency_ms,
                trace.success
            ],
        )?;
        Ok(())
    }

    async fn save_cost_event(&self, event: CostEvent) -> anyhow::Result<()> {
        let conn = self.store.get_conn()?;
        conn.execute(
            "INSERT INTO model_costs (id, model_id, input_tokens, output_tokens, total_cost) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                event.id.to_string(),
                event.model_id,
                event.input_tokens,
                event.output_tokens,
                event.total_cost
            ],
        )?;
        Ok(())
    }

    async fn save_learning_event(&self, event: LearningEvent) -> anyhow::Result<()> {
        let conn = self.store.get_conn()?;
        let payload = serde_json::to_string(&event)?;
        conn.execute(
            "INSERT INTO intelligence_events (id, event_type, payload, created_at) VALUES (?1, 'LEARNING_EVENT', ?2, ?3)",
            params![
                event.id.to_string(),
                payload,
                event.updated_at.timestamp_millis()
            ],
        )?;
        Ok(())
    }

    async fn save_provider_health(&self, health: ProviderHealthEvent) -> anyhow::Result<()> {
        let conn = self.store.get_conn()?;
        conn.execute(
            "INSERT INTO provider_health (provider_id, status, last_checked_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(provider_id) DO UPDATE SET status = excluded.status, last_checked_at = excluded.last_checked_at",
            params![
                health.provider_id,
                health.status,
                health.last_checked_at.timestamp_millis()
            ],
        )?;
        Ok(())
    }

    async fn get_provider_health(
        &self,
        provider_id: &str,
    ) -> anyhow::Result<Option<ProviderHealthEvent>> {
        let conn = self.store.get_conn()?;
        let result = conn.query_row(
            "SELECT status, last_checked_at FROM provider_health WHERE provider_id = ?1",
            params![provider_id],
            |row| {
                let status: String = row.get(0)?;
                let ts: i64 = row.get(1)?;
                Ok(ProviderHealthEvent {
                    provider_id: provider_id.to_string(),
                    status,
                    last_checked_at: Utc.timestamp_millis_opt(ts).unwrap(),
                })
            },
        );

        match result {
            Ok(h) => Ok(Some(h)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn save_circuit_breaker_state(&self, state: CircuitBreakerState) -> anyhow::Result<()> {
        let conn = self.store.get_conn()?;
        let opened_ts = state.opened_at.map(|d| d.timestamp_millis());
        conn.execute(
            "INSERT INTO circuit_breaker_states (provider_id, state, failure_count, opened_at) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(provider_id) DO UPDATE SET state = excluded.state, failure_count = excluded.failure_count, opened_at = excluded.opened_at",
            params![
                state.provider_id,
                state.state,
                state.failure_count,
                opened_ts
            ],
        )?;
        Ok(())
    }

    async fn get_circuit_breaker_state(
        &self,
        provider_id: &str,
    ) -> anyhow::Result<Option<CircuitBreakerState>> {
        let conn = self.store.get_conn()?;
        let result = conn.query_row(
            "SELECT state, failure_count, opened_at FROM circuit_breaker_states WHERE provider_id = ?1",
            params![provider_id],
            |row| {
                let state: String = row.get(0)?;
                let failure_count: i64 = row.get(1)?;
                let ts: Option<i64> = row.get(2)?;
                let opened_at = ts.map(|t| Utc.timestamp_millis_opt(t).unwrap());

                Ok(CircuitBreakerState {
                    provider_id: provider_id.to_string(),
                    state,
                    failure_count,
                    opened_at,
                })
            },
        );

        match result {
            Ok(h) => Ok(Some(h)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}
