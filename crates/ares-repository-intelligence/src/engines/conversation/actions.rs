use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    OpenGraph,
    RunImpact,
    Traceability,
    SimulateRemoval,
    OpenFile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub action_type: ActionType,
    pub title: String,
    pub payload: serde_json::Value,
}

impl Action {
    pub fn open_graph(node_id: &str) -> Self {
        Self {
            action_type: ActionType::OpenGraph,
            title: format!("View {} in Graph", node_id),
            payload: serde_json::json!({ "node_id": node_id }),
        }
    }

    pub fn run_impact(node_id: &str) -> Self {
        Self {
            action_type: ActionType::RunImpact,
            title: format!("Run Impact Analysis on {}", node_id),
            payload: serde_json::json!({ "node_id": node_id }),
        }
    }

    pub fn traceability(node_id: &str) -> Self {
        Self {
            action_type: ActionType::Traceability,
            title: format!("View Traceability for {}", node_id),
            payload: serde_json::json!({ "node_id": node_id }),
        }
    }

    pub fn simulate_removal(node_id: &str) -> Self {
        Self {
            action_type: ActionType::SimulateRemoval,
            title: format!("Simulate Removal of {}", node_id),
            payload: serde_json::json!({ "node_id": node_id }),
        }
    }

    pub fn open_file(path: &str) -> Self {
        Self {
            action_type: ActionType::OpenFile,
            title: format!("Open {}", path),
            payload: serde_json::json!({ "path": path }),
        }
    }
}
