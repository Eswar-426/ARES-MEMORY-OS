use ares_core::AresError;
use ares_store::Store;
use serde::Serialize;
use std::path::PathBuf;
use std::collections::HashMap;
use crate::hotspots::Hotspot;
use crate::co_change::HiddenCoupling;

#[derive(Serialize)]
pub struct ContextFile {
    pub project_name: String,
    pub timestamp: String,
    pub health_score: f64,
    pub what_this_project_is: String,
    pub technology_stack: Vec<String>,
    pub key_entry_points: Vec<EntryPoint>,
    pub critical_files: Vec<CriticalFile>,
    pub ownership: Vec<Owner>,
    pub recent_decisions: Vec<RecentDecision>,
    pub known_gaps: Vec<String>,
    pub mcp_tools: Vec<ToolEntry>,
    pub hotspots: Vec<Hotspot>,
    pub hidden_coupling: Vec<HiddenCoupling>,
}

#[derive(Serialize)]
pub struct EntryPoint {
    pub path: String,
    pub purpose: String,
    pub inbound_count: i64,
}

#[derive(Serialize)]
pub struct CriticalFile {
    pub path: String,
    pub outbound_count: i64,
}

#[derive(Serialize)]
pub struct Owner {
    pub name: String,
    pub file_percentage: f64,
}

#[derive(Serialize)]
pub struct RecentDecision {
    pub date: String,
    pub summary: String,
    pub author: String,
}

#[derive(Serialize)]
pub struct ToolEntry {
    pub name: String,
    pub description: String,
}

