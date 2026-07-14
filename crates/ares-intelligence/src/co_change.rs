use ares_core::AresError;
use rusqlite::Connection;
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct HiddenCoupling {
    pub file_a: String,
    pub file_b: String,
    pub co_change_count: i64,
    pub period_days: i64,
    pub has_explicit_dependency: bool,
    pub risk: String,
}

pub fn detect_hidden_coupling(
    conn: &Connection,
    period_days: i64,
    min_co_changes: i64,
    limit: usize,
) -> Result<Vec<HiddenCoupling>, AresError> {
    let mut couplings = Vec::new();

    let sql = r#"
        WITH file_touches AS (
            SELECT e.to_node_id as file_id, e.from_node_id as toucher_id
            FROM graph_edges e
            JOIN graph_nodes n ON n.id = e.to_node_id
            WHERE e.edge_type = 'touches'
            AND n.node_type = 'file'
            AND n.file_path IS NOT NULL
            AND e.created_at > datetime('now', '-' || ?1 || ' days')
        ),
        file_pairs AS (
            SELECT
                f1.file_id as id_a,
                f2.file_id as id_b,
                COUNT(DISTINCT f1.toucher_id) as co_change_count
            FROM file_touches f1
            JOIN file_touches f2 ON f1.toucher_id = f2.toucher_id AND f1.file_id < f2.file_id
            GROUP BY f1.file_id, f2.file_id
            HAVING co_change_count > ?2
        )
        SELECT
            n1.file_path,
            n2.file_path,
            fp.co_change_count,
            CASE WHEN EXISTS (
                SELECT 1 FROM graph_edges e
                WHERE ((e.from_node_id = fp.id_a AND e.to_node_id = fp.id_b)
                    OR (e.from_node_id = fp.id_b AND e.to_node_id = fp.id_a))
                AND e.edge_type IN ('depends_on', 'imports', 'calls')
            ) THEN 1 ELSE 0 END
        FROM file_pairs fp
        JOIN graph_nodes n1 ON n1.id = fp.id_a
        JOIN graph_nodes n2 ON n2.id = fp.id_b
        ORDER BY fp.co_change_count DESC
        LIMIT ?3
    "#;

    if let Ok(mut stmt) = conn.prepare(sql) {
        let rows = stmt.query_map(
            rusqlite::params![period_days, min_co_changes, limit as i64],
            |row| {
                Ok((
                    row.get::<usize, String>(0)?,
                    row.get::<usize, String>(1)?,
                    row.get::<usize, i64>(2)?,
                    row.get::<usize, i64>(3)?,
                ))
            },
        ).map_err(|e| AresError::validation(e.to_string()))?;

        for row in rows.flatten() {
            let (file_a, file_b, count, has_dep) = row;
            couplings.push(HiddenCoupling {
                risk: format!(
                    "These files change together but have no declared dependency. Consider adding explicit coupling or investigating the relationship."
                ),
                file_a,
                file_b,
                co_change_count: count,
                period_days,
                has_explicit_dependency: has_dep == 1,
            });
        }
    }

    Ok(couplings)
}
