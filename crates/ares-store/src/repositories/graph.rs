use crate::db::Store;
use ares_core::{
    new_id, types::event::now_micros, AresError, Contradiction, EdgeDirection, EdgeType, GraphEdge,
    GraphNode, ImpactEntry, ImpactGraph, NodeId, NodeType, ProjectId,
};
use rusqlite::params;

pub struct SqliteGraphRepository {
    store: Store,
}

impl SqliteGraphRepository {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    // ----------------------------------------------------------------
    // Upsert node — insert or update based on (project_id, node_type, label, file_path)
    // ----------------------------------------------------------------
    pub fn upsert_node(&self, node: GraphNode) -> Result<GraphNode, AresError> {
        let conn = self.store.get_conn()?;
        let now = now_micros();

        conn.execute(
            "INSERT INTO graph_nodes (id, project_id, node_type, label, properties, file_path, created_at, updated_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8)
             ON CONFLICT(id) DO UPDATE SET
               label      = excluded.label,
               properties = excluded.properties,
               updated_at = excluded.updated_at,
               deleted_at = NULL",
            params![
                node.id.as_str(),
                node.project_id.as_str(),
                node.node_type.as_str(),
                node.label,
                node.properties.to_string(),
                node.file_path,
                now,
                now,
            ],
        ).map_err(AresError::db)?;

        self.get_node(&node.id)?
            .ok_or_else(|| AresError::not_found("node", node.id.as_str()))
    }

    // ----------------------------------------------------------------
    // Upsert edge — enforces unique active edge constraint
    // ----------------------------------------------------------------
    pub fn upsert_edge(&self, edge: GraphEdge) -> Result<GraphEdge, AresError> {
        let conn = self.store.get_conn()?;
        let now = now_micros();

        // Expire existing active edge of same (from, to, type) before inserting new
        conn.execute(
            "UPDATE graph_edges SET valid_until = ?1
             WHERE from_node_id = ?2 AND to_node_id = ?3 AND edge_type = ?4 AND valid_until IS NULL",
            params![now, edge.from_node_id.as_str(), edge.to_node_id.as_str(), edge.edge_type.as_str()],
        ).map_err(AresError::db)?;

        let edge_id = new_id();
        conn.execute(
            "INSERT INTO graph_edges
               (id, project_id, from_node_id, to_node_id, edge_type, weight, confidence,
                source, valid_from, valid_until, created_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,NULL,?10)",
            params![
                edge_id,
                edge.project_id.as_str(),
                edge.from_node_id.as_str(),
                edge.to_node_id.as_str(),
                edge.edge_type.as_str(),
                edge.weight,
                edge.confidence,
                edge.source,
                now,
                now,
            ],
        )
        .map_err(AresError::db)?;

        Ok(GraphEdge {
            id: edge_id,
            valid_from: now,
            valid_until: None,
            created_at: now,
            ..edge
        })
    }

