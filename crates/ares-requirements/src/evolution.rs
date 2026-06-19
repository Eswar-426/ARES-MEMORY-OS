use crate::models::{EventOrigin, RequirementEvolutionEvent, RequirementEvolutionType, RequirementTimeline};
use ares_core::{AresError, RequirementId, id::new_id};
use ares_store::db::Store;
use chrono::Utc;
use rusqlite::params;
use tracing::debug;

pub struct RequirementEvolutionStorage {
    store: Store,
}

impl RequirementEvolutionStorage {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    pub fn record_event(
        &self,
        requirement_id: &RequirementId,
        event_type: RequirementEvolutionType,
        event_origin: EventOrigin,
        actor: Option<&str>,
        description: &str,
        correlation_id: Option<&str>,
        previous_score: Option<f32>,
        new_score: Option<f32>,
    ) -> Result<RequirementEvolutionEvent, AresError> {
        let conn = self.store.get_conn()?;
        let event_id = new_id();
        let now = Utc::now().timestamp_micros();

        let event_type_str = serde_json::to_string(&event_type)
            .unwrap_or_else(|_| "\"unknown\"".to_string())
            .replace("\"", "");
        let event_origin_str = serde_json::to_string(&event_origin)
            .unwrap_or_else(|_| "\"recorded\"".to_string())
            .replace("\"", "");

        conn.execute(
            "INSERT INTO requirement_evolution_events (
                id, requirement_id, event_type, event_origin, actor, description, correlation_id, previous_score, new_score, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                event_id,
                requirement_id.as_str(),
                event_type_str,
                event_origin_str,
                actor,
                description,
                correlation_id,
                previous_score,
                new_score,
                now,
            ],
        )
        .map_err(AresError::db)?;

        debug!(req_id = %requirement_id, event = %event_type_str, "Recorded requirement evolution event");

        Ok(RequirementEvolutionEvent {
            id: event_id,
            requirement_id: requirement_id.clone(),
            timestamp: now,
            event_type,
            event_origin,
            actor: actor.map(|s| s.to_string()),
            description: description.to_string(),
            correlation_id: correlation_id.map(|s| s.to_string()),
            previous_score,
            new_score,
        })
    }

    pub fn get_timeline(&self, requirement_id: &RequirementId) -> Result<RequirementTimeline, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, requirement_id, event_type, event_origin, actor, description, correlation_id, previous_score, new_score, created_at
                 FROM requirement_evolution_events
                 WHERE requirement_id = ?1
                 ORDER BY created_at ASC"
            )
            .map_err(AresError::db)?;

        let rows = stmt
            .query_map(params![requirement_id.as_str()], |row| {
                let r_id: String = row.get(1)?;
                let e_type_str: String = row.get(2)?;
                let origin_str: String = row.get(3)?;
                
                let event_type: RequirementEvolutionType = serde_json::from_str(&format!("\"{}\"", e_type_str)).unwrap_or(RequirementEvolutionType::RequirementCreated);
                let event_origin: EventOrigin = serde_json::from_str(&format!("\"{}\"", origin_str)).unwrap_or(EventOrigin::Recorded);

                Ok(RequirementEvolutionEvent {
                    id: row.get(0)?,
                    requirement_id: RequirementId::from(r_id),
                    event_type,
                    event_origin,
                    actor: row.get(4)?,
                    description: row.get(5)?,
                    correlation_id: row.get(6)?,
                    previous_score: row.get(7)?,
                    new_score: row.get(8)?,
                    timestamp: row.get(9)?,
                })
            })
            .map_err(AresError::db)?;

        let mut events = Vec::new();
        for row in rows {
            events.push(row.map_err(AresError::db)?);
        }

        Ok(RequirementTimeline {
            requirement_id: requirement_id.clone(),
            events,
        })
    }
}

pub struct RequirementEvolutionEngine {
    storage: RequirementEvolutionStorage,
}

impl RequirementEvolutionEngine {
    pub fn new(store: Store) -> Self {
        Self {
            storage: RequirementEvolutionStorage::new(store),
        }
    }

    pub fn get_timeline(&self, requirement_id: &RequirementId) -> Result<RequirementTimeline, AresError> {
        self.storage.get_timeline(requirement_id)
    }

    pub fn record_event(
        &self,
        requirement_id: &RequirementId,
        event_type: RequirementEvolutionType,
        event_origin: EventOrigin,
        actor: Option<&str>,
        description: &str,
        correlation_id: Option<&str>,
        previous_score: Option<f32>,
        new_score: Option<f32>,
    ) -> Result<RequirementEvolutionEvent, AresError> {
        self.storage.record_event(
            requirement_id,
            event_type,
            event_origin,
            actor,
            description,
            correlation_id,
            previous_score,
            new_score,
        )
    }
}
