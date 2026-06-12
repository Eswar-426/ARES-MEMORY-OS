use super::models::*;
use ares_core::AresError;
use ares_store::db::Store;
use rusqlite::params;

/// SQLite-backed repository for consolidation clusters and memberships.
pub struct ConsolidationRepository {
    store: Store,
}

impl ConsolidationRepository {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    /// Insert a memory cluster.
    pub fn insert_cluster(&self, cluster: &MemoryCluster) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        let tags_json = serde_json::to_string(&cluster.centroid_tags)
            .map_err(|e| AresError::Serialization(e.to_string()))?;

        conn.execute(
            "INSERT INTO memory_clusters (id, name, description, cluster_type, member_count,
             centroid_tags, summary, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                cluster.id,
                cluster.name,
                cluster.description,
                cluster.cluster_type.as_str(),
                cluster.member_count,
                tags_json,
                cluster.summary,
                cluster.created_at,
                cluster.updated_at,
            ],
        )
        .map_err(AresError::db)?;
        Ok(())
    }

    /// Add an episode to a cluster.
    pub fn add_membership(&self, membership: &ClusterMembership) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        conn.execute(
            "INSERT OR REPLACE INTO cluster_memberships (cluster_id, episode_id, similarity, added_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                membership.cluster_id,
                membership.episode_id,
                membership.similarity,
                membership.added_at,
            ],
        )
        .map_err(AresError::db)?;

        // Update member count
        conn.execute(
            "UPDATE memory_clusters SET member_count = (
                SELECT COUNT(*) FROM cluster_memberships WHERE cluster_id = ?1
             ) WHERE id = ?1",
            params![membership.cluster_id],
        )
        .map_err(AresError::db)?;
        Ok(())
    }

    /// Get a cluster by ID.
    pub fn get_cluster(&self, id: &str) -> Result<Option<MemoryCluster>, AresError> {
        let conn = self.store.get_conn()?;
        let result = conn.query_row(
            "SELECT id, name, description, cluster_type, member_count, centroid_tags,
                    summary, created_at, updated_at
             FROM memory_clusters WHERE id = ?1",
            params![id],
            Self::row_to_cluster,
        );
        match result {
            Ok(c) => Ok(Some(c)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AresError::db(e)),
        }
    }

    /// List all clusters.
    pub fn list_clusters(&self) -> Result<Vec<MemoryCluster>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, name, description, cluster_type, member_count, centroid_tags,
                        summary, created_at, updated_at
                 FROM memory_clusters ORDER BY updated_at DESC LIMIT 100",
            )
            .map_err(AresError::db)?;

        let rows = stmt
            .query_map([], Self::row_to_cluster)
            .map_err(AresError::db)?;

        let mut clusters = Vec::new();
        for row in rows {
            clusters.push(row.map_err(AresError::db)?);
        }
        Ok(clusters)
    }

    /// Get members of a cluster.
    pub fn get_members(&self, cluster_id: &str) -> Result<Vec<ClusterMembership>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT cluster_id, episode_id, similarity, added_at
                 FROM cluster_memberships WHERE cluster_id = ?1
                 ORDER BY similarity DESC",
            )
            .map_err(AresError::db)?;

        let rows = stmt
            .query_map(params![cluster_id], |row| {
                Ok(ClusterMembership {
                    cluster_id: row.get(0)?,
                    episode_id: row.get(1)?,
                    similarity: row.get(2)?,
                    added_at: row.get(3)?,
                })
            })
            .map_err(AresError::db)?;

        let mut members = Vec::new();
        for row in rows {
            members.push(row.map_err(AresError::db)?);
        }
        Ok(members)
    }

    /// Count clusters.
    pub fn count_clusters(&self) -> Result<u64, AresError> {
        let conn = self.store.get_conn()?;
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM memory_clusters", [], |row| row.get(0))
            .map_err(AresError::db)?;
        Ok(count as u64)
    }

    fn row_to_cluster(row: &rusqlite::Row<'_>) -> Result<MemoryCluster, rusqlite::Error> {
        let tags_str: String = row.get(5)?;
        Ok(MemoryCluster {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            cluster_type: ClusterType::from_str_val(&row.get::<_, String>(3)?),
            member_count: row.get(4)?,
            centroid_tags: serde_json::from_str(&tags_str).unwrap_or_default(),
            summary: row.get(6)?,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::test_store;
    use chrono::Utc;

    fn make_cluster(id: &str) -> MemoryCluster {
        let now = Utc::now().timestamp_micros();
        MemoryCluster {
            id: id.into(),
            name: format!("Cluster {}", id),
            description: "Test cluster".into(),
            cluster_type: ClusterType::Topic,
            member_count: 0,
            centroid_tags: vec!["test".into()],
            summary: "Test summary".into(),
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn insert_and_get_cluster() {
        let (store, _dir) = test_store();
        let repo = ConsolidationRepository::new(store);
        repo.insert_cluster(&make_cluster("cl_1")).unwrap();

        let fetched = repo.get_cluster("cl_1").unwrap().unwrap();
        assert_eq!(fetched.name, "Cluster cl_1");
    }

    #[test]
    fn get_nonexistent_cluster() {
        let (store, _dir) = test_store();
        let repo = ConsolidationRepository::new(store);
        assert!(repo.get_cluster("nope").unwrap().is_none());
    }

    #[test]
    fn add_and_get_members() {
        let (store, _dir) = test_store();
        let repo = ConsolidationRepository::new(store.clone());
        repo.insert_cluster(&make_cluster("cl_mem")).unwrap();

        // Need to insert episodes first for FK
        let ep_repo = crate::episodic::repository::EpisodeRepository::new(store);
        ep_repo
            .insert_episode(&crate::episodic::repository::make_test_episode(
                "ep_m1",
                crate::episodic::models::EpisodeOutcome::Success,
            ))
            .unwrap();

        let now = Utc::now().timestamp_micros();
        let membership = ClusterMembership {
            cluster_id: "cl_mem".into(),
            episode_id: "ep_m1".into(),
            similarity: 0.85,
            added_at: now,
        };
        repo.add_membership(&membership).unwrap();

        let members = repo.get_members("cl_mem").unwrap();
        assert_eq!(members.len(), 1);
        assert!((members[0].similarity - 0.85).abs() < f64::EPSILON);

        // Check that member_count was updated
        let cluster = repo.get_cluster("cl_mem").unwrap().unwrap();
        assert_eq!(cluster.member_count, 1);
    }

    #[test]
    fn list_clusters() {
        let (store, _dir) = test_store();
        let repo = ConsolidationRepository::new(store);
        repo.insert_cluster(&make_cluster("cl_l1")).unwrap();
        repo.insert_cluster(&make_cluster("cl_l2")).unwrap();

        let clusters = repo.list_clusters().unwrap();
        assert_eq!(clusters.len(), 2);
    }

    #[test]
    fn count_clusters() {
        let (store, _dir) = test_store();
        let repo = ConsolidationRepository::new(store);
        assert_eq!(repo.count_clusters().unwrap(), 0);
        repo.insert_cluster(&make_cluster("cl_c1")).unwrap();
        assert_eq!(repo.count_clusters().unwrap(), 1);
    }
}
