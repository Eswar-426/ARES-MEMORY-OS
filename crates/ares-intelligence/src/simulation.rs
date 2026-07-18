use ares_store::Store;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimulationAction {
    Remove,
    Add,
    Modify,
    Replace,
}

impl std::str::FromStr for SimulationAction {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "remove" => Ok(SimulationAction::Remove),
            "add" => Ok(SimulationAction::Add),
            "modify" => Ok(SimulationAction::Modify),
            "replace" => Ok(SimulationAction::Replace),
            _ => Err(anyhow::anyhow!("Unsupported simulation action: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    pub action: String,
    pub target: String,
    pub impact_radius: Vec<String>,
    pub decision_conflicts: Vec<String>,
    pub risk_score: f32,
    pub summary: String,
    pub reversible: bool,
}

#[derive(Debug, Default)]
struct ImpactGraph {
    affected_files: Vec<String>,
    affected_functions: Vec<String>,
    affected_tests: Vec<String>,
    affected_decisions: Vec<String>,
    #[allow(dead_code)]
    dependency_paths: Vec<Vec<String>>,
}

struct RiskFactors {
    dependency_weight: f32,
    decision_weight: f32,
    caller_weight: f32,
    architecture_weight: f32,
}

impl Default for RiskFactors {
    fn default() -> Self {
        Self {
            dependency_weight: 0.35,
            decision_weight: 0.30,
            caller_weight: 0.20,
            architecture_weight: 0.15,
        }
    }
}

fn summarize(action: SimulationAction, graph: &ImpactGraph) -> String {
    match action {
        SimulationAction::Remove => format!(
            "Removing this entity impacts {} files and {} decisions.",
            graph.affected_files.len(),
            graph.affected_decisions.len()
        ),
        SimulationAction::Add => format!(
            "Adding this dependency introduces {} architectural conflicts.",
            graph.affected_decisions.len()
        ),
        SimulationAction::Modify => format!(
            "Changing this signature affects {} callers and {} tests.",
            graph.affected_functions.len(),
            graph.affected_tests.len()
        ),
        SimulationAction::Replace => format!(
            "Replacing this technology affects {} files and requires {} migration effort.",
            graph.affected_files.len(),
            if graph.affected_files.len() > 20 {
                "significant"
            } else {
                "moderate"
            }
        ),
    }
}

#[tracing::instrument(skip(store))]
pub async fn simulate(
    action: SimulationAction,
    target: &str,
    _related: Option<&str>,
    store: &Store,
) -> anyhow::Result<SimulationResult> {
    // Use graph_repo.traverse_impact() — same filtered traversal as ares_impact
    // (only imports/depends_on/calls/implements, not contains/defines/touches)
    let graph_repo = ares_store::repositories::graph::SqliteGraphRepository::new(store.clone());
    let target_id = ares_core::NodeId::from(target);

    let mut impact = ImpactGraph::default();
    let mut risk_score = 0.0;
    let factors = RiskFactors::default();

    let reversible = match action {
        SimulationAction::Remove => {
            if let Ok(result) = graph_repo.traverse_impact(&target_id, 10) {
                for entry in &result.impacts {
                    let node_type_str = format!("{:?}", entry.node.node_type);
                    let label = entry.node.file_path.as_deref().unwrap_or(&entry.node.label).to_string();
                    if node_type_str == "Decision" || node_type_str == "Architecture" {
                        impact.affected_decisions.push(format!("[{}] {}", node_type_str, entry.node.label));
                    } else {
                        impact.affected_files.push(label);
                    }
                }
            }
            risk_score +=
                (impact.affected_files.len() as f32 * 0.05).clamp(0.0, factors.dependency_weight);
            risk_score +=
                (impact.affected_decisions.len() as f32 * 0.1).clamp(0.0, factors.decision_weight);
            false
        }
        SimulationAction::Add => {
            if let Ok(result) = graph_repo.traverse_impact(&target_id, 5) {
                for entry in &result.impacts {
                    let node_type_str = format!("{:?}", entry.node.node_type);
                    let label = entry.node.file_path.as_deref().unwrap_or(&entry.node.label).to_string();
                    if node_type_str == "Decision" || node_type_str == "Architecture" {
                        impact.affected_decisions.push(format!("[{}] {}", node_type_str, entry.node.label));
                    } else {
                        impact.affected_files.push(label);
                    }
                }
            }
            risk_score +=
                (impact.affected_files.len() as f32 * 0.05).clamp(0.0, factors.dependency_weight);
            risk_score += (impact.affected_decisions.len() as f32 * 0.15)
                .clamp(0.0, factors.architecture_weight + factors.decision_weight);
            impact.affected_decisions.is_empty()
        }
        SimulationAction::Modify => {
            if let Ok(result) = graph_repo.traverse_impact(&target_id, 5) {
                for entry in &result.impacts {
                    let node_type_str = format!("{:?}", entry.node.node_type);
                    if node_type_str == "Decision" || node_type_str == "Architecture" {
                        impact.affected_decisions.push(format!("[{}] {}", node_type_str, entry.node.label));
                    } else if node_type_str == "Test" {
                        impact.affected_tests.push(format!("[{}] {}", node_type_str, entry.node.label));
                    } else {
                        impact.affected_functions.push(format!("[{}] {}", node_type_str, entry.node.label));
                    }
                }
            }
            risk_score +=
                (impact.affected_functions.len() as f32 * 0.05).clamp(0.0, factors.caller_weight);
            risk_score +=
                (impact.affected_decisions.len() as f32 * 0.1).clamp(0.0, factors.decision_weight);
            false
        }
        SimulationAction::Replace => {
            if let Ok(result) = graph_repo.traverse_impact(&target_id, 10) {
                for entry in &result.impacts {
                    let node_type_str = format!("{:?}", entry.node.node_type);
                    let label = entry.node.file_path.as_deref().unwrap_or(&entry.node.label).to_string();
                    if node_type_str == "Decision" || node_type_str == "Architecture" {
                        impact.affected_decisions.push(format!("[{}] {}", node_type_str, entry.node.label));
                    } else {
                        impact.affected_files.push(label);
                    }
                }
            }
            risk_score +=
                (impact.affected_files.len() as f32 * 0.02).clamp(0.0, factors.dependency_weight);
            risk_score += (impact.affected_decisions.len() as f32 * 0.1)
                .clamp(0.0, factors.decision_weight + factors.architecture_weight);
            false
        }
    };

    let mut impact_radius = impact.affected_files.clone();
    impact_radius.extend(impact.affected_functions.clone());
    impact_radius.extend(impact.affected_tests.clone());
    impact_radius.sort();
    impact_radius.dedup();

    let mut decision_conflicts = impact.affected_decisions.clone();
    decision_conflicts.sort();
    decision_conflicts.dedup();

    let result = SimulationResult {
        action: match action {
            SimulationAction::Remove => "remove",
            SimulationAction::Add => "add",
            SimulationAction::Modify => "modify",
            SimulationAction::Replace => "replace",
        }
        .to_string(),
        target: target.to_string(),
        impact_radius,
        decision_conflicts,
        risk_score: risk_score.clamp(0.0, 1.0),
        summary: summarize(action, &impact),
        reversible,
    };

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ares_store::Store;
    use tempfile::TempDir;

    fn setup_test_db() -> (Store, TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let store = Store::open(&db_path).unwrap();

        let conn = store.get_conn().unwrap();
        // Insert a project and memory to satisfy foreign keys
        conn.execute("INSERT INTO projects (id, name, root_path, primary_language, domain, maturity, created_at, updated_at) VALUES ('p1', 'test', '/test', 'rust', 'domain', 'greenfield', 0, 0)", []).unwrap();
        conn.execute("INSERT INTO memories (id, project_id, memory_type, title, content, status, version, confidence, source, created_at, updated_at) VALUES ('m1', 'p1', 'project', 'title', 'content', 'active', 1, 1.0, 'human', 0, 0)", []).unwrap();

        // Insert some nodes for traversal
        conn.execute("INSERT INTO graph_entities (id, entity_type, name, properties, created_at, updated_at) VALUES ('target', 'code_artifact', 'target', '{}', 0, 0)", []).unwrap();
        conn.execute("INSERT INTO graph_entities (id, entity_type, name, properties, created_at, updated_at) VALUES ('file1', 'code_artifact', 'file1', '{}', 0, 0)", []).unwrap();
        conn.execute("INSERT INTO graph_entities (id, entity_type, name, properties, created_at, updated_at) VALUES ('decision1', 'decision', 'decision1', '{}', 0, 0)", []).unwrap();

        // Target -> File1 -> Decision1 (downstream)
        conn.execute("INSERT INTO graph_relationships (id, source_entity, target_entity, relationship_type, properties, created_at, updated_at) VALUES ('e1', 'target', 'file1', 'depends_on', '{}', 0, 0)", []).unwrap();
        conn.execute("INSERT INTO graph_relationships (id, source_entity, target_entity, relationship_type, properties, created_at, updated_at) VALUES ('e2', 'file1', 'decision1', 'motivated_by', '{}', 0, 0)", []).unwrap();

        // Target <- Test1 (upstream)
        conn.execute("INSERT INTO graph_entities (id, entity_type, name, properties, created_at, updated_at) VALUES ('test1', 'test', 'test1', '{}', 0, 0)", []).unwrap();
        conn.execute("INSERT INTO graph_relationships (id, source_entity, target_entity, relationship_type, properties, created_at, updated_at) VALUES ('e3', 'test1', 'target', 'references', '{}', 0, 0)", []).unwrap();

        (store, dir)
    }

    #[tokio::test]
    async fn test_simulate_remove() {
        let (store, _dir) = setup_test_db();
        let result = simulate(SimulationAction::Remove, "target", None, &store)
            .await
            .unwrap();
        assert_eq!(result.action, "remove");
        assert!(!result.reversible);
        assert!(result.impact_radius.contains(&"file1".to_string()));
    }

    #[tokio::test]
    async fn test_simulate_add() {
        let (store, _dir) = setup_test_db();
        let result = simulate(SimulationAction::Add, "target", None, &store)
            .await
            .unwrap();
        assert_eq!(result.action, "add");
        // Target has upstream test1
        assert!(result.impact_radius.contains(&"test1".to_string()));
    }

    #[tokio::test]
    async fn test_simulate_modify() {
        let (store, _dir) = setup_test_db();
        let result = simulate(SimulationAction::Modify, "target", None, &store)
            .await
            .unwrap();
        assert_eq!(result.action, "modify");
        assert!(result.impact_radius.contains(&"test1".to_string())); // Upstream test
        assert!(result.decision_conflicts.contains(&"decision1".to_string())); // Downstream decision
    }

    #[tokio::test]
    async fn test_simulate_replace() {
        let (store, _dir) = setup_test_db();
        let result = simulate(SimulationAction::Replace, "target", None, &store)
            .await
            .unwrap();
        assert_eq!(result.action, "replace");
        assert!(result.impact_radius.contains(&"file1".to_string()));
    }

    #[tokio::test]
    async fn test_multiple_actions() {
        let (store, _dir) = setup_test_db();

        let result_remove = simulate(SimulationAction::Remove, "target", None, &store)
            .await
            .unwrap();
        let result_modify = simulate(SimulationAction::Modify, "target", None, &store)
            .await
            .unwrap();
        let result_replace = simulate(SimulationAction::Replace, "target", None, &store)
            .await
            .unwrap();

        assert_eq!(result_remove.action, "remove");
        assert_eq!(result_modify.action, "modify");
        assert_eq!(result_replace.action, "replace");

        assert!(result_remove.risk_score >= 0.0);
        assert!(result_modify.risk_score >= 0.0);
        assert!(result_replace.risk_score >= 0.0);

        assert!(!result_remove.summary.is_empty());
        assert!(!result_modify.summary.is_empty());
        assert!(!result_replace.summary.is_empty());
    }

    #[test]
    fn test_simulation_action_parsing() {
        use std::str::FromStr;
        assert_eq!(
            SimulationAction::from_str("remove").unwrap(),
            SimulationAction::Remove
        );
        assert_eq!(
            SimulationAction::from_str("add").unwrap(),
            SimulationAction::Add
        );
        assert_eq!(
            SimulationAction::from_str("modify").unwrap(),
            SimulationAction::Modify
        );
        assert_eq!(
            SimulationAction::from_str("replace").unwrap(),
            SimulationAction::Replace
        );
        assert!(SimulationAction::from_str("invalid").is_err());
    }
}
