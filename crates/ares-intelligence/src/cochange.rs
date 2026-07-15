use ares_core::AresError;
use rusqlite::Connection;
use serde::Serialize;

/// A pair of files that change together frequently without explicit dependency.
#[derive(Debug, Clone, Serialize)]
pub struct CoChangePair {
    pub file_a: String,
    pub file_b: String,
    pub co_change_count: i64,
    pub period_days: i64,
    pub has_explicit_dependency: bool,
    pub risk: String,
}

/// Detect hidden coupling: file pairs that co-change but lack explicit dependency edges.
///
/// Logic:
/// 1. Self-join 'touches' edges on commit (from_node_id) to find file pairs
///    that appear in the same commit >= min_count times within period_days
/// 2. For each pair, check if ANY explicit dependency edge exists
///    (depends_on, imports, calls, uses) in either direction
/// 3. Return only pairs WITHOUT explicit dependency — these are hidden coupling
pub fn detect_hidden_coupling(
    conn: &Connection,
    min_count: i64,
    period_days: i64,
    limit: usize,
) -> Result<Vec<CoChangePair>, AresError> {
    let min_count = if min_count <= 0 { 3 } else { min_count };
    let period_days = if period_days <= 0 { 90 } else { period_days };
    let limit = if limit == 0 { 20 } else { limit };

    let cutoff_micros = (chrono::Utc::now().timestamp() - (period_days * 86400)) * 1_000_000;

    // Step 1: Find co-changing file pairs via self-join on commit node
    let pair_sql = r#"
        SELECT
            f1.id        AS id_a,
            f1.file_path AS path_a,
            f2.id        AS id_b,
            f2.file_path AS path_b,
            COUNT(DISTINCT e1.from_node_id) AS co_change_count
        FROM graph_edges e1
        INNER JOIN graph_edges e2
            ON e1.from_node_id = e2.from_node_id
            AND e1.to_node_id < e2.to_node_id
        INNER JOIN graph_nodes f1
            ON e1.to_node_id = f1.id
            AND f1.node_type = 'file'
            AND f1.file_path IS NOT NULL
        INNER JOIN graph_nodes f2
            ON e2.to_node_id = f2.id
            AND f2.node_type = 'file'
            AND f2.file_path IS NOT NULL
        WHERE e1.edge_type = 'touches'
          AND e2.edge_type = 'touches'
          AND e1.created_at > ?1
          AND e1.valid_until IS NULL
          AND e2.valid_until IS NULL
        GROUP BY f1.id, f2.id
        HAVING co_change_count >= ?2
        ORDER BY co_change_count DESC
        LIMIT ?3
    "#;

    let mut stmt = conn
        .prepare(pair_sql)
        .map_err(|e| AresError::Database(format!("co-change pair query: {}", e)))?;

    let rows = stmt
        .query_map(rusqlite::params![cutoff_micros, min_count, limit as i64], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
            ))
        })
        .map_err(|e| AresError::Database(format!("co-change iteration: {}", e)))?;

    // Step 2: Check each pair for explicit dependency edges
    // Includes 'uses' because scanner creates 'uses' edges (not just 'imports')
    let dep_sql = r#"
        SELECT COUNT(*) AS cnt
        FROM graph_edges
        WHERE (
            (from_node_id = ?1 AND to_node_id = ?2)
            OR
            (from_node_id = ?2 AND to_node_id = ?1)
        )
        AND edge_type IN ('depends_on', 'imports', 'calls', 'uses')
        AND valid_until IS NULL
    "#;

    let mut dep_stmt = conn
        .prepare(dep_sql)
        .map_err(|e| AresError::Database(format!("co-change dep check: {}", e)))?;

    let mut pairs: Vec<CoChangePair> = Vec::new();

    for row_result in rows {
        let (id_a, path_a, id_b, path_b, count) = row_result
            .map_err(|e| AresError::Database(format!("co-change row read: {}", e)))?;

        // Skip noise files that change with everything
        let noise_suffixes = [
            "Cargo.toml", "Cargo.lock", "package.json", "package-lock.json",
            "yarn.lock", "pnpm-lock.yaml", "go.sum", "go.mod",
            ".gitignore", "README.md", "CHANGELOG.md", "LICENSE",
        ];
        let a_name = path_a.rsplit('/').next().unwrap_or(&path_a);
        let b_name = path_b.rsplit('/').next().unwrap_or(&path_b);
        if noise_suffixes.contains(&a_name) || noise_suffixes.contains(&b_name) {
            continue;
        }

        let dep_count: i64 = dep_stmt
            .query_row(rusqlite::params![id_a, id_b], |row| row.get(0))
            .unwrap_or(0);

        // Only report pairs WITHOUT any explicit dependency
        if dep_count == 0 {
            pairs.push(CoChangePair {
                file_a: path_a,
                file_b: path_b,
                co_change_count: count,
                period_days,
                has_explicit_dependency: false,
                risk: format!(
                    "These files changed together {} times in {} days but have \
                     no declared dependency. Consider adding explicit coupling \
                     or investigating the relationship.",
                    count, period_days
                ),
            });
        }
    }

    Ok(pairs)
}
