use super::models::JobLease;
use ares_core::AresError;
use ares_store::db::Store;
use chrono::Utc;
use rusqlite::params;

pub struct LeaseRepository {
    store: Store,
}

impl LeaseRepository {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    pub fn acquire(&self, lease: &JobLease) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        conn.execute(
            "INSERT INTO job_leases (id, worker_id, queue_id, workflow_id, execution_id, acquired_at, expires_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                lease.id,
                lease.worker_id,
                lease.queue_id,
                lease.workflow_id,
                lease.execution_id,
                lease.acquired_at,
                lease.expires_at
            ],
        ).map_err(|e| {
            if e.to_string().contains("UNIQUE constraint failed: job_leases.queue_id") {
                AresError::conflict("Lease already active for this queue item")
            } else {
                AresError::db(e)
            }
        })?;
        Ok(())
    }

    pub fn renew(&self, lease_id: &str, new_expires_at: &str) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        conn.execute(
            "UPDATE job_leases SET expires_at = ?1 WHERE id = ?2",
            params![new_expires_at, lease_id],
        )
        .map_err(AresError::db)?;
        Ok(())
    }

    pub fn delete(&self, lease_id: &str) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        conn.execute("DELETE FROM job_leases WHERE id = ?1", params![lease_id])
            .map_err(AresError::db)?;
        Ok(())
    }

    pub fn find_expired(&self) -> Result<Vec<JobLease>, AresError> {
        let conn = self.store.get_conn()?;
        let now = Utc::now().to_rfc3339();

        let mut stmt = conn.prepare(
            "SELECT id, worker_id, queue_id, workflow_id, execution_id, acquired_at, expires_at 
             FROM job_leases WHERE expires_at < ?1"
        ).map_err(AresError::db)?;

        let rows = stmt
            .query_map(params![now], |row| {
                Ok(JobLease {
                    id: row.get(0)?,
                    worker_id: row.get(1)?,
                    queue_id: row.get(2)?,
                    workflow_id: row.get(3)?,
                    execution_id: row.get(4)?,
                    acquired_at: row.get(5)?,
                    expires_at: row.get(6)?,
                })
            })
            .map_err(AresError::db)?;

        let mut items = Vec::new();
        for r in rows {
            items.push(r.map_err(AresError::db)?);
        }
        Ok(items)
    }
}