    // ----------------------------------------------------------------
    // Read
    // ----------------------------------------------------------------
    pub fn get_node(&self, id: &NodeId) -> Result<Option<GraphNode>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, project_id, node_type, label, properties, file_path,
                    created_at, updated_at, deleted_at
             FROM graph_nodes WHERE id = ?1 AND deleted_at IS NULL",
            )
            .map_err(AresError::db)?;

        let result = stmt.query_row(params![id.as_str()], row_to_node);
        match result {
            Ok(n) => Ok(Some(n)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AresError::db(e)),
        }
    }

    pub fn get_by_file_path(
        &self,
        project_id: &ProjectId,
        path: &str,
    ) -> Result<Vec<GraphNode>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, project_id, node_type, label, properties, file_path,
                    created_at, updated_at, deleted_at
             FROM graph_nodes
             WHERE project_id = ?1 AND file_path = ?2 AND deleted_at IS NULL",
            )
            .map_err(AresError::db)?;

        let rows = stmt
            .query_map(params![project_id.as_str(), path], row_to_node)
            .map_err(AresError::db)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(AresError::db)
    }

    pub fn list_nodes_paginated(
        &self,
        project_id: &ProjectId,
        node_type: Option<ares_core::NodeType>,
        search: Option<&str>,
        pagination: &ares_core::types::pagination::Pagination,
    ) -> Result<ares_core::types::pagination::Page<GraphNode>, AresError> {
        let conn = self.store.get_conn()?;

        let mut where_clauses = vec![
            "project_id = ?1".to_string(),
            "deleted_at IS NULL".to_string(),
        ];
        let mut bind_values: Vec<Box<dyn rusqlite::ToSql>> =
            vec![Box::new(project_id.as_str().to_string())];
        let mut idx = 2usize;

        if let Some(nt) = node_type {
            where_clauses.push(format!("node_type = ?{idx}"));
            bind_values.push(Box::new(nt.as_str().to_string()));
            idx += 1;
        }

        if let Some(q) = search {
            if !q.trim().is_empty() {
                where_clauses.push(format!(
                    "(label LIKE ?{idx} OR properties LIKE ?{idx} OR file_path LIKE ?{idx})"
                ));
                bind_values.push(Box::new(format!("%{}%", q)));
            }
        }
        let _ = idx; // suppress unused warning

        let where_sql = where_clauses.join(" AND ");

        // Count total
        let count_sql = format!("SELECT COUNT(*) FROM graph_nodes WHERE {where_sql}");
        let mut count_stmt = conn.prepare(&count_sql).map_err(AresError::db)?;
        let refs: Vec<&dyn rusqlite::ToSql> = bind_values.iter().map(|b| b.as_ref()).collect();
        let total: u64 = count_stmt
            .query_row(refs.as_slice(), |row| row.get(0))
            .map_err(AresError::db)?;

        // Fetch paginated
        let offset = pagination.offset();
        let limit = pagination.limit();
        let sql = format!(
            "SELECT id, project_id, node_type, label, properties, file_path,
                    created_at, updated_at, deleted_at
             FROM graph_nodes
             WHERE {where_sql}
             ORDER BY label ASC
             LIMIT {limit} OFFSET {offset}"
        );

        let mut stmt = conn.prepare(&sql).map_err(AresError::db)?;
        let rows = stmt
            .query_map(refs.as_slice(), row_to_node)
            .map_err(AresError::db)?;

        let items = rows.collect::<Result<Vec<_>, _>>().map_err(AresError::db)?;

        Ok(ares_core::types::pagination::Page::new(
            items,
            total,
            pagination.page,
            pagination.page_size,
        ))
    }

    // ----------------------------------------------------------------
    // Get neighbors
    // ----------------------------------------------------------------
    pub fn get_neighbors(
        &self,
        node_id: &NodeId,
        direction: EdgeDirection,
        edge_types: &[EdgeType],
    ) -> Result<Vec<GraphNode>, AresError> {
        let conn = self.store.get_conn()?;

        let type_placeholders: Vec<String> = (1..=edge_types.len())
            .map(|i| format!("?{}", i + 1))
            .collect();
        let types_sql = type_placeholders.join(",");

        let (join_col, node_col) = match direction {
            EdgeDirection::Outgoing => ("from_node_id", "to_node_id"),
            EdgeDirection::Incoming => ("to_node_id", "from_node_id"),
            EdgeDirection::Both => {
                // For Both, do two queries and merge
                let mut out = self.get_neighbors(node_id, EdgeDirection::Outgoing, edge_types)?;
                let inc = self.get_neighbors(node_id, EdgeDirection::Incoming, edge_types)?;
                let existing_ids: std::collections::HashSet<_> =
                    out.iter().map(|n| n.id.clone()).collect();
                for n in inc {
                    if !existing_ids.contains(&n.id) {
                        out.push(n);
                    }
                }
                return Ok(out);
            }
        };

        let sql = format!(
            "SELECT DISTINCT n.id, n.project_id, n.node_type, n.label, n.properties,
                    n.file_path, n.created_at, n.updated_at, n.deleted_at
             FROM graph_edges e
             JOIN graph_nodes n ON n.id = e.{node_col}
             WHERE e.{join_col} = ?1
               AND e.valid_until IS NULL
               AND e.edge_type IN ({types_sql})
               AND n.deleted_at IS NULL"
        );

        let mut stmt = conn.prepare(&sql).map_err(AresError::db)?;
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> =
            vec![Box::new(node_id.as_str().to_string())];
        for (i, et) in edge_types.iter().enumerate() {
            params_vec.push(Box::new(et.as_str().to_string()));
            let _ = i;
        }
        let refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();

        let rows = stmt
            .query_map(refs.as_slice(), row_to_node)
            .map_err(AresError::db)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(AresError::db)
    }

    pub fn get_edges_from(&self, node_id: &NodeId) -> Result<Vec<GraphEdge>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, project_id, from_node_id, to_node_id, edge_type, weight,
                    confidence, source, valid_from, valid_until, created_at
             FROM graph_edges
             WHERE from_node_id = ?1 AND valid_until IS NULL",
            )
            .map_err(AresError::db)?;

        let rows = stmt
            .query_map(params![node_id.as_str()], row_to_edge)
            .map_err(AresError::db)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(AresError::db)
    }

    pub fn get_edges_to(&self, node_id: &NodeId) -> Result<Vec<GraphEdge>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, project_id, from_node_id, to_node_id, edge_type, weight,
                    confidence, source, valid_from, valid_until, created_at
             FROM graph_edges
             WHERE to_node_id = ?1 AND valid_until IS NULL",
            )
            .map_err(AresError::db)?;

        let rows = stmt
            .query_map(params![node_id.as_str()], row_to_edge)
            .map_err(AresError::db)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(AresError::db)
    }

    // ----------------------------------------------------------------
    // Impact analysis — recursive CTE, max depth 5
    // ----------------------------------------------------------------
    pub fn traverse_impact(
        &self,
        start_node_id: &NodeId,
        depth: u8,
    ) -> Result<ImpactGraph, AresError> {
        let depth = depth.min(5); // hard cap
        let start = self
            .get_node(start_node_id)?
            .ok_or_else(|| AresError::not_found("node", start_node_id.as_str()))?;

        let conn = self.store.get_conn()?;
        let mut stmt = conn
            .prepare(
                "WITH RECURSIVE impact(node_id, depth, via_edge_type) AS (
               SELECT ?1, 0, ''
               UNION ALL
               SELECT e.to_node_id, i.depth + 1, e.edge_type
               FROM graph_edges e
               JOIN impact i ON e.from_node_id = i.node_id
               WHERE e.valid_until IS NULL
                 AND e.edge_type IN ('imports','depends_on','calls','implements','defines')
                 AND i.depth < ?2
             )
             SELECT DISTINCT n.id, n.project_id, n.node_type, n.label, n.properties,
                    n.file_path, n.created_at, n.updated_at, n.deleted_at,
                    MIN(i.depth) as min_depth,
                    GROUP_CONCAT(DISTINCT i.via_edge_type) as edge_types
             FROM impact i
             JOIN graph_nodes n ON n.id = i.node_id
             WHERE n.id != ?1 AND n.deleted_at IS NULL
             GROUP BY n.id
             ORDER BY min_depth ASC
             LIMIT 500",
            )
            .map_err(AresError::db)?;

        let rows = stmt
            .query_map(params![start_node_id.as_str(), depth as i64], |row| {
                let node = row_to_node(row)?;
                let dist: i64 = row.get(9)?;
                let edges_str: String = row.get(10)?;
                Ok((node, dist as u8, edges_str))
            })
            .map_err(AresError::db)?;

        let mut impacts = vec![];
        for row in rows {
            let (node, distance, edge_types_str) = row.map_err(AresError::db)?;
            let via_edges: Vec<EdgeType> = edge_types_str
                .split(',')
                .filter(|s| !s.is_empty())
                .filter_map(|s| s.parse().ok())
                .collect();
            // Confidence decays 0.1 per hop from 1.0
            let confidence = (1.0_f32 - (distance as f32 * 0.1)).max(0.1);
            impacts.push(ImpactEntry {
                node,
                distance,
                confidence,
                via_edges,
            });
        }

        Ok(ImpactGraph {
            target: start,
            impacts,
        })
    }

    // ----------------------------------------------------------------
    // Contradiction detection
    // ----------------------------------------------------------------
    pub fn detect_contradictions(
        &self,
        project_id: &ProjectId,
    ) -> Result<Vec<Contradiction>, AresError> {
        // Find nodes impacted by 2+ ACTIVE decisions
        let conn = self.store.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT n.id, n.label, GROUP_CONCAT(e.from_node_id) as decision_node_ids, COUNT(*) as cnt
             FROM graph_edges e
             JOIN graph_nodes n ON n.id = e.to_node_id
             JOIN graph_nodes dn ON dn.id = e.from_node_id
             WHERE dn.project_id = ?1
               AND dn.node_type = 'decision'
               AND e.edge_type = 'impacts'
               AND e.valid_until IS NULL
               AND n.deleted_at IS NULL
               AND dn.deleted_at IS NULL
             GROUP BY n.id
             HAVING cnt >= 2"
        ).map_err(AresError::db)?;

        let rows = stmt
            .query_map(params![project_id.as_str()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(AresError::db)?;

        let mut contradictions = vec![];
        for row in rows {
            let (_node_id, node_label, ids_str) = row.map_err(AresError::db)?;
            let decision_ids: Vec<String> = ids_str.split(',').map(String::from).collect();
            if decision_ids.len() >= 2 {
                contradictions.push(Contradiction {
                    source_id: NodeId::from(decision_ids[0].clone()),
                    target_id: NodeId::from(decision_ids[1].clone()),
                    reason: format!("Multiple decisions impact node '{node_label}'"),
                    confidence: 0.7,
                });
            }
        }
        Ok(contradictions)
    }

    /// Soft-delete all nodes for files that no longer exist (stale after incremental scan).
    pub fn delete_stale_file_nodes(
        &self,
        project_id: &ProjectId,
        stale_paths: &[String],
    ) -> Result<u32, AresError> {
        if stale_paths.is_empty() {
            return Ok(0);
        }
        let now = now_micros();
        let conn = self.store.get_conn()?;
        let mut total = 0u32;

        for path in stale_paths {
            let rows = conn
                .execute(
                    "UPDATE graph_nodes SET deleted_at = ?1, updated_at = ?1
                 WHERE project_id = ?2 AND file_path = ?3 AND deleted_at IS NULL",
                    params![now, project_id.as_str(), path],
                )
                .map_err(AresError::db)?;
            total += rows as u32;
        }
        Ok(total)
    }

    // ----------------------------------------------------------------
    // Scan State
    // ----------------------------------------------------------------
    pub fn get_scan_state(
        &self,
        project_id: &ProjectId,
        file_path: &str,
    ) -> Result<Option<String>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn
            .prepare("SELECT file_hash FROM scan_state WHERE project_id = ?1 AND file_path = ?2")
            .map_err(AresError::db)?;
        let result: Result<String, _> =
            stmt.query_row(params![project_id.as_str(), file_path], |row| row.get(0));
        match result {
            Ok(hash) => Ok(Some(hash)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AresError::db(e)),
        }
    }

    pub fn update_scan_state(
        &self,
        project_id: &ProjectId,
        file_path: &str,
        file_hash: &str,
        node_ids: &[NodeId],
    ) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        let now = now_micros();
        let ids_json = serde_json::to_string(node_ids).unwrap_or_else(|_| "[]".to_string());
        let id = new_id();

        conn.execute(
            "INSERT INTO scan_state (id, project_id, file_path, file_hash, last_scanned, node_ids)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(project_id, file_path) DO UPDATE SET
               file_hash = excluded.file_hash,
               last_scanned = excluded.last_scanned,
               node_ids = excluded.node_ids",
            params![id, project_id.as_str(), file_path, file_hash, now, ids_json,],
        )
        .map_err(AresError::db)?;
        Ok(())
    }

    // ----------------------------------------------------------------
    // Scan Runs
    // ----------------------------------------------------------------
    pub fn start_scan_run(
        &self,
        project_id: &ProjectId,
        run_type: &str,
    ) -> Result<String, AresError> {
        let conn = self.store.get_conn()?;
        let id = new_id();
        let now = now_micros();
        conn.execute(
            "INSERT INTO scan_runs (id, project_id, run_type, status, started_at)
             VALUES (?1, ?2, ?3, 'running', ?4)",
            params![id, project_id.as_str(), run_type, now],
        )
        .map_err(AresError::db)?;
        Ok(id)
    }

    pub fn complete_scan_run(
        &self,
        run_id: &str,
        status: &str,
        files_total: u32,
        files_parsed: u32,
        files_failed: u32,
    ) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        let now = now_micros();
        conn.execute(
            "UPDATE scan_runs SET status = ?1, files_total = ?2, files_parsed = ?3, files_failed = ?4, completed_at = ?5
             WHERE id = ?6",
            params![status, files_total, files_parsed, files_failed, now, run_id],
        ).map_err(AresError::db)?;
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────
// Row mapper
// ─────────────────────────────────────────────────────────────────

