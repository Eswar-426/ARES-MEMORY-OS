use rusqlite::Connection;
use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub struct DecayResult {
    pub decay_score: f64,
    pub staleness: String,
}

/// Calculate how much a decision has decayed based on subsequent commits.
/// 0 commits since decision → 1.0 (fresh)
/// 20+ commits since decision → 0.0 (expired)
pub fn calculate_decision_decay(
    conn: &Connection,
    decision_created_at: &str,
    file_paths: &[String],
) -> DecayResult {
    if file_paths.is_empty() {
        return DecayResult {
            decay_score: 1.0,
            staleness: "fresh".to_string(),
        };
    }

    let mut total_commits: i64 = 0;
    for path in file_paths {
        if let Ok(count) = conn.query_row(
            "SELECT COUNT(*) FROM graph_edges e
             JOIN graph_nodes n ON n.id = e.to_node_id
             WHERE n.file_path = ?1
             AND e.edge_type = 'touches'
             AND e.created_at > ?2",
            rusqlite::params![path, decision_created_at],
            |row| row.get::<usize, i64>(0),
        ) {
            total_commits += count;
        }
    }

    let decay_score = (1.0 - (total_commits as f64 / 20.0)).max(0.0);

    let staleness = if decay_score >= 0.8 {
        "fresh"
    } else if decay_score >= 0.5 {
        "aging"
    } else if decay_score >= 0.2 {
        "stale"
    } else {
        "expired"
    };

    DecayResult {
        decay_score,
        staleness: staleness.to_string(),
    }
}

/// Count decisions that are stale or expired for health check gap detection.
pub fn count_decayed_decisions(conn: &Connection) -> Result<(i64, i64), String> {
    let mut stale_count: i64 = 0;
    let mut expired_count: i64 = 0;

    let sql = r#"
        SELECT d.id, d.created_at
        FROM graph_nodes d
        WHERE d.node_type = 'decision'
        AND d.created_at IS NOT NULL
    "#;

    if let Ok(mut stmt) = conn.prepare(sql) {
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<usize, String>(0)?,
                row.get::<usize, String>(1)?,
            ))
        }).map_err(|e| e.to_string())?;

        for row in rows.flatten() {
            let (_id, created_at) = row;

            // Get files linked to this decision
            let mut files: Vec<String> = Vec::new();
            if let Ok(mut f_stmt) = conn.prepare(
                "SELECT n.file_path FROM graph_edges e
                 JOIN graph_nodes n ON n.id = e.to_node_id
                 WHERE e.from_node_id = ?1
                 AND e.edge_type IN ('affects', 'relates_to')
                 AND n.node_type = 'file'
                 LIMIT 10"
            ) {
                if let Ok(f_rows) = f_stmt.query_map(rusqlite::params![_id], |r| r.get::<usize, String>(0)) {
                    for f in f_rows.flatten() {
                        files.push(f);
                    }
                }
            }

            let decay = calculate_decision_decay(conn, &created_at, &files);
            if decay.staleness == "expired" {
                expired_count += 1;
            } else if decay.staleness == "stale" {
                stale_count += 1;
            }
        }
    }

    Ok((stale_count, expired_count))
}

/// Enrich a JSON evidence array: add decay_score/staleness to decision-shaped items.
pub fn enrich_evidence_with_decay(
    conn: &Connection,
    evidence: &mut Vec<serde_json::Value>,
) {
    for item in evidence.iter_mut() {
        // Look for decision-shaped evidence: has "date" and "files" fields
        let date = item.get("date").and_then(|v| v.as_str());
        let files_val = item.get("files").or_else(|| item.get("file_paths"));

        if let (Some(date_str), Some(files_json)) = (date, files_val) {
            let files: Vec<String> = if files_json.is_array() {
                files_json
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            } else if files_json.is_string() {
                vec![files_json.as_str().unwrap().to_string()]
            } else {
                continue;
            };

            let decay = calculate_decision_decay(conn, date_str, &files);
            if let Some(obj) = item.as_object_mut() {
                obj.insert("decay_score".to_string(), serde_json::json!(decay.decay_score));
                obj.insert("staleness".to_string(), serde_json::json!(decay.staleness));
            }
        }
    }
}
