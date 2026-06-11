use ares_intelligence::models::capability::{ModelCapability, TaskType};
use ares_intelligence::models::model::Model;
use ares_intelligence::models::profile::ModelProfile;
use ares_intelligence::selection::selector::{ModelSelector, Objective, SelectionCriteria};
use std::collections::HashMap;
use uuid::Uuid;

fn create_model(id: Uuid, cost: f64) -> Model {
    Model {
        id,
        name: format!("Model-{}", id),
        provider_id: "test_provider".to_string(),
        version: "1.0".to_string(),
        capabilities: vec![ModelCapability::Reasoning],
        max_context_window: 4096,
        cost_per_1k_tokens: cost,
    }
}

fn create_profile(model_id: Uuid, latency: u64, success: f64) -> ModelProfile {
    ModelProfile {
        model_id,
        success_rate: success,
        average_latency_ms: latency,
        total_executions: 100,
    }
}

#[test]
fn test_selection_cheapest() {
    let selector = ModelSelector::new();
    let m1 = create_model(Uuid::now_v7(), 0.05);
    let m2 = create_model(Uuid::now_v7(), 0.01); // Cheapest
    let m3 = create_model(Uuid::now_v7(), 0.10);

    let models = vec![m1.clone(), m2.clone(), m3.clone()];
    let profiles = HashMap::new();

    let criteria = SelectionCriteria {
        task_id: "task-1".to_string(),
        task_type: TaskType::Reasoning,
        required_caps: vec![],
        objective: Objective::Cheapest,
        max_latency_ms: None,
        max_cost: None,
    };

    let (selected, exp) = selector
        .select_best_model(&criteria, &models, &profiles)
        .unwrap();
    assert_eq!(selected.id, m2.id);
    assert_eq!(exp.selected_model_id, m2.id.to_string());
}

#[test]
fn test_selection_fastest() {
    let selector = ModelSelector::new();
    let m1 = create_model(Uuid::now_v7(), 0.05);
    let m2 = create_model(Uuid::now_v7(), 0.05);
    let m3 = create_model(Uuid::now_v7(), 0.05);

    let mut profiles = HashMap::new();
    profiles.insert(m1.id, create_profile(m1.id, 500, 0.9));
    profiles.insert(m2.id, create_profile(m2.id, 200, 0.9)); // Fastest
    profiles.insert(m3.id, create_profile(m3.id, 800, 0.9));

    let models = vec![m1.clone(), m2.clone(), m3.clone()];

    let criteria = SelectionCriteria {
        task_id: "task-1".to_string(),
        task_type: TaskType::Reasoning,
        required_caps: vec![],
        objective: Objective::Fastest,
        max_latency_ms: None,
        max_cost: None,
    };

    let (selected, _exp) = selector
        .select_best_model(&criteria, &models, &profiles)
        .unwrap();
    assert_eq!(selected.id, m2.id);
}

#[test]
fn test_selection_highest_quality() {
    let selector = ModelSelector::new();
    let m1 = create_model(Uuid::now_v7(), 0.05);
    let m2 = create_model(Uuid::now_v7(), 0.05);
    let m3 = create_model(Uuid::now_v7(), 0.05);

    let mut profiles = HashMap::new();
    profiles.insert(m1.id, create_profile(m1.id, 500, 0.8));
    profiles.insert(m2.id, create_profile(m2.id, 200, 0.5));
    profiles.insert(m3.id, create_profile(m3.id, 800, 0.99)); // Highest Quality

    let models = vec![m1.clone(), m2.clone(), m3.clone()];

    let criteria = SelectionCriteria {
        task_id: "task-1".to_string(),
        task_type: TaskType::Reasoning,
        required_caps: vec![],
        objective: Objective::HighestQuality,
        max_latency_ms: None,
        max_cost: None,
    };

    let (selected, _exp) = selector
        .select_best_model(&criteria, &models, &profiles)
        .unwrap();
    assert_eq!(selected.id, m3.id);
}

#[test]
fn test_selection_balanced() {
    let selector = ModelSelector::new();
    let m1 = create_model(Uuid::now_v7(), 0.10); // Expensive, good
    let m2 = create_model(Uuid::now_v7(), 0.02); // Cheap, very good -> Should win
    let m3 = create_model(Uuid::now_v7(), 0.01); // Very cheap, terrible

    let mut profiles = HashMap::new();
    profiles.insert(m1.id, create_profile(m1.id, 500, 0.95)); // Score: 0.95 / 0.1 = 9.5
    profiles.insert(m2.id, create_profile(m2.id, 200, 0.90)); // Score: 0.90 / 0.02 = 45.0
    profiles.insert(m3.id, create_profile(m3.id, 800, 0.10)); // Score: 0.10 / 0.01 = 10.0

    let models = vec![m1.clone(), m2.clone(), m3.clone()];

    let criteria = SelectionCriteria {
        task_id: "task-1".to_string(),
        task_type: TaskType::Reasoning,
        required_caps: vec![],
        objective: Objective::Balanced,
        max_latency_ms: None,
        max_cost: None,
    };

    let (selected, _exp) = selector
        .select_best_model(&criteria, &models, &profiles)
        .unwrap();
    assert_eq!(selected.id, m2.id);
}