fn row_to_node(row: &rusqlite::Row<'_>) -> Result<GraphNode, rusqlite::Error> {
    let node_type_str: String = row.get(2)?;
    let props_str: String = row.get(4)?;

    Ok(GraphNode {
        id: NodeId::from(row.get::<_, String>(0)?),
        project_id: ProjectId::from(row.get::<_, String>(1)?),
        node_type: node_type_str.parse().unwrap_or(NodeType::Concept),
        label: row.get(3)?,
        properties: serde_json::from_str(&props_str).unwrap_or_default(),
        file_path: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
        deleted_at: row.get(8)?,
    })
}

fn row_to_edge(row: &rusqlite::Row<'_>) -> Result<GraphEdge, rusqlite::Error> {
    let edge_type_str: String = row.get(4)?;
    Ok(GraphEdge {
        id: row.get(0)?,
        project_id: ProjectId::from(row.get::<_, String>(1)?),
        from_node_id: NodeId::from(row.get::<_, String>(2)?),
        to_node_id: NodeId::from(row.get::<_, String>(3)?),
        edge_type: edge_type_str.parse().unwrap_or(EdgeType::RelatedTo),
        weight: row.get(5)?,
        confidence: row.get(6)?,
        source: row.get(7)?,
        valid_from: row.get(8)?,
        valid_until: row.get(9)?,
        created_at: row.get(10)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_helpers::test_store;
    use crate::repositories::project::SqliteProjectRepository;
    use ares_core::{Project, ProjectMaturity};

    fn setup_project(store: &Store) -> ProjectId {
        let now = now_micros();
        let project_id = ProjectId::new();
        let repo = SqliteProjectRepository::new(store.clone());
        repo.create(&Project {
            id: project_id.clone(),
            name: "graph-test".into(),
            description: "".into(),
            root_path: format!("/tmp/{}", new_id()),
            primary_language: "ts".into(),
            domain: "".into(),
            maturity: ProjectMaturity::Greenfield,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        })
        .unwrap();
        project_id
    }

    fn make_node(project_id: &ProjectId, label: &str, file_path: Option<&str>) -> GraphNode {
        let now = now_micros();
        GraphNode {
            id: NodeId::new(),
            project_id: project_id.clone(),
            node_type: NodeType::File,
            label: label.into(),
            properties: serde_json::json!({}),
            file_path: file_path.map(String::from),
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }

    #[test]
    fn upsert_and_get_node() {
        let (store, _dir) = test_store();
        let project_id = setup_project(&store);
        let repo = SqliteGraphRepository::new(store);
        let node = make_node(&project_id, "src/auth.ts", Some("src/auth.ts"));
        let upserted = repo.upsert_node(node).unwrap();
        assert_eq!(upserted.label, "src/auth.ts");

        let fetched = repo.get_node(&upserted.id).unwrap().unwrap();
        assert_eq!(fetched.id, upserted.id);
    }

    #[test]
    fn get_neighbors_returns_connected_nodes() {
        let (store, _dir) = test_store();
        let project_id = setup_project(&store);
        let repo = SqliteGraphRepository::new(store);

        let file_node = repo
            .upsert_node(make_node(&project_id, "auth.ts", Some("auth.ts")))
            .unwrap();
        let func_node = repo
            .upsert_node(make_node(&project_id, "validateJwt", None))
            .unwrap();

        repo.upsert_edge(GraphEdge {
            id: String::new(),
            project_id: project_id.clone(),
            from_node_id: file_node.id.clone(),
            to_node_id: func_node.id.clone(),
            edge_type: EdgeType::Defines,
            weight: 1.0,
            confidence: 1.0,
            source: "scanner".into(),
            valid_from: 0,
            valid_until: None,
            created_at: 0,
        })
        .unwrap();

        let neighbors = repo
            .get_neighbors(&file_node.id, EdgeDirection::Outgoing, &[EdgeType::Defines])
            .unwrap();
        assert_eq!(neighbors.len(), 1);
        assert_eq!(neighbors[0].id, func_node.id);
    }

    #[test]
    fn traverse_impact_returns_depth_correct_results() {
        let (store, _dir) = test_store();
        let project_id = setup_project(&store);
        let repo = SqliteGraphRepository::new(store);

        // A → imports → B → imports → C
        let a = repo
            .upsert_node(make_node(&project_id, "a.ts", Some("a.ts")))
            .unwrap();
        let b = repo
            .upsert_node(make_node(&project_id, "b.ts", Some("b.ts")))
            .unwrap();
        let c = repo
            .upsert_node(make_node(&project_id, "c.ts", Some("c.ts")))
            .unwrap();

        let make_edge = |from: &NodeId, to: &NodeId| GraphEdge {
            id: String::new(),
            project_id: project_id.clone(),
            from_node_id: from.clone(),
            to_node_id: to.clone(),
            edge_type: EdgeType::Imports,
            weight: 1.0,
            confidence: 1.0,
            source: "scanner".into(),
            valid_from: 0,
            valid_until: None,
            created_at: 0,
        };

        repo.upsert_edge(make_edge(&a.id, &b.id)).unwrap();
        repo.upsert_edge(make_edge(&b.id, &c.id)).unwrap();

        let impact = repo.traverse_impact(&a.id, 3).unwrap();
        assert_eq!(impact.impacts.len(), 2); // b and c
        assert_eq!(impact.impacts[0].distance, 1); // b at depth 1
        assert_eq!(impact.impacts[1].distance, 2); // c at depth 2
                                                   // Confidence decay
        assert!((impact.impacts[0].confidence - 0.9).abs() < 0.01);
        assert!((impact.impacts[1].confidence - 0.8).abs() < 0.01);
    }
}