pub async fn generate_context_file(
    store: &Store,
    workspace_root: &str,
    project_id: &str,
    output_path: Option<&str>,
) -> Result<String, AresError> {
    let project_name = project_id.to_string();

    let conn = store.get_conn()?;
    let repo = ares_store::repositories::gaps::SqliteGapRepository::new(store.clone());
    let project_id = ares_core::ProjectId::from(project_name.to_string());

    // Get project overview stats
    let total_files: i64 = conn.query_row(
        "SELECT COUNT(*) FROM graph_nodes WHERE node_type = 'file'",
        [],
        |row| row.get::<usize, i64>(0),
    ).unwrap_or(0);

    let total_functions: i64 = conn.query_row(
        "SELECT COUNT(*) FROM graph_nodes WHERE node_type IN ('function', 'method')",
        [],
        |row| row.get::<usize, i64>(0),
    ).unwrap_or(0);

    let module_count: i64 = conn.query_row(
        "SELECT COUNT(DISTINCT substr(file_path, 1, instr(file_path, '/') - 1)) FROM graph_nodes WHERE node_type = 'file' AND instr(file_path, '/') > 0",
        [],
        |row| row.get::<usize, i64>(0),
    ).unwrap_or(0);

    // Get technology stack from file extensions
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

    let primary_language = if let Some((ext, _)) = ext_counts.iter().max_by_key(|(_, c)| *c) {
        ext_map.get(ext.as_str()).copied().unwrap_or(ext).to_string()
    } else {
        "Unknown".to_string()
    };

    let mut tech_stack: Vec<String> = ext_counts.keys()
        .filter_map(|ext| ext_map.get(ext.as_str()).map(|&s| s.to_string()))
        .collect();
    tech_stack.sort();
    tech_stack.dedup();
    if tech_stack.is_empty() {
        tech_stack.push("Unknown".to_string());
    }

    let health_score = repo.calculate_health_score(&project_id).map(|h| h.overall).unwrap_or(100.0);

    let mut entry_points = Vec::new();
    if let Ok(mut stmt) = conn.prepare(
        "SELECT n.file_path, n.label, COUNT(e.id) as inbound \
         FROM graph_nodes n \
         LEFT JOIN graph_edges e ON e.to_node_id = n.id AND e.valid_until IS NULL \
         WHERE n.node_type = 'file' AND n.file_path IS NOT NULL \
         GROUP BY n.id \
         ORDER BY inbound DESC LIMIT 5"
    ) {
        if let Ok(rows) = stmt.query_map([], |row| {
            Ok(EntryPoint {
                path: row.get::<usize, String>(0)?,
                purpose: row.get::<usize, Option<String>>(1)?.unwrap_or_default(),
                inbound_count: row.get::<usize, i64>(2)?,
            })
        }) {
            for row in rows.flatten() {
                entry_points.push(row);
            }
        }
    }

    let mut critical_files = Vec::new();
    if let Ok(mut stmt) = conn.prepare(
        "SELECT n.file_path, COUNT(e.id) as outbound \
         FROM graph_nodes n \
         LEFT JOIN graph_edges e ON e.from_node_id = n.id AND e.edge_type IN ('depends_on', 'imports', 'calls') AND e.valid_until IS NULL \
         WHERE n.node_type = 'file' AND n.file_path IS NOT NULL \
         AND (n.file_path LIKE '%.rs' OR n.file_path LIKE '%.py' OR n.file_path LIKE '%.ts' \
              OR n.file_path LIKE '%.tsx' OR n.file_path LIKE '%.js' OR n.file_path LIKE '%.jsx' \
              OR n.file_path LIKE '%.go' OR n.file_path LIKE '%.java' OR n.file_path LIKE '%.c' \
              OR n.file_path LIKE '%.cpp' OR n.file_path LIKE '%.cc' OR n.file_path LIKE '%.h' \
              OR n.file_path LIKE '%.rb' OR n.file_path LIKE '%.cs' OR n.file_path LIKE '%.php' \
              OR n.file_path LIKE '%.kt') \
         GROUP BY n.id \
         ORDER BY outbound DESC LIMIT 5"
    ) {
        if let Ok(rows) = stmt.query_map([], |row| {
            Ok(CriticalFile {
                path: row.get::<usize, String>(0)?,
                outbound_count: row.get::<usize, i64>(1)?,
            })
        }) {
            for row in rows.flatten() {
                critical_files.push(row);
            }
        }
    }

    let mut ownership = Vec::new();
    if let Ok(mut stmt) = conn.prepare("SELECT p.label as author, COUNT(DISTINCT n.id) as file_count, (COUNT(DISTINCT n.id) * 100.0 / (SELECT COUNT(*) FROM graph_nodes WHERE node_type = 'file')) as percentage FROM graph_nodes n JOIN graph_edges e1 ON (e1.from_node_id = n.id OR e1.to_node_id = n.id) AND e1.edge_type = 'touches' JOIN graph_nodes c ON (e1.from_node_id = c.id OR e1.to_node_id = c.id) AND c.node_type = 'commit' JOIN graph_edges e2 ON e2.from_node_id = c.id AND e2.edge_type = 'authored_by' JOIN graph_nodes p ON e2.to_node_id = p.id WHERE n.node_type = 'file' GROUP BY author ORDER BY file_count DESC LIMIT 3") {
        if let Ok(rows) = stmt.query_map([], |row| {
            Ok(Owner {
                name: row.get::<usize, String>(0)?,
                file_percentage: row.get::<usize, f64>(2)?,
            })
        }) {
            for row in rows.flatten() {
                ownership.push(row);
            }
        }
    }

    let mut recent_decisions = Vec::new();
    let thirty_days_ago = (chrono::Utc::now().timestamp() - (30 * 86400)) * 1_000_000;
    if let Ok(mut stmt) = conn.prepare("SELECT n.created_at, COALESCE(json_extract(n.metadata, '$.summary'), json_extract(n.metadata, '$.message'), n.label) as summary, COALESCE(json_extract(n.metadata, '$.author'), json_extract(n.metadata, '$.committer_name'), 'unknown') as author FROM graph_nodes n WHERE n.node_type IN ('decision', 'commit') AND n.created_at > ?1 ORDER BY n.created_at DESC LIMIT 10") {
        if let Ok(rows) = stmt.query_map([thirty_days_ago], |row| {
            let ts = row.get::<usize, i64>(0)?;
            let date = chrono::DateTime::from_timestamp_micros(ts as i64).map(|dt| dt.format("%Y-%m-%d").to_string()).unwrap_or_default();
            Ok(RecentDecision {
                date,
                summary: row.get::<usize, String>(1)?,
                author: row.get::<usize, String>(2)?,
            })
        }) {
            for row in rows.flatten() {
                recent_decisions.push(row);
            }
        }
    }

    let mut known_gaps = Vec::new();
    if let Ok(files) = repo.get_code_without_decision(&project_id, 10) {
        if !files.is_empty() {
            known_gaps.push(format!("{} files without recorded decisions. Use ARES: Record Decision.", files.len()));
        }
    }
    if let Ok(files) = repo.get_unknown_ownership(&project_id) {
        for f in files.iter().take(5) {
            known_gaps.push(f.details.clone());
        }
    }
    if let Ok(files) = repo.get_orphaned_requirements(&project_id) {
        if !files.is_empty() {
            known_gaps.push(format!("{} orphaned requirements.", files.len()));
        }
    }

    let hotspots = crate::hotspots::calculate_hotspots(&conn, 10).unwrap_or_default();
    let hidden_coupling = crate::co_change::detect_hidden_coupling(&conn, 90, 3, 20).unwrap_or_default();

    let mcp_tools = vec![
        ToolEntry { name: "ares_briefing".to_string(), description: "Full project state for new sessions".to_string() },
        ToolEntry { name: "ares_why_exists".to_string(), description: "Why any file or function exists".to_string() },
        ToolEntry { name: "ares_impact".to_string(), description: "What breaks if you change something".to_string() },
        ToolEntry { name: "ares_who_owns".to_string(), description: "Who is responsible for a file".to_string() },
        ToolEntry { name: "ares_decisions".to_string(), description: "Recorded architectural decisions".to_string() },
        ToolEntry { name: "ares_health_check".to_string(), description: "Repository memory health score".to_string() },
        ToolEntry { name: "ares_dead_code".to_string(), description: "Files and functions with no callers".to_string() },
        ToolEntry { name: "ares_simulate".to_string(), description: "What-if change impact prediction".to_string() },
        ToolEntry { name: "ares_record_decision".to_string(), description: "Record an architectural decision".to_string() },
        ToolEntry { name: "ares_generate_context_file".to_string(), description: "Generate a CLAUDE.md file for instant orientation".to_string() },
    ];

    let what_this_project_is = format!(
        "{} files, {} functions across {} modules. Primary language: {}.",
        total_files, total_functions, module_count, primary_language
    );

    let context = ContextFile {
        project_name: project_name.clone(),
        timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        health_score,
        what_this_project_is,
        technology_stack: tech_stack,
        key_entry_points: entry_points,
        critical_files,
        ownership,
        recent_decisions,
        known_gaps,
        hotspots,
        hidden_coupling,
        mcp_tools,
    };

    let markdown = generate_markdown(&context);

    let output = output_path.unwrap_or(".ares/CLAUDE.md");
    let full_path = PathBuf::from(workspace_root).join(output);

    if let Some(parent) = full_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| AresError::invalid_path(e.to_string()))?;
    }

    std::fs::write(&full_path, &markdown).map_err(|e| AresError::invalid_path(e.to_string()))?;

    Ok(full_path.to_string_lossy().to_string())
}

