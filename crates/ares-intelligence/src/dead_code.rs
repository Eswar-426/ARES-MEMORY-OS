use ares_core::AresError;
use ares_store::Store;
use serde::Serialize;

#[derive(Serialize)]
pub struct DeadCodeReport {
    pub dead_files: Vec<DeadFile>,
    pub dead_functions: Vec<DeadFunction>,
    pub total_dead_files: i64,
    pub total_dead_functions: i64,
    pub estimated_removable_lines: i64,
    pub warning: String,
}

#[derive(Serialize)]
pub struct DeadFile {
    pub path: String,
    pub age_days: i64,
    pub language: String,
    pub recommendation: String,
}

#[derive(Serialize)]
pub struct DeadFunction {
    pub path: String,
    pub function_name: String,
    pub age_days: i64,
    pub recommendation: String,
}

pub async fn find_dead_code(
    store: &Store,
    threshold_days: i64,
) -> Result<DeadCodeReport, AresError> {
    let conn = store.get_conn()?;

    // --- Dead Files ---
    let mut dead_files = Vec::new();
    let mut stmt = conn.prepare(
        "SELECT n.file_path, n.created_at, NULL as node_type_detail
         FROM graph_nodes n
         WHERE n.node_type = 'file'
         AND n.created_at < datetime('now', '-' || ?1 || ' days')
         AND NOT EXISTS (
             SELECT 1 FROM graph_edges e
             WHERE e.to_node_id = n.id
             AND e.edge_type IN ('depends_on', 'imports', 'calls')
         )
         AND n.file_path NOT LIKE '%test%'
         AND n.file_path NOT LIKE '%spec%'
         AND n.file_path NOT LIKE '%__init__%'
         AND n.file_path NOT LIKE '%main%'
         AND n.file_path NOT LIKE '%config%'
         AND n.file_path NOT LIKE '%setup%'
         ORDER BY n.created_at ASC"
    ).map_err(|e| AresError::validation(e.to_string()))?;

    let rows = stmt.query_map(rusqlite::params![threshold_days], |row| {
        Ok((
            row.get::<usize, String>(0)?,
            row.get::<usize, String>(1)?,
            row.get::<usize, Option<String>>(2)?,
        ))
    }).map_err(|e| AresError::validation(e.to_string()))?;

    for row in rows {
        if let Ok((path, created_at, lang)) = row {
            let age_days = calculate_age_days(&created_at);
            let language = lang.unwrap_or_else(|| {
                path.rsplit('.').next().unwrap_or("unknown").to_string()
            });
            dead_files.push(DeadFile {
                path,
                age_days,
                language,
                recommendation: "Safe to delete if not a public API entry point".to_string(),
            });
        }
    }

    // --- Dead Functions ---
    let mut dead_functions = Vec::new();
    let excluded_names = [
        "main", "new", "default", "init", "setup",
        "test", "describe", "it", "expect",
    ];

    let mut stmt = conn.prepare(
        "SELECT n.file_path, n.label, n.created_at
         FROM graph_nodes n
         WHERE n.node_type IN ('function', 'method')
         AND n.created_at < datetime('now', '-' || ?1 || ' days')
         AND NOT EXISTS (
             SELECT 1 FROM graph_edges e
             WHERE e.to_node_id = n.id
             AND e.edge_type = 'calls'
         )
         AND n.label NOT IN ('main', 'new', 'default', 'init', 'setup',
                              'test', 'describe', 'it', 'expect')
         AND n.label NOT LIKE 'test_%'
         AND n.label NOT LIKE '%_test'
         ORDER BY n.file_path"
    ).map_err(|e| AresError::validation(e.to_string()))?;

    let rows = stmt.query_map(rusqlite::params![threshold_days], |row| {
        Ok((
            row.get::<usize, String>(0)?,
            row.get::<usize, String>(1)?,
            row.get::<usize, String>(2)?,
        ))
    }).map_err(|e| AresError::validation(e.to_string()))?;

    for row in rows {
        if let Ok((path, name, created_at)) = row {
            if excluded_names.contains(&name.as_str()) {
                continue;
            }
            let age_days = calculate_age_days(&created_at);
            dead_functions.push(DeadFunction {
                path,
                function_name: name,
                age_days,
                recommendation: "Consider removing or making private".to_string(),
            });
        }
    }

    let total_dead_files = dead_files.len() as i64;
    let total_dead_functions = dead_functions.len() as i64;

    // Estimate ~50 lines per dead file, ~20 lines per dead function
    let estimated_removable_lines = (total_dead_files * 50) + (total_dead_functions * 20);

    Ok(DeadCodeReport {
        dead_files,
        dead_functions,
        total_dead_files,
        total_dead_functions,
        estimated_removable_lines,
        warning: "Review each entry manually before deletion. ARES cannot detect reflection-based or dynamic calls.".to_string(),
    })
}

fn calculate_age_days(created_at: &str) -> i64 {
    // Try to parse the timestamp and calculate days
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(created_at) {
        let now = chrono::Utc::now();
        (now - dt.with_timezone(&chrono::Utc)).num_days()
    } else if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(created_at, "%Y-%m-%d %H:%M:%S") {
        let now = chrono::Utc::now().naive_utc();
        (now - dt).num_days()
    } else {
        0
    }
}
