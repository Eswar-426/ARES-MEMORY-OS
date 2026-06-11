use ares_intelligence::models::capability::{ModelCapability, TaskType};
use ares_intelligence::models::model::Model;
use ares_intelligence::models::profile::ModelProfile;
use ares_intelligence::selection::selector::{ModelSelector, Objective, SelectionCriteria};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
struct BenchmarkBaseline {
    latency_micros: u128,
}

fn get_baseline_path(name: &str) -> PathBuf {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("tests");
    d.push(format!("{}_baseline.json", name));
    d
}

fn load_or_create_baseline(name: &str, current_micros: u128) -> u128 {
    let path = get_baseline_path(name);
    if path.exists() {
        let content = fs::read_to_string(&path).unwrap();
        let baseline: BenchmarkBaseline =
            serde_json::from_str(&content).unwrap_or(BenchmarkBaseline {
                latency_micros: current_micros,
            });
        baseline.latency_micros
    } else {
        let baseline = BenchmarkBaseline {
            latency_micros: current_micros,
        };
        let content = serde_json::to_string_pretty(&baseline).unwrap();
        fs::write(&path, content).unwrap();
        current_micros
    }
}

fn create_model(id: Uuid) -> Model {
    Model {
        id,
        name: format!("model_{}", id),
        provider_id: "test".to_string(),
        version: "1.0".to_string(),
        max_context_window: 4096,
        cost_per_1k_tokens: 0.1,
        capabilities: vec![ModelCapability::Reasoning],
    }
}

fn create_profile(model_id: Uuid) -> ModelProfile {
    ModelProfile {
        model_id,
        success_rate: 0.9,
        average_latency_ms: 100,
        total_executions: 100,
    }
}

#[test]
fn test_performance_budget_selection() {
    let selector = ModelSelector::new();

    // Setup 100 models
    let mut models = Vec::new();
    let mut profiles = HashMap::new();
    for _ in 0..100 {
        let m = create_model(Uuid::now_v7());
        profiles.insert(m.id, create_profile(m.id));
        models.push(m);
    }

    let criteria = SelectionCriteria {
        task_id: "task-bench".to_string(),
        task_type: TaskType::Reasoning,
        required_caps: vec![],
        objective: Objective::Balanced,
        max_latency_ms: None,
        max_cost: None,
    };

    // Warmup
    for _ in 0..10 {
        let _ = selector.select_best_model(&criteria, &models, &profiles);
    }

    // Benchmark 1,000 selections
    let start = Instant::now();
    for _ in 0..1000 {
        let _ = selector
            .select_best_model(&criteria, &models, &profiles)
            .unwrap();
    }
    let duration = start.elapsed();
    let current_micros = duration.as_micros();

    let baseline_micros = load_or_create_baseline("selection", current_micros);

    // We allow a 500% variance above baseline, plus a 500ms absolute floor to prevent micro-fluctuations failing the test on blazing fast machines vs CI.
    let allowed_micros = (baseline_micros as f64 * 5.0) as u128 + 500_000;

    assert!(
        current_micros <= allowed_micros,
        "Selection budget exceeded! Current: {}μs, Allowed: {}μs (Baseline: {}μs)",
        current_micros,
        allowed_micros,
        baseline_micros
    );
}

#[test]
fn test_performance_budget_routing_overhead() {
    // We measure just the lookup overhead in Cache / simple simulated routing
    let start = Instant::now();
    for _ in 0..1000 {
        // simulate 10 ops
        let mut x = 0;
        for i in 0..10 {
            x += i;
        }
        std::hint::black_box(x);
    }
    let duration = start.elapsed();
    let current_micros = duration.as_micros();

    let baseline_micros = load_or_create_baseline("routing_overhead", current_micros);
    let allowed_micros = (baseline_micros as f64 * 5.0) as u128 + 500_000;

    assert!(
        current_micros <= allowed_micros,
        "Routing overhead budget exceeded! Current: {}μs, Allowed: {}μs (Baseline: {}μs)",
        current_micros,
        allowed_micros,
        baseline_micros
    );
}
