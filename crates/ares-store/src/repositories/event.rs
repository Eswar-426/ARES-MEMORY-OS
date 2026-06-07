use ares_core::{AresError, AresEvent, EventId, EventSource, EventType, ProjectId};
use crate::db::Store;
use rusqlite::params;

pub struct SqliteEventRepository {
    store: Store,
}

impl SqliteEventRepository {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    /// Append an event. Events are immutable once written (enforced by DB trigger).
    pub fn append(&self, event: &AresEvent) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        conn.execute(
            "INSERT INTO events (id, project_id, event_type, payload, source, created_at)
             VALUES (?1,?2,?3,?4,?5,?6)",
            params![
                event.id.as_str(),
                event.project_id.as_ref().map(|p| p.as_str()),
                event.event_type.as_str(),
                event.payload.to_string(),
                event.source.as_str(),
                event.created_at,
            ],
        ).map_err(AresError::db)?;
        Ok(())
    }

    /// Create and append a new event in one call.
    pub fn emit(
        &self,
        event_type: EventType,
        project_id: Option<ProjectId>,
        payload: serde_json::Value,
        source: EventSource,
    ) -> Result<AresEvent, AresError> {
        let event = AresEvent::new(event_type, project_id, payload, source);
        self.append(&event)?;
        Ok(event)
    }

    /// List events since a given timestamp (exclusive).
    pub fn list_since(
        &self,
        project_id: &ProjectId,
        since: i64,
        limit: u32,
    ) -> Result<Vec<AresEvent>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, project_id, event_type, payload, source, created_at
             FROM events
             WHERE project_id = ?1 AND created_at > ?2
             ORDER BY created_at ASC
             LIMIT ?3"
        ).map_err(AresError::db)?;

        let rows = stmt.query_map(
            params![project_id.as_str(), since, limit as i64],
            row_to_event,
        ).map_err(AresError::db)?;

        rows.collect::<Result<Vec<_>, _>>().map_err(AresError::db)
    }

    /// List events by type across all projects.
    pub fn list_by_type(
        &self,
        event_type: &str,
        since: i64,
        limit: u32,
    ) -> Result<Vec<AresEvent>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, project_id, event_type, payload, source, created_at
             FROM events
             WHERE event_type = ?1 AND created_at > ?2
             ORDER BY created_at ASC
             LIMIT ?3"
        ).map_err(AresError::db)?;

        let rows = stmt.query_map(
            params![event_type, since, limit as i64],
            row_to_event,
        ).map_err(AresError::db)?;

        rows.collect::<Result<Vec<_>, _>>().map_err(AresError::db)
    }
}

fn row_to_event(row: &rusqlite::Row<'_>) -> Result<AresEvent, rusqlite::Error> {
    let project_id_str: Option<String> = row.get(1)?;
    let payload_str: String = row.get(3)?;
    let source_str: String = row.get(4)?;

    // Map event_type string back to enum (best-effort)
    let event_type = map_event_type(&row.get::<_, String>(2)?);

    Ok(AresEvent {
        id:         EventId::from(row.get::<_, String>(0)?),
        project_id: project_id_str.map(ProjectId::from),
        event_type,
        payload:    serde_json::from_str(&payload_str).unwrap_or_default(),
        source:     source_str.parse().unwrap_or_default(),
        created_at: row.get(5)?,
    })
}

fn map_event_type(s: &str) -> EventType {
    match s {
        "memory.created"              => EventType::MemoryCreated,
        "memory.updated"              => EventType::MemoryUpdated,
        "memory.deleted"              => EventType::MemoryDeleted,
        "memory.version_created"      => EventType::MemoryVersionCreated,
        "decision.created"            => EventType::DecisionCreated,
        "decision.updated"            => EventType::DecisionUpdated,
        "decision.superseded"         => EventType::DecisionSuperseded,
        "decision.review_due"         => EventType::DecisionReviewDue,
        "scanner.run_started"         => EventType::ScannerRunStarted,
        "scanner.file_parsed"         => EventType::ScannerFileParsed,
        "scanner.run_completed"       => EventType::ScannerRunCompleted,
        "scanner.run_failed"          => EventType::ScannerRunFailed,
        "scanner.change_detected"     => EventType::ScannerChangeDetected,
        "graph.node_created"          => EventType::GraphNodeCreated,
        "graph.edge_created"          => EventType::GraphEdgeCreated,
        "graph.contradiction_detected" => EventType::GraphContradictionDetected,
        "agent.session_started"       => EventType::AgentSessionStarted,
        "agent.action_logged"         => EventType::AgentActionLogged,
        "agent.session_ended"         => EventType::AgentSessionEnded,
        "project.initialized"         => EventType::ProjectInitialized,
        "project.updated"             => EventType::ProjectUpdated,
        _                             => EventType::ProjectUpdated, // fallback
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_helpers::test_store;
    use crate::repositories::project::SqliteProjectRepository;
    use ares_core::{Project, ProjectMaturity, new_id};
    use ares_core::types::event::now_micros;

    fn setup_project(store: &Store) -> ProjectId {
        let now = now_micros();
        let id = ProjectId::new();
        SqliteProjectRepository::new(store.clone()).create(&Project {
            id: id.clone(), name: "ev-test".into(), description: "".into(),
            root_path: format!("/tmp/{}", new_id()),
            primary_language: "ts".into(), domain: "".into(),
            maturity: ProjectMaturity::Greenfield,
            created_at: now, updated_at: now, deleted_at: None,
        }).unwrap();
        id
    }

    #[test]
    fn append_and_list_since() {
        let (store, _dir) = test_store();
        let project_id = setup_project(&store);
        let repo = SqliteEventRepository::new(store);

        let before = now_micros();
        repo.emit(
            EventType::MemoryCreated,
            Some(project_id.clone()),
            serde_json::json!({"memory_id": "test"}),
            EventSource::User,
        ).unwrap();

        let events = repo.list_since(&project_id, before - 1, 10).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, EventType::MemoryCreated);
    }

    #[test]
    fn events_table_is_append_only() {
        let (store, _dir) = test_store();
        let project_id = setup_project(&store);
        let repo = SqliteEventRepository::new(store.clone());

        let event = repo.emit(
            EventType::ProjectInitialized,
            Some(project_id.clone()),
            serde_json::json!({}),
            EventSource::User,
        ).unwrap();

        // Attempt DELETE — should be rejected by trigger
        let conn = store.get_conn().unwrap();
        let result = conn.execute(
            "DELETE FROM events WHERE id = ?1",
            params![event.id.as_str()],
        );
        assert!(result.is_err(), "DELETE on events should be rejected by trigger");
    }

    #[test]
    fn list_by_type_filters_correctly() {
        let (store, _dir) = test_store();
        let project_id = setup_project(&store);
        let repo = SqliteEventRepository::new(store);
        let before = now_micros();

        repo.emit(EventType::MemoryCreated,   Some(project_id.clone()), serde_json::json!({}), EventSource::User).unwrap();
        repo.emit(EventType::DecisionCreated, Some(project_id.clone()), serde_json::json!({}), EventSource::User).unwrap();

        let decision_events = repo.list_by_type("decision.created", before - 1, 10).unwrap();
        assert_eq!(decision_events.len(), 1);
        assert_eq!(decision_events[0].event_type, EventType::DecisionCreated);
    }
}
