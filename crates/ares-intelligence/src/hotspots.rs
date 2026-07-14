use ares_core::AresError;
use rusqlite::{params, Connection};
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct Hotspot {
    pub path: String,
    pub hotspot_score: f64,
    pub commits_30_days: i64,
    pub complexity_proxy: i64,
    pub owner: String,
    pub recommendation: String,
}

pub fn calculate_hotspots(conn: &Connection, limit: usize) -> Result<Vec<Hotspot>, AresError> {
    let thirty_days_ago_micros = (chrono::Utc::now().timestamp() - (30 * 86400)) * 1_000_000;

    let mut raw_scores: Vec<(String, i64, i64, String)> = Vec::new();

    if let Ok(mut stmt) = conn.prepare(
        "SELECT
            f.file_path,
            COALESCE(churn.cnt, 0) as commits_30_days,
            COALESCE(comp.cnt, 0) + COALESCE(funcs.cnt, 0) as complexity_proxy,
            COALESCE(owner.author, 'unknown') as owner
         FROM graph_nodes f
         LEFT JOIN (
             SELECT e.to_node_id, COUNT(*) as cnt
             FROM graph_edges e
             WHERE e.edge_type = 'touches'
             AND e.created_at > ?1
             AND e.valid_until IS NULL
             GROUP BY e.to_node_id
         ) churn ON churn.to_node_id = f.id
         LEFT JOIN (
             SELECT e.from_node_id, COUNT(*) as cnt
             FROM graph_edges e
             WHERE e.edge_type IN ('depends_on', 'imports', 'calls')
             AND e.valid_until IS NULL
             GROUP BY e.from_node_id
         ) comp ON comp.from_node_id = f.id
         LEFT JOIN (
             SELECT file_path, COUNT(*) as cnt
             FROM graph_nodes
             WHERE node_type IN ('function', 'method')
             GROUP BY file_path
         ) funcs ON funcs.file_path = f.file_path
         LEFT JOIN (
             SELECT f2.file_path, p.label as author
             FROM graph_nodes f2
             JOIN graph_edges e_t ON e_t.to_node_id = f2.id
                 AND e_t.edge_type = 'touches' AND e_t.valid_until IS NULL
             JOIN graph_nodes c ON c.id = e_t.from_node_id AND c.node_type = 'commit'
             JOIN graph_edges e_a ON e_a.from_node_id = c.id
                 AND e_a.edge_type = 'authored_by' AND e_a.valid_until IS NULL
             JOIN graph_nodes p ON p.id = e_a.to_node_id
             GROUP BY f2.file_path
             ORDER BY COUNT(*) DESC
         ) owner ON owner.file_path = f.file_path
         WHERE f.node_type = 'file'
         AND f.file_path IS NOT NULL"
    ) {
        if let Ok(rows) = stmt.query_map(params![thirty_days_ago_micros], |row| {
            Ok((
                row.get::<usize, String>(0)?,
                row.get::<usize, i64>(1)?,
                row.get::<usize, i64>(2)?,
                row.get::<usize, String>(3)?,
            ))
        }) {
            for row in rows.flatten() {
                raw_scores.push(row);
            }
        }
    }

    if raw_scores.is_empty() {
        return Ok(Vec::new());
    }

    let max_churn: i64 = raw_scores.iter().map(|(_, c, _, _)| *c).max().unwrap_or(1);
    let max_complexity: i64 = raw_scores.iter().map(|(_, _, x, _)| *x).max().unwrap_or(1);

    let mut hotspots: Vec<Hotspot> = raw_scores
        .into_iter()
        .map(|(path, churn, complexity, owner)| {
            let churn_norm = churn as f64 / max_churn as f64;
            let comp_norm = complexity as f64 / max_complexity as f64;
            let score = churn_norm * comp_norm;

            let recommendation = if score > 0.8 {
                "Critical hotspot. High risk for bugs. Consider refactoring."
            } else if score > 0.6 {
                "Active hotspot. Monitor closely. Consider adding tests."
            } else if score > 0.4 {
                "Moderate hotspot. Watch for complexity growth."
            } else {
                "Low hotspot score."
            };

            Hotspot {
                path,
                hotspot_score: score,
                commits_30_days: churn,
                complexity_proxy: complexity,
                owner,
                recommendation: recommendation.to_string(),
            }
        })
        .collect();

    hotspots.sort_by(|a, b| b.hotspot_score.partial_cmp(&a.hotspot_score).unwrap_or(std::cmp::Ordering::Equal));
    hotspots.truncate(limit);

    Ok(hotspots)
}
