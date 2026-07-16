use ares_core::AresError;
use ares_store::Store;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct BriefingPackage {
    pub project: ProjectSnapshot,
    pub recent_activity: RecentActivity,
    pub agent_handoff: AgentHandoff,
    pub critical_gaps: Vec<String>,
    pub recommended_first_action: String,
    pub context_freshness_hours: f64,
    pub ares_version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectSnapshot {
    pub name: String,
    pub primary_language: String,
    pub architecture_summary: String,
    pub total_files: i64,
    pub total_functions: i64,
    pub health_score: f64,
    pub technology_stack: Vec<String>,
    pub key_modules: Vec<KeyModule>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyModule {
    pub path: String,
    pub purpose: String,
    pub owner: String,
    pub inbound_edges: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecentActivity {
    pub since_days: i64,
    pub commits_analyzed: i64,
    pub files_changed: Vec<String>,
    pub decisions_recorded: Vec<RecentDecision>,
    pub most_active_module: String,
    pub summary: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecentDecision {
    pub summary: String,
    pub files_affected: Vec<String>,
    pub date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentHandoff {
    pub sessions_available: i64,
    pub last_session: Option<LastSession>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LastSession {
    pub summary: String,
    pub files_touched: Vec<String>,
    pub decisions_made: Vec<String>,
    pub left_incomplete: String,
    pub project_name: String,
    pub recommended_next: String,
}

pub async fn generate_briefing(store: &Store, workspace_root: &str) -> Result<BriefingPackage, AresError> {
    let conn = store.get_conn()?;

    // ── Project Snapshot ──────────────────────────────────────
    let project_name = std::path::Path::new(workspace_root)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    let total_files: i64 = conn.query_row(
        "SELECT COUNT(*) FROM graph_nodes WHERE node_type = 'file'", [], |r| r.get(0)
    ).unwrap_or(0);

    let total_functions: i64 = conn.query_row(
        "SELECT COUNT(*) FROM graph_nodes WHERE node_type IN ('function', 'method')", [], |r| r.get(0)
    ).unwrap_or(0);

    // Technology stack from file extensions
    let mut ext_counts: HashMap<String, i64> = HashMap::new();
    if let Ok(mut stmt) = conn.prepare("SELECT file_path FROM graph_nodes WHERE node_type = 'file'") {
        if let Ok(rows) = stmt.query_map([], |row| row.get::<usize, String>(0)) {
            for path in rows.flatten() {
                if let Some(ext) = std::path::Path::new(&path).extension().and_then(|e| e.to_str()) {
                    *ext_counts.entry(ext.to_lowercase()).or_insert(0) += 1;
                }
            }
        }
    }
    let ext_map: HashMap<&str, &str> = [
        ("rs", "Rust"), ("py", "Python"), ("ts", "TypeScript"), ("tsx", "TypeScript/React"),
        ("js", "JavaScript"), ("jsx", "JavaScript/React"), ("go", "Go"), ("java", "Java"),
        ("c", "C"), ("cpp", "C++"), ("cc", "C++"), ("cxx", "C++"), ("h", "C/C++"),
        ("rb", "Ruby"), ("cs", "C#"), ("php", "PHP"), ("kt", "Kotlin"),
        ("toml", "TOML"), ("yaml", "YAML"), ("yml", "YAML"), ("json", "JSON"),
        ("md", "Markdown"), ("sql", "SQL"), ("ps1", "PowerShell")
    ].iter().copied().collect();

    // Primary language: most common file extension
    let primary_language = if let Some((ext, _)) = ext_counts.iter().max_by_key(|(_, c)| *c) {
        ext_map.get(ext.as_str()).copied().unwrap_or(ext).to_string()
    } else {
        "Unknown".to_string()
    };
    
    let mut sorted_exts: Vec<_> = ext_counts.iter().collect();
    sorted_exts.sort_by_key(|(_, &c)| std::cmp::Reverse(c));
    let mut tech_stack: Vec<String> = sorted_exts.into_iter()
        .filter_map(|(ext, _)| ext_map.get(ext.as_str()).map(|&s| s.to_string()))
        .collect();
    tech_stack.dedup();
    tech_stack.truncate(8);

    // Key modules: top 5 files by inbound edge count, plus function count for purpose
    let mut key_modules = Vec::new();
    if let Ok(mut stmt) = conn.prepare(
        "SELECT n.file_path, n.id, COUNT(e.id) as inbound FROM graph_nodes n LEFT JOIN graph_edges e ON e.to_node_id = n.id AND e.valid_until IS NULL WHERE n.node_type = 'file' AND n.file_path IS NOT NULL GROUP BY n.id ORDER BY inbound DESC LIMIT 5"
    ) {
        let rows = stmt.query_map([], |row| {
            let path: String = row.get(0)?;
            let id: String = row.get(1)?;
            let inbound: i64 = row.get(2)?;
            Ok((path, id, inbound))
        });
        if let Ok(rows) = rows {
            for row in rows.flatten() {
                let owner = get_owner_for_file(&conn, &row.0);
                
                // Count functions in this file for purpose
                let fn_count: i64 = conn.query_row(
                    "SELECT COUNT(*) FROM graph_edges e JOIN graph_nodes n ON e.to_node_id = n.id WHERE e.from_node_id = ?1 AND n.node_type IN ('function', 'method')",
                    [&row.1],
                    |r| r.get(0)
                ).unwrap_or(0);
                
                let purpose = format!("{} functions, {} inbound dependencies", fn_count, row.2);
                
                key_modules.push(KeyModule {
                    path: row.0,
                    purpose,
                    owner,
                    inbound_edges: row.2,
                });
            }
        }
    }

    let module_count: usize = conn.query_row(
        "SELECT COUNT(DISTINCT substr(file_path, 1, instr(file_path, '/'))) FROM graph_nodes WHERE node_type = 'file' AND file_path LIKE '%/%'",
        [], |r| r.get::<usize, usize>(0)
    ).unwrap_or(0);

    let repo = ares_store::repositories::gaps::SqliteGapRepository::new(store.clone());
    let project_id = ares_core::ProjectId::from(project_name.to_string());
    let health_score = repo.calculate_health_score(&project_id).map(|h| h.overall).unwrap_or(100.0);

    let architecture_summary = format!(
        "{} files, {} functions across {} modules. Primary language: {}. Health: {}/100.",
        total_files, total_functions, module_count, primary_language, health_score.round() as i32
    );

    // ── Recent Activity (7 days) ──────────────────────────────
    let since_days: i64 = 7;
    let since_ts = (chrono::Utc::now().timestamp() - (since_days * 86400)) * 1_000_000;

    let commits_analyzed: i64 = conn.query_row(
        "SELECT COUNT(*) FROM graph_nodes WHERE node_type = 'commit' AND created_at > ?1",
        [since_ts],
        |r| r.get(0)
    ).unwrap_or(0);

    let files_changed: Vec<String> = if let Ok(mut stmt) = conn.prepare("SELECT DISTINCT n.file_path FROM graph_nodes n JOIN graph_edges e ON (e.from_node_id = n.id OR e.to_node_id = n.id) JOIN graph_nodes c ON (e.from_node_id = c.id OR e.to_node_id = c.id) WHERE n.node_type = 'file' AND c.node_type = 'commit' AND e.edge_type = 'touches' AND c.created_at > ?1 ORDER BY n.file_path LIMIT 50") {
        if let Ok(rows) = stmt.query_map([since_ts], |r| r.get::<usize, String>(0)) {
            rows.flatten().collect()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // Most active module
    let most_active_module: String = conn.query_row(
        "SELECT n.file_path FROM graph_nodes n JOIN graph_edges e ON (e.from_node_id = n.id OR e.to_node_id = n.id) JOIN graph_nodes c ON (e.from_node_id = c.id OR e.to_node_id = c.id) WHERE n.node_type = 'file' AND c.node_type = 'commit' AND e.edge_type = 'touches' AND c.created_at > ?1 GROUP BY n.id ORDER BY COUNT(*) DESC LIMIT 1",
        [since_ts],
        |r| r.get::<usize, String>(0)
    ).unwrap_or_default();

    // Recent decisions
    let mut decisions_recorded: Vec<RecentDecision> = Vec::new();
    if let Ok(mut stmt) = conn.prepare(
        "SELECT d.label, group_concat(DISTINCT n.file_path), d.created_at FROM graph_nodes d LEFT JOIN graph_edges e ON e.from_node_id = d.id AND e.edge_type = 'related_to' LEFT JOIN graph_nodes n ON e.to_node_id = n.id WHERE d.node_type = 'decision' AND d.created_at > ?1 GROUP BY d.id ORDER BY d.created_at DESC LIMIT 10"
    ) {
        if let Ok(rows) = stmt.query_map([since_ts], |row| {
            let label: String = row.get(0)?;
            let paths_str: Option<String> = row.get(1)?;
            let created_at: i64 = row.get(2)?;
            let files: Vec<String> = paths_str.unwrap_or_default().split(',').filter(|s| !s.is_empty()).map(|s| s.to_string()).collect();
            let date = chrono::DateTime::from_timestamp(created_at, 0)
                .map(|dt| dt.format("%Y-%m-%d").to_string())
                .unwrap_or_default();
            Ok(RecentDecision { summary: label, files_affected: files, date })
        }) {
            for dec in rows.flatten() {
                decisions_recorded.push(dec);
            }
        }
    }

    let _recent_summary = if commits_analyzed > 0 {
        format!(
            "{} commits in the last {} days. {} files modified.",
            commits_analyzed, since_days,
            files_changed.len()
        )
    } else {
        "No commits in the last 7 days.".to_string()
    };

    // ── Agent Handoff ────────────────────────────────────────
    let sessions_available: i64 = conn.query_row(
        "SELECT COUNT(*) FROM agent_sessions",
        [], |r| r.get::<usize, i64>(0)
    ).unwrap_or(0);

    let last_session: Option<LastSession> = if sessions_available > 0 {
        conn.query_row(
            "SELECT summary, files_touched, tool_calls, left_incomplete, recommended_next, project_id FROM agent_sessions ORDER BY ended_at DESC LIMIT 1",
            [],
            |row| {
                let summary: String = row.get(0)?;
                let files_str: String = row.get(1).unwrap_or_default();
                let decs_str: String = row.get(2).unwrap_or_default();
                let incomplete: String = row.get(3).unwrap_or_default();
                let next: String = row.get(4).unwrap_or_default();
                let proj: String = row.get(5).unwrap_or_default();
                Ok(LastSession {
                    summary,
                    files_touched: files_str.split(',').filter(|s| !s.is_empty()).map(|s| s.to_string()).collect(),
                    decisions_made: decs_str.split(',').filter(|s| !s.is_empty()).map(|s| s.to_string()).collect(),
                    left_incomplete: incomplete,
                    recommended_next: next,
                    project_name: proj,
                })
            }
        ).ok()
    } else {
        None
    };

    // ── Critical Gaps ─────────────────────────────────────────
    let mut all_gaps = Vec::new();
    if let Ok(mut gaps) = repo.get_code_without_decision(&project_id, 30) { all_gaps.append(&mut gaps); }
    if let Ok(mut gaps) = repo.get_decisions_without_code(&project_id, 7) { all_gaps.append(&mut gaps); }
    if let Ok(mut gaps) = repo.get_orphaned_requirements(&project_id) { all_gaps.append(&mut gaps); }
    if let Ok(mut gaps) = repo.get_stale_decisions(&project_id, 30) { all_gaps.append(&mut gaps); }
    if let Ok(mut gaps) = repo.get_unknown_ownership(&project_id) { all_gaps.append(&mut gaps); }
    
    let critical_gaps: Vec<String> = all_gaps.into_iter().take(5).map(|g| g.details).collect();

    // ── Recommended First Action ──────────────────────────────
    let recommended_first_action = compute_recommended_action(
        health_score,
        commits_analyzed,
        &decisions_recorded,
        last_session.as_ref(),
        &most_active_module,
    );

    // ── Context Freshness ──────────────────────────────────────
    let context_freshness_hours: f64 = get_context_freshness(&conn);

    let files_changed_count: i64 = conn.query_row(
        "SELECT COUNT(DISTINCT n.file_path) FROM graph_nodes n JOIN graph_edges e ON (e.from_node_id = n.id OR e.to_node_id = n.id) JOIN graph_nodes c ON (e.from_node_id = c.id OR e.to_node_id = c.id) WHERE n.node_type = 'file' AND c.node_type = 'commit' AND e.edge_type = 'touches' AND c.created_at > ?1",
        [since_ts],
        |r| r.get(0)
    ).unwrap_or(0);

    Ok(BriefingPackage {
        project: ProjectSnapshot {
            name: project_name.to_string(),
            primary_language,
            architecture_summary,
            total_files,
            total_functions,
            health_score,
            technology_stack: tech_stack,
            key_modules,
        },
        recent_activity: RecentActivity {
            since_days,
            commits_analyzed,
            files_changed,
            decisions_recorded,
            most_active_module,
            summary: format!("{} commits analyzed, {} files modified in the last 7 days", commits_analyzed, files_changed_count),
        },
        agent_handoff: AgentHandoff {
            sessions_available,
            last_session,
        },
        critical_gaps,
        recommended_first_action,
        context_freshness_hours,
        ares_version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

fn get_owner_for_file(conn: &rusqlite::Connection, file_path: &str) -> String {
    // Try to find owner through: file → touches → commit → authored_by → person
    let file_id: String = conn.query_row(
        "SELECT id FROM graph_nodes WHERE file_path = ?1 AND node_type = 'file' LIMIT 1",
        [file_path],
        |r| r.get::<usize, String>(0)
    ).unwrap_or_default();

    let commit_ids: Vec<String> = if let Ok(mut stmt) = conn.prepare("SELECT DISTINCT from_node_id FROM graph_edges WHERE to_node_id = ?1 AND edge_type = 'touches' LIMIT 20") {
        if let Ok(rows) = stmt.query_map([&file_id], |r| r.get::<usize, String>(0)) {
            rows.flatten().collect()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    for cid in &commit_ids {
        if let Ok(person_id) = conn.query_row(
            "SELECT n.label FROM graph_edges e JOIN graph_nodes n ON e.to_node_id = n.id WHERE e.from_node_id = ?1 AND e.edge_type = 'authored_by' LIMIT 1",
            [cid],
            |r| r.get::<usize, String>(0)
        ) {
            return person_id;
        }
    }

    "Unknown".to_string()
}



fn compute_recommended_action(
    health_score: f64,
    commits_analyzed: i64,
    recent_decisions: &[RecentDecision],
    last_session: Option<&LastSession>,
    most_active_module: &str,
) -> String {
    if health_score < 60.0 {
        return format!(
            "Run ARES: Health Check — health score is {}/100. Critical gaps detected that need attention.",
            health_score.round()
        );
    }

    if commits_analyzed > 10 && recent_decisions.is_empty() {
        return format!(
            "Record decisions for recent changes — {} has {} commits in the last 7 days with no recorded decisions. Use ARES: Record Decision on the most active module.",
            most_active_module, commits_analyzed
        );
    }

    if let Some(session) = last_session {
        if !session.left_incomplete.is_empty() {
            return session.left_incomplete.clone();
        }
        if !session.recommended_next.is_empty() {
            return session.recommended_next.clone();
        }
    }

    if !most_active_module.is_empty() {
        return format!(
            "Review: {} — it's the most actively modified module in the last 7 days.",
            most_active_module
        );
    }

    "Run ARES: Health Check to see repository state.".to_string()
}

fn get_context_freshness(conn: &rusqlite::Connection) -> f64 {
    let max_ts: i64 = conn.query_row(
        "SELECT MAX(created_at) FROM graph_nodes",
        [], |r| r.get::<usize, i64>(0)
    ).unwrap_or(0);

    if max_ts == 0 { return 999.0; }
    
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_micros() as i64;
        
    let hours = (now - max_ts) as f64 / 3_600_000_000.0;
    if hours < 0.0 { 999.0 } else { hours }
}
