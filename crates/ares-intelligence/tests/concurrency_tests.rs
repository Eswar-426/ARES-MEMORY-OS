use ares_intelligence::learning::profiler::ModelProfiler;
use ares_intelligence::models::capability::{ModelCapability, TaskType};
use ares_intelligence::models::model::Model;

use ares_intelligence::selection::selector::{ModelSelector, Objective, SelectionCriteria};
use std::collections::HashMap;
use std::sync::Arc;
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

#[tokio::test]
async fn test_parallel_profile_updates() {
    let profiler = Arc::new(ModelProfiler::new(0.1, 0.1));
    let model_id = Uuid::now_v7();

    let mut handles = vec![];

    // Spawn 100 threads updating the same profile simultaneously
    for _ in 0..100 {
        let p = profiler.clone();
        let mid = model_id;
        handles.push(tokio::spawn(async move {
            p.update_success_rate(mid, TaskType::Reasoning, true);
            p.record_latency(mid, 200);
        }));
    }

    for h in handles {
        h.await.unwrap();
    }

    let profile = profiler.get_profile(model_id).unwrap();
    // 100 executions should be exactly recorded without dropping any
    assert_eq!(profile.total_executions, 100);
}

#[tokio::test]
async fn test_parallel_model_selections() {
    let selector = Arc::new(ModelSelector::new());

    let m1 = create_model(Uuid::now_v7(), 0.05);
    let m2 = create_model(Uuid::now_v7(), 0.01); // Cheapest
    let m3 = create_model(Uuid::now_v7(), 0.10);

    let models = Arc::new(vec![m1.clone(), m2.clone(), m3.clone()]);
    let profiles = Arc::new(HashMap::new());

    let mut handles = vec![];

    // 100 parallel selection requests
    for i in 0..100 {
        let sel = selector.clone();
        let mods = models.clone();
        let profs = profiles.clone();

        handles.push(tokio::spawn(async move {
            let criteria = SelectionCriteria {
                task_id: format!("task-{}", i),
                task_type: TaskType::Reasoning,
                required_caps: vec![],
                objective: Objective::Cheapest,
                max_latency_ms: None,
                max_cost: None,
            };

            let (selected, _) = sel.select_best_model(&criteria, &mods, &profs).unwrap();
            selected.id
        }));
    }

    for h in handles {
        let id = h.await.unwrap();
        // Should securely select the cheapest model under high concurrent load
        assert_eq!(id, m2.id);
    }
}