fn generate_markdown(ctx: &ContextFile) -> String {
    let mut md = String::new();

    md.push_str(&format!("# ARES Memory: {}\n\n", ctx.project_name));
    md.push_str("*Generated by ARES Memory OS — updated on every ingest*\n");
    md.push_str(&format!("*Last updated: {} | Health score: {:.0}/100*\n\n", ctx.timestamp, ctx.health_score));

    md.push_str("## What this project is\n\n");
    md.push_str(&ctx.what_this_project_is);
    md.push_str("\n\n");

    md.push_str("## Technology stack\n\n");
    md.push_str(&ctx.technology_stack.join(", "));
    md.push_str("\n\n");

    md.push_str("## Key entry points (start here)\n\n");
    if ctx.key_entry_points.is_empty() {
        md.push_str("No entry points detected.\n\n");
    } else {
        for ep in &ctx.key_entry_points {
            md.push_str(&format!("- `{}`", ep.path));
            if !ep.purpose.is_empty() {
                md.push_str(&format!(" — {}", ep.purpose));
            }
            md.push_str(&format!(" ({} inbound dependencies)\n", ep.inbound_count));
        }
        md.push_str("\n");
    }

    md.push_str("## Critical files (high blast radius — change carefully)\n\n");
    if ctx.critical_files.is_empty() {
        md.push_str("No critical files detected.\n\n");
    } else {
        for cf in &ctx.critical_files {
            md.push_str(&format!("- `{}` — {} dependencies\n", cf.path, cf.outbound_count));
        }
        md.push_str("\n");
    }

    md.push_str("## Hotspots (high churn × complexity — bug-prone)\n\n");
    if ctx.hotspots.is_empty() {
        md.push_str("No hotspots detected.\n\n");
    } else {
        for h in &ctx.hotspots {
            md.push_str(&format!(
                "- `{}` — score {:.2} ({} commits/30d, complexity {}) — {}\n",
                h.path, h.hotspot_score, h.commits_30_days, h.complexity_proxy, h.recommendation
            ));
        }
        md.push_str("\n");
    }

    if !ctx.hidden_coupling.is_empty() {
        md.push_str("## Hidden coupling detected\n\n");
        md.push_str("These files change together frequently but have no declared dependency.\n\n");
        for c in &ctx.hidden_coupling {
            md.push_str(&format!(
                "- `{}` + `{}` — {} co-changes in {} days\n",
                c.file_a, c.file_b, c.co_change_count, c.period_days
            ));
        }
        md.push_str("\n");
    }

    md.push_str("## Ownership\n\n");
    if ctx.ownership.is_empty() {
        md.push_str("No ownership information available.\n\n");
    } else {
        for owner in &ctx.ownership {
            md.push_str(&format!("- {} ({:.1}% of files)\n", owner.name, owner.file_percentage));
        }
        md.push_str("\n");
    }

    md.push_str("## Recent decisions (last 30 days)\n\n");
    if ctx.recent_decisions.is_empty() {
        md.push_str("No decisions recorded in last 30 days. Use ARES: Record Decision to capture architectural choices.\n\n");
    } else {
        for dec in &ctx.recent_decisions {
            md.push_str(&format!("- **{}** ({})\n", dec.summary, dec.date));
        }
        md.push_str("\n");
    }

    md.push_str("## Known knowledge gaps\n\n");
    if ctx.known_gaps.is_empty() {
        md.push_str("No gaps detected.\n\n");
    } else {
        for gap in &ctx.known_gaps {
            md.push_str(&format!("- {}\n", gap));
        }
        md.push_str("\n");
    }

    md.push_str("## ARES MCP tools available\n\n");
    md.push_str("| Tool | What it answers |\n");
    md.push_str("|------|----------------|\n");
    for tool in &ctx.mcp_tools {
        md.push_str(&format!("| `{}` | {} |\n", tool.name, tool.description));
    }
    md.push_str("\n");

    md.push_str("## How to query ARES\n\n");
    md.push_str("Ask: \"Run ares_why_exists on [file path]\"\n");
    md.push_str("Ask: \"Run ares_briefing to understand the current state\"\n");
    md.push_str("Ask: \"Run ares_impact on [file path] before modifying it\"\n");

    md
}
