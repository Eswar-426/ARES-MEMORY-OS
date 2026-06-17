use super::models::{Worker, WorkerStatus, WorkerResources};
use ares_core::AresError;
use ares_store::db::Store;
use rusqlite::params;

pub struct WorkerRepository {
    store: Store,
}

impl WorkerRepository {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    pub fn register(&self, worker: &Worker) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        let capabilities_json = serde_json::to_string(&worker.capabilities).unwrap_or_default();
        let labels_json = serde_json::to_string(&worker.labels).unwrap_or_default();
        let resources_json = serde_json::to_string(&worker.resources).unwrap_or_default();
        let status_str =
            serde_json::to_string(&worker.status).unwrap_or_else(|_| "\"Registering\"".to_string());
        // remove quotes from status
        let status_str = status_str.replace("\"", "");

        conn.execute(
            "INSERT INTO workers (id, hostname, capabilities, labels, status, resources, registered_at, last_heartbeat) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                worker.id,
                worker.hostname,
                capabilities_json,
                labels_json,
                status_str,
                resources_json,
                worker.registered_at,
                worker.last_heartbeat
            ],
        ).map_err(AresError::db)?;
        Ok(())
    }

    pub fn get(&self, id: &str) -> Result<Option<Worker>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn.prepare("SELECT id, hostname, capabilities, labels, status, resources, registered_at, last_heartbeat FROM workers WHERE id = ?1").map_err(AresError::db)?;

        let mut rows = stmt.query(params![id]).map_err(AresError::db)?;
        if let Some(row) = rows.next().map_err(AresError::db)? {
            let capabilities_json: String = row.get(2).map_err(AresError::db)?;
            let labels_json: String = row.get(3).map_err(AresError::db)?;
            let status_str: String = row.get(4).map_err(AresError::db)?;
            let resources_json: String = row.get(5).map_err(AresError::db)?;

            let capabilities = serde_json::from_str(&capabilities_json).unwrap_or_default();
            let labels = serde_json::from_str(&labels_json).unwrap_or_default();
            let status = serde_json::from_str(&format!("\"{}\"", status_str))
                .unwrap_or(WorkerStatus::Offline);
            let resources =
                serde_json::from_str(&resources_json).unwrap_or(WorkerResources {
                    cpu: 0.0,
                    memory: 0,
                    disk: 0,
                    available_cpu: 0.0,
                    available_memory: 0,
                });

            Ok(Some(Worker {
                id: row.get(0).map_err(AresError::db)?,
                hostname: row.get(1).map_err(AresError::db)?,
                capabilities,
                labels,
                status,
                resources,
                registered_at: row.get(6).map_err(AresError::db)?,
                last_heartbeat: row.get(7).map_err(AresError::db)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn update_status(
        &self,
        id: &str,
        status: &WorkerStatus,
        resources_json: &str,
        last_heartbeat: &str,
    ) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        let status_str = serde_json::to_string(status)
            .unwrap_or_else(|_| "\"Offline\"".to_string())
            .replace("\"", "");

        conn.execute(
            "UPDATE workers SET status = ?1, resources = ?2, last_heartbeat = ?3 WHERE id = ?4",
            params![status_str, resources_json, last_heartbeat, id],
        )
        .map_err(AresError::db)?;
        Ok(())
    }

    pub fn list(&self) -> Result<Vec<Worker>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn.prepare("SELECT id, hostname, capabilities, labels, status, resources, registered_at, last_heartbeat FROM workers").map_err(AresError::db)?;

        let rows = stmt
            .query_map([], |row| {
                let capabilities_json: String = row.get(2)?;
                let labels_json: String = row.get(3)?;
                let status_str: String = row.get(4)?;
                let resources_json: String = row.get(5)?;

                let capabilities = serde_json::from_str(&capabilities_json).unwrap_or_default();
                let labels = serde_json::from_str(&labels_json).unwrap_or_default();
                let status = serde_json::from_str(&format!("\"{}\"", status_str))
                    .unwrap_or(WorkerStatus::Offline);
                let resources = serde_json::from_str(&resources_json).unwrap_or(
                    WorkerResources {
                        cpu: 0.0,
                        memory: 0,
                        disk: 0,
                        available_cpu: 0.0,
                        available_memory: 0,
                    },
                );

                Ok(Worker {
                    id: row.get(0)?,
                    hostname: row.get(1)?,
                    capabilities,
                    labels,
                    status,
                    resources,
                    registered_at: row.get(6)?,
                    last_heartbeat: row.get(7)?,
                })
            })
            .map_err(AresError::db)?;

        let mut workers = Vec::new();
        for r in rows {
            workers.push(r.map_err(AresError::db)?);
        }
        Ok(workers)
    }

    pub fn delete(&self, id: &str) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        conn.execute("DELETE FROM workers WHERE id = ?1", params![id])
            .map_err(AresError::db)?;
        Ok(())
    }
}
