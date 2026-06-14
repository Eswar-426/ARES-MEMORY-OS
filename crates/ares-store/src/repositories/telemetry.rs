use crate::db::Store;
use ares_core::AresError;
use chrono::Utc;
use rusqlite::params;
use serde_json::Value;
use uuid::Uuid;

pub struct TelemetryReport {
    pub id: String,
    pub timestamp: String,
    pub source: String,
    pub continuity_score: f64,
    pub provider_health: Value,
    pub fallback_events: Value,
    pub dynamic_chains: Value,
}

pub struct TelemetryRepository<'a> {
    store: &'a Store,
}

impl<'a> TelemetryRepository<'a> {
    pub fn new(store: &'a Store) -> Self {
        Self { store }
    }

    pub fn save_report(
        &self,
        source: &str,
        continuity_score: f64,
        provider_health: Value,
        fallback_events: Value,
        dynamic_chains: Value,
    ) -> Result<String, AresError> {
        let id = Uuid::new_v4().to_string();
        let timestamp = Utc::now().to_rfc3339();

        let ph_json = serde_json::to_string(&provider_health).unwrap_or_default();
        let fe_json = serde_json::to_string(&fallback_events).unwrap_or_default();
        let dc_json = serde_json::to_string(&dynamic_chains).unwrap_or_default();

        self.store.with_transaction(|tx| {
            tx.execute(
                "INSERT INTO telemetry_reports (id, timestamp, source, continuity_score, provider_health, fallback_events, dynamic_chains) 
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![id, timestamp, source, continuity_score, ph_json, fe_json, dc_json],
            ).map_err(AresError::db)?;
            Ok(())
        })?;

        Ok(id)
    }

    pub fn get_latest_report(&self) -> Result<Option<TelemetryReport>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, source, continuity_score, provider_health, fallback_events, dynamic_chains 
             FROM telemetry_reports 
             ORDER BY timestamp DESC 
             LIMIT 1"
        ).map_err(AresError::db)?;

        let mut rows = stmt.query([]).map_err(AresError::db)?;

        if let Some(row) = rows.next().map_err(AresError::db)? {
            let ph_str: String = row.get(4).map_err(AresError::db)?;
            let fe_str: String = row.get(5).map_err(AresError::db)?;
            let dc_str: String = row.get(6).map_err(AresError::db)?;

            let report = TelemetryReport {
                id: row.get(0).map_err(AresError::db)?,
                timestamp: row.get(1).map_err(AresError::db)?,
                source: row.get(2).map_err(AresError::db)?,
                continuity_score: row.get(3).map_err(AresError::db)?,
                provider_health: serde_json::from_str(&ph_str).unwrap_or(Value::Null),
                fallback_events: serde_json::from_str(&fe_str).unwrap_or(Value::Null),
                dynamic_chains: serde_json::from_str(&dc_str).unwrap_or(Value::Null),
            };
            Ok(Some(report))
        } else {
            Ok(None)
        }
    }
}
