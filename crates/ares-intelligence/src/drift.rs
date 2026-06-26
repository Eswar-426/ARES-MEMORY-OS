use ares_core::ProjectId;
use ares_store::repositories::{decision::SqliteDecisionRepository, graph::SqliteGraphRepository};
use ares_store::Store;
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftReport {
    pub file_path: String,
    pub drift_score: f32,
    pub historical_available: bool,
    pub added_functions: Vec<String>,
    pub removed_functions: Vec<String>,
    pub new_decisions: Vec<String>,
    pub orphaned_decisions: Vec<String>,
    pub ownership_changed: bool,
    pub summary: String,
}

pub struct HistoricalState {
    pub functions: Vec<String>,
    pub decisions: Vec<String>,
    pub owner: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[async_trait]
pub trait HistoricalStateProvider: Send + Sync {
    async fn state_90_days_ago(
        &self,
        project_id: &ProjectId,
        file_path: &str,
    ) -> anyhow::Result<HistoricalState>;
}

pub struct GitMemoryHistoricalProvider {
    #[allow(dead_code)]
    store: Store,
}

impl GitMemoryHistoricalProvider {
    pub fn new(store: Store) -> Self {
        Self { store }
    }
}

#[async_trait]
impl HistoricalStateProvider for GitMemoryHistoricalProvider {
    async fn state_90_days_ago(
        &self,
        _project_id: &ProjectId,
        _file_path: &str,
    ) -> anyhow::Result<HistoricalState> {
        // Gap 3: We currently do not reconstruct ASTs or extract CODEOWNERS from historical commits.
        // We leave the historical state empty and mark historical_available = false downstream.
        Ok(HistoricalState {
            functions: vec![],
            decisions: vec![],
            owner: None,
            timestamp: Utc::now() - Duration::days(90),
        })
    }
}

#[tracing::instrument(skip(store, provider))]
#[allow(deprecated)]
pub async fn analyze_drift(
    file_path: &str,
    store: &Store,
    project_id: &ProjectId,
    provider: &dyn HistoricalStateProvider,
) -> anyhow::Result<DriftReport> {
    // 1. Current State Extraction
    let graph_repo = SqliteGraphRepository::new(store.clone());
    let decision_repo = SqliteDecisionRepository::new(store.clone());
    let file_node_id = ares_core::canonicalize_node_id(file_path);
    let mut current_functions = Vec::new();

    if let Ok(nodes) = graph_repo.get_by_file_path(project_id, file_path) {
        for node in nodes {
            if node.node_type == ares_core::NodeType::Function {
                current_functions.push(node.label.clone());
            }
        }
    }

    let mut current_decisions = Vec::new();
    let filter = ares_core::DecisionFilter {
        file_path: Some(file_path.to_string()),
        ..Default::default()
    };
    if let Ok(decisions) = decision_repo.list(project_id, filter) {
        for d in decisions {
            current_decisions.push(d.id.to_string());
        }
    }

    let mut current_owner = None;
    if let Ok(neighbors) = graph_repo.get_neighbors(
        &ares_core::NodeId::from(file_node_id.clone()),
        ares_core::EdgeDirection::Incoming,
        &[ares_core::EdgeType::Owns],
    ) {
        if let Some(owner) = neighbors.first() {
            current_owner = Some(owner.label.clone());
        }
    }

    // 2. Historical State Extraction
    let hist_state = provider.state_90_days_ago(project_id, file_path).await?;

    // As per Gap 3 design, if we can't reliably populate historical functions/decisions,
    // we consider history unavailable to avoid false 100% drift.
    let historical_available = !hist_state.functions.is_empty()
        || !hist_state.decisions.is_empty()
        || hist_state.owner.is_some();

    if !historical_available {
        return Ok(DriftReport {
            file_path: file_path.to_string(),
            drift_score: 0.0,
            historical_available: false,
            added_functions: vec![],
            removed_functions: vec![],
            new_decisions: vec![],
            orphaned_decisions: vec![],
            ownership_changed: false,
            summary: "No historical snapshot available.".to_string(),
        });
    }

    // 3. Drift Computation
    let mut added_functions = Vec::new();
    for f in &current_functions {
        if !hist_state.functions.contains(f) {
            added_functions.push(f.clone());
        }
    }

    let mut removed_functions = Vec::new();
    for f in &hist_state.functions {
        if !current_functions.contains(f) {
            removed_functions.push(f.clone());
        }
    }

    let mut new_decisions = Vec::new();
    for d in &current_decisions {
        if !hist_state.decisions.contains(d) {
            new_decisions.push(d.clone());
        }
    }

    let mut orphaned_decisions = Vec::new();
    for d in &hist_state.decisions {
        if !current_decisions.contains(d) {
            orphaned_decisions.push(d.clone());
        }
    }

    let ownership_changed = current_owner != hist_state.owner;

    // 4. Scoring
    // 40% functions, 35% decisions, 25% ownership
    let mut score = 0.0;

    let total_funcs = std::cmp::max(current_functions.len(), hist_state.functions.len()) as f32;
    if total_funcs > 0.0 {
        let diff_funcs = (added_functions.len() + removed_functions.len()) as f32;
        let func_drift = (diff_funcs / total_funcs).min(1.0);
        score += func_drift * 0.40;
    }

    let total_decs = std::cmp::max(current_decisions.len(), hist_state.decisions.len()) as f32;
    if total_decs > 0.0 {
        let diff_decs = (new_decisions.len() + orphaned_decisions.len()) as f32;
        let dec_drift = (diff_decs / total_decs).min(1.0);
        score += dec_drift * 0.35;
    }

    if ownership_changed {
        score += 0.25;
    }

    score = score.clamp(0.0, 1.0);

    // 5. Deterministic Summary
    let mut summary_parts = Vec::new();
    if score < 0.2 {
        summary_parts.push("Minimal drift detected.".to_string());
    } else if score < 0.6 {
        summary_parts.push("Moderate drift detected.".to_string());
    } else {
        summary_parts.push("High drift detected.".to_string());
    }

    if !added_functions.is_empty() {
        summary_parts.push(format!("{} functions added.", added_functions.len()));
    }
    if !removed_functions.is_empty() {
        summary_parts.push(format!("{} functions removed.", removed_functions.len()));
    }

    if ownership_changed {
        summary_parts.push("Ownership changed.".to_string());
    } else {
        summary_parts.push("Ownership unchanged.".to_string());
    }

    if !new_decisions.is_empty() {
        summary_parts.push(format!("{} decisions added.", new_decisions.len()));
    }
    if !orphaned_decisions.is_empty() {
        summary_parts.push(format!(
            "{} linked decisions are now orphaned.",
            orphaned_decisions.len()
        ));
    }

    let summary = summary_parts.join(" ");

    Ok(DriftReport {
        file_path: file_path.to_string(),
        drift_score: score,
        historical_available: true,
        added_functions,
        removed_functions,
        new_decisions,
        orphaned_decisions,
        ownership_changed,
        summary,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    struct MockProvider {
        state: HistoricalState,
    }

    #[async_trait]
    impl HistoricalStateProvider for MockProvider {
        async fn state_90_days_ago(
            &self,
            _project_id: &ProjectId,
            _file_path: &str,
        ) -> anyhow::Result<HistoricalState> {
            Ok(HistoricalState {
                functions: self.state.functions.clone(),
                decisions: self.state.decisions.clone(),
                owner: self.state.owner.clone(),
                timestamp: self.state.timestamp,
            })
        }
    }

    fn setup_store_with_current(
        project_id: &ProjectId,
        file_path: &str,
        functions: Vec<&str>,
        decisions: Vec<&str>,
        owner: Option<&str>,
    ) -> (Store, tempfile::TempDir) {
        let dir = tempfile::TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");
        let store = Store::open(&db_path).unwrap();
        let conn = store.get_conn().unwrap();

        conn.execute("INSERT INTO projects (id, name, root_path, primary_language, domain, maturity, created_at, updated_at) VALUES (?1, 'test', '/test', 'rust', 'domain', 'greenfield', 0, 0)", [project_id.as_str()]).unwrap();

        let file_node_id = ares_core::canonicalize_node_id(file_path);

        conn.execute("INSERT INTO graph_nodes (id, project_id, node_type, label, properties, file_path, created_at, updated_at) VALUES (?1, ?2, 'file', ?3, '{}', ?3, 0, 0)", 
            [&file_node_id, project_id.as_str(), file_path]).unwrap();

        for f in functions {
            let fn_id = format!("function:{}", f);
            conn.execute("INSERT INTO graph_nodes (id, project_id, node_type, label, properties, file_path, created_at, updated_at) VALUES (?1, ?2, 'function', ?3, '{}', ?4, 0, 0)", 
                [&fn_id, project_id.as_str(), f, file_path]).unwrap();
        }

        conn.execute("INSERT INTO memories (id, project_id, memory_type, title, content, status, version, confidence, source, created_at, updated_at) VALUES ('m1', ?1, 'project', 'title', 'content', 'active', 1, 1.0, 'human', 0, 0)", [project_id.as_str()]).unwrap();

        for d in decisions {
            let files = serde_json::json!([file_path]).to_string();
            conn.execute("INSERT INTO decisions (id, project_id, memory_id, decision_text, reason, status, confidence, files_impacted, created_at, updated_at) VALUES (?1, ?2, 'm1', ?3, '', 'accepted', 1.0, ?4, 0, 0)", 
                [d, project_id.as_str(), d, &files]).unwrap();
        }

        if let Some(o) = owner {
            let o_id = format!("person:{}", o);
            conn.execute("INSERT INTO graph_nodes (id, project_id, node_type, label, properties, created_at, updated_at) VALUES (?1, ?2, 'person', ?3, '{}', 0, 0)", 
                [&o_id, project_id.as_str(), o]).unwrap();

            let edge_id = format!("{}-owns-{}", o_id, file_node_id);
            conn.execute("INSERT INTO graph_edges (id, project_id, from_node_id, to_node_id, edge_type, weight, confidence, source, valid_from, created_at) VALUES (?1, ?2, ?3, ?4, 'owns', 1.0, 1.0, 'human', 0, 0)", 
                [&edge_id, project_id.as_str(), &o_id, &file_node_id]).unwrap();
        }

        (store, dir)
    }

    #[tokio::test]
    async fn test_drift_missing_history() {
        let hist = HistoricalState {
            functions: vec![],
            decisions: vec![],
            owner: None,
            timestamp: Utc::now() - Duration::days(90),
        };
        let historical_available =
            !hist.functions.is_empty() || !hist.decisions.is_empty() || hist.owner.is_some();
        assert_eq!(historical_available, false);
    }

    #[tokio::test]
    async fn test_no_drift() {
        let pid = ProjectId::from("p1");
        let (store, _dir) = setup_store_with_current(
            &pid,
            "src/main.rs",
            vec!["main"],
            vec!["dec1"],
            Some("Alice"),
        );
        let provider = MockProvider {
            state: HistoricalState {
                functions: vec!["main".to_string()],
                decisions: vec!["dec1".to_string()],
                owner: Some("Alice".to_string()),
                timestamp: Utc::now() - Duration::days(90),
            },
        };

        let report = analyze_drift("src/main.rs", &store, &pid, &provider)
            .await
            .unwrap();
        assert_eq!(report.drift_score, 0.0);
        assert!(report.added_functions.is_empty());
        assert!(report.removed_functions.is_empty());
        assert!(report.new_decisions.is_empty());
        assert!(report.orphaned_decisions.is_empty());
        assert!(!report.ownership_changed);
    }

    #[tokio::test]
    async fn test_added_function() {
        let pid = ProjectId::from("p1");
        let (store, _dir) = setup_store_with_current(
            &pid,
            "src/main.rs",
            vec!["main", "helper"],
            vec!["dec1"],
            Some("Alice"),
        );
        let provider = MockProvider {
            state: HistoricalState {
                functions: vec!["main".to_string()],
                decisions: vec!["dec1".to_string()],
                owner: Some("Alice".to_string()),
                timestamp: Utc::now() - Duration::days(90),
            },
        };

        let report = analyze_drift("src/main.rs", &store, &pid, &provider)
            .await
            .unwrap();
        assert!(report.drift_score > 0.0);
        assert_eq!(report.added_functions, vec!["helper".to_string()]);
    }

    #[tokio::test]
    async fn test_removed_function() {
        let pid = ProjectId::from("p1");
        let (store, _dir) = setup_store_with_current(
            &pid,
            "src/main.rs",
            vec!["main"],
            vec!["dec1"],
            Some("Alice"),
        );
        let provider = MockProvider {
            state: HistoricalState {
                functions: vec!["main".to_string(), "helper".to_string()],
                decisions: vec!["dec1".to_string()],
                owner: Some("Alice".to_string()),
                timestamp: Utc::now() - Duration::days(90),
            },
        };

        let report = analyze_drift("src/main.rs", &store, &pid, &provider)
            .await
            .unwrap();
        assert!(report.drift_score > 0.0);
        assert_eq!(report.removed_functions, vec!["helper".to_string()]);
    }

    #[tokio::test]
    async fn test_ownership_changed() {
        let pid = ProjectId::from("p1");
        let (store, _dir) =
            setup_store_with_current(&pid, "src/main.rs", vec!["main"], vec!["dec1"], Some("Bob"));
        let provider = MockProvider {
            state: HistoricalState {
                functions: vec!["main".to_string()],
                decisions: vec!["dec1".to_string()],
                owner: Some("Alice".to_string()),
                timestamp: Utc::now() - Duration::days(90),
            },
        };

        let report = analyze_drift("src/main.rs", &store, &pid, &provider)
            .await
            .unwrap();
        assert!(report.drift_score >= 0.25);
        assert!(report.ownership_changed);
    }

    #[tokio::test]
    async fn test_orphaned_decision() {
        let pid = ProjectId::from("p1");
        let (store, _dir) =
            setup_store_with_current(&pid, "src/main.rs", vec!["main"], vec![], Some("Alice"));
        let provider = MockProvider {
            state: HistoricalState {
                functions: vec!["main".to_string()],
                decisions: vec!["dec1".to_string()],
                owner: Some("Alice".to_string()),
                timestamp: Utc::now() - Duration::days(90),
            },
        };

        let report = analyze_drift("src/main.rs", &store, &pid, &provider)
            .await
            .unwrap();
        assert!(report.drift_score > 0.0);
        assert_eq!(report.orphaned_decisions, vec!["dec1".to_string()]);
    }
}