#[test]
fn test_selection_no_candidates() {
    let selector = ModelSelector::new();
    let models = vec![];
    let profiles = HashMap::new();

    let criteria = SelectionCriteria {
        task_id: "task-1".to_string(),
        task_type: TaskType::Reasoning,
        required_caps: vec![],
        objective: Objective::Balanced,
        max_latency_ms: None,
        max_cost: None,
    };

    let res = selector.select_best_model(&criteria, &models, &profiles);
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err().to_string(),
        "No suitable model found for task"
    );
}

#[test]
fn test_selection_tie_break() {
    let selector = ModelSelector::new();
    let m1 = create_model(Uuid::now_v7(), 0.05);
    let m2 = create_model(Uuid::now_v7(), 0.05); // Tied for cheapest

    // Sort vector to make sure test fails if tie-break logic fails
    let mut models = vec![m1.clone(), m2.clone()];
    // Shuffle or insert randomly, but their IDs provide deterministic tie break.

    let criteria = SelectionCriteria {
        task_id: "task-1".to_string(),
        task_type: TaskType::Reasoning,
        required_caps: vec![],
        objective: Objective::Cheapest,
        max_latency_ms: None,
        max_cost: None,
    };

    let (selected, _) = selector
        .select_best_model(&criteria, &models, &HashMap::new())
        .unwrap();
    // Deterministic selection (should always be the one with smaller ID)
    let expected_id = if m1.id < m2.id { m1.id } else { m2.id };
    assert_eq!(selected.id, expected_id);

    // Swap order and re-verify
    models.reverse();
    let (selected2, _) = selector
        .select_best_model(&criteria, &models, &HashMap::new())
        .unwrap();
    assert_eq!(selected2.id, expected_id);
}

#[test]
fn test_selection_capability_filter() {
    let selector = ModelSelector::new();
    let mut m1 = create_model(Uuid::now_v7(), 0.01);
    m1.capabilities = vec![ModelCapability::Reasoning];

    let mut m2 = create_model(Uuid::now_v7(), 0.10);
    m2.capabilities = vec![ModelCapability::Reasoning, ModelCapability::Vision]; // Has vision

    let models = vec![m1.clone(), m2.clone()];

    let criteria = SelectionCriteria {
        task_id: "task-1".to_string(),
        task_type: TaskType::Reasoning,
        required_caps: vec![ModelCapability::Vision],
        objective: Objective::Cheapest, // m1 is cheapest but lacks capability
        max_latency_ms: None,
        max_cost: None,
    };

    let (selected, exp) = selector
        .select_best_model(&criteria, &models, &HashMap::new())
        .unwrap();
    assert_eq!(selected.id, m2.id);
    assert_eq!(exp.rejected_models.len(), 1);
    assert_eq!(exp.rejected_models[0].model_id, m1.id.to_string());
    assert!(exp.rejected_models[0].reason.contains("Missing"));
}

#[test]
fn test_selection_budget_enforcement() {
    let selector = ModelSelector::new();
    let m1 = create_model(Uuid::now_v7(), 0.10); // Too expensive
    let m2 = create_model(Uuid::now_v7(), 0.04); // Within budget
    let m3 = create_model(Uuid::now_v7(), 0.20); // Too expensive

    let models = vec![m1.clone(), m2.clone(), m3.clone()];

    let criteria = SelectionCriteria {
        task_id: "task-1".to_string(),
        task_type: TaskType::Reasoning,
        required_caps: vec![],
        objective: Objective::HighestQuality,
        max_latency_ms: None,
        max_cost: Some(0.05), // Max cost is 0.05
    };

    let (selected, exp) = selector
        .select_best_model(&criteria, &models, &HashMap::new())
        .unwrap();
    assert_eq!(selected.id, m2.id); // m2 is the only one within budget
    assert_eq!(exp.rejected_models.len(), 2);
}

#[test]
fn test_selection_latency_enforcement() {
    let selector = ModelSelector::new();
    let m1 = create_model(Uuid::now_v7(), 0.05);
    let m2 = create_model(Uuid::now_v7(), 0.05);

    let mut profiles = HashMap::new();
    profiles.insert(m1.id, create_profile(m1.id, 1000, 0.9)); // Too slow
    profiles.insert(m2.id, create_profile(m2.id, 200, 0.9)); // Fast enough

    let models = vec![m1.clone(), m2.clone()];

    let criteria = SelectionCriteria {
        task_id: "task-1".to_string(),
        task_type: TaskType::Reasoning,
        required_caps: vec![],
        objective: Objective::HighestQuality,
        max_latency_ms: Some(500), // Max latency 500ms
        max_cost: None,
    };

    let (selected, exp) = selector
        .select_best_model(&criteria, &models, &profiles)
        .unwrap();
    assert_eq!(selected.id, m2.id); // m2 is the only one fast enough
    assert_eq!(exp.rejected_models.len(), 1);
}
