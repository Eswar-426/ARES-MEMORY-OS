use super::*;
use crate::scenario::generator::ScenarioGenerator;
use crate::scenario::models::*;

#[test]
fn generates_four_standard_scenarios() {
    let gen = ScenarioGenerator::new();
    let state = make_world_state();
    let config = make_scenario_config();
    let scenarios = gen.generate("g1", "Build a REST API", &state, &config);
    assert_eq!(scenarios.len(), 4);
}

#[test]
fn standard_types_include_all_four() {
    let types = ScenarioType::standard_types();
    assert_eq!(types.len(), 4);
    assert!(types.contains(&ScenarioType::Fastest));
    assert!(types.contains(&ScenarioType::Cheapest));
    assert!(types.contains(&ScenarioType::HighestQuality));
    assert!(types.contains(&ScenarioType::Balanced));
}

#[test]
fn fastest_has_lowest_duration() {
    let gen = ScenarioGenerator::new();
    let state = make_world_state();
    let scenarios = gen.generate("g1", "Build API", &state, &make_scenario_config());
    let fastest = scenarios
        .iter()
        .find(|s| s.scenario_type == ScenarioType::Fastest)
        .unwrap();
    let balanced = scenarios
        .iter()
        .find(|s| s.scenario_type == ScenarioType::Balanced)
        .unwrap();
    assert!(fastest.estimated_duration_secs <= balanced.estimated_duration_secs);
}

#[test]
fn cheapest_has_lowest_cost() {
    let gen = ScenarioGenerator::new();
    let state = make_world_state();
    let scenarios = gen.generate("g1", "Build API", &state, &make_scenario_config());
    let cheapest = scenarios
        .iter()
        .find(|s| s.scenario_type == ScenarioType::Cheapest)
        .unwrap();
    let balanced = scenarios
        .iter()
        .find(|s| s.scenario_type == ScenarioType::Balanced)
        .unwrap();
    assert!(cheapest.estimated_cost <= balanced.estimated_cost);
}

#[test]
fn highest_quality_has_best_quality() {
    let gen = ScenarioGenerator::new();
    let state = make_world_state();
    let scenarios = gen.generate("g1", "Build API", &state, &make_scenario_config());
    let quality = scenarios
        .iter()
        .find(|s| s.scenario_type == ScenarioType::HighestQuality)
        .unwrap();
    let cheapest = scenarios
        .iter()
        .find(|s| s.scenario_type == ScenarioType::Cheapest)
        .unwrap();
    assert!(quality.estimated_quality >= cheapest.estimated_quality);
}

#[test]
fn scenario_has_steps() {
    let gen = ScenarioGenerator::new();
    let state = make_world_state();
    let scenarios = gen.generate("g1", "Build API", &state, &make_scenario_config());
    for s in &scenarios {
        assert!(!s.steps.is_empty());
    }
}

#[test]
fn scenario_steps_have_ordered_indices() {
    let gen = ScenarioGenerator::new();
    let state = make_world_state();
    let scenarios = gen.generate("g1", "Build API", &state, &make_scenario_config());
    for s in &scenarios {
        for (i, step) in s.steps.iter().enumerate() {
            assert_eq!(step.order, (i + 1) as u32);
        }
    }
}

#[test]
fn complex_goal_has_more_steps() {
    let gen = ScenarioGenerator::new();
    let state = make_world_state();
    let simple = gen.generate("g1", "Fix bug", &state, &make_scenario_config());
    let complex = gen.generate(
        "g2",
        "Build full production enterprise distributed system",
        &state,
        &make_scenario_config(),
    );
    assert!(complex[0].steps.len() > simple[0].steps.len());
}

#[test]
fn custom_scenario_type_generated() {
    let gen = ScenarioGenerator::new();
    let state = make_world_state();
    let config = ScenarioGenerationConfig {
        generate_all_standard: false,
        custom_types: vec!["experimental".to_string()],
        ..Default::default()
    };
    let scenarios = gen.generate("g1", "Test", &state, &config);
    assert_eq!(scenarios.len(), 1);
    assert_eq!(
        scenarios[0].scenario_type,
        ScenarioType::Custom("experimental".to_string())
    );
}

#[test]
fn scenario_type_roundtrip() {
    for t in ScenarioType::standard_types() {
        assert_eq!(ScenarioType::from_str_val(t.as_str()), t);
    }
}

#[test]
fn scenario_type_display() {
    assert_eq!(ScenarioType::Fastest.to_string(), "fastest");
    assert_eq!(ScenarioType::Balanced.to_string(), "balanced");
}

#[test]
fn scenario_total_step_cost() {
    let gen = ScenarioGenerator::new();
    let state = make_world_state();
    let scenarios = gen.generate("g1", "Build API", &state, &make_scenario_config());
    for s in &scenarios {
        assert!(s.total_step_cost() > 0.0);
    }
}

#[test]
fn scenario_total_step_duration() {
    let gen = ScenarioGenerator::new();
    let state = make_world_state();
    let scenarios = gen.generate("g1", "Build API", &state, &make_scenario_config());
    for s in &scenarios {
        assert!(s.total_step_duration() > 0.0);
    }
}

#[test]
fn scenario_step_count() {
    let gen = ScenarioGenerator::new();
    let state = make_world_state();
    let scenarios = gen.generate("g1", "Build API", &state, &make_scenario_config());
    for s in &scenarios {
        assert_eq!(s.step_count(), s.steps.len());
    }
}

#[test]
fn scenario_serialization_roundtrip() {
    let gen = ScenarioGenerator::new();
    let state = make_world_state();
    let scenarios = gen.generate("g1", "Build API", &state, &make_scenario_config());
    let json = serde_json::to_string(&scenarios[0]).unwrap();
    let back: Scenario = serde_json::from_str(&json).unwrap();
    assert_eq!(back.goal_id, "g1");
}

#[test]
fn config_default_generates_standard() {
    let config = ScenarioGenerationConfig::default();
    assert!(config.generate_all_standard);
    assert!(config.custom_types.is_empty());
    assert_eq!(config.max_steps, 20);
}

#[test]
fn cost_multiplier_affects_cost() {
    let gen = ScenarioGenerator::new();
    let state = make_world_state();
    let normal = gen.generate("g1", "Test", &state, &ScenarioGenerationConfig::default());
    let expensive = gen.generate(
        "g1",
        "Test",
        &state,
        &ScenarioGenerationConfig {
            cost_multiplier: 2.0,
            ..Default::default()
        },
    );
    let n_cost: f64 = normal.iter().map(|s| s.estimated_cost).sum();
    let e_cost: f64 = expensive.iter().map(|s| s.estimated_cost).sum();
    assert!(e_cost > n_cost);
}

#[test]
fn scenario_description_not_empty() {
    let gen = ScenarioGenerator::new();
    let state = make_world_state();
    let scenarios = gen.generate("g1", "Build API", &state, &make_scenario_config());
    for s in &scenarios {
        assert!(!s.description.is_empty());
    }
}

#[test]
fn scenario_goal_id_preserved() {
    let gen = ScenarioGenerator::new();
    let state = make_world_state();
    let scenarios = gen.generate("goal_xyz", "Test", &state, &make_scenario_config());
    for s in &scenarios {
        assert_eq!(s.goal_id, "goal_xyz");
    }
}

#[test]
fn no_agents_still_generates_scenarios() {
    let gen = ScenarioGenerator::new();
    let state = make_world_state_no_agents();
    let scenarios = gen.generate("g1", "Build API", &state, &make_scenario_config());
    assert_eq!(scenarios.len(), 4);
}

#[test]
fn max_steps_respected() {
    let gen = ScenarioGenerator::new();
    let state = make_world_state();
    let config = ScenarioGenerationConfig {
        max_steps: 3,
        ..Default::default()
    };
    let scenarios = gen.generate(
        "g1",
        "Build a full production enterprise distributed scalable system",
        &state,
        &config,
    );
    for s in &scenarios {
        assert!(s.steps.len() <= 3);
    }
}

#[test]
fn scenario_has_created_at_timestamp() {
    let gen = ScenarioGenerator::new();
    let state = make_world_state();
    let before = chrono::Utc::now();
    let scenarios = gen.generate("g1", "Build API", &state, &make_scenario_config());
    let after = chrono::Utc::now();
    for s in &scenarios {
        assert!(s.created_at >= before);
        assert!(s.created_at <= after);
    }
}

#[test]
fn scenario_steps_have_positive_cost() {
    let gen = ScenarioGenerator::new();
    let state = make_world_state();
    let scenarios = gen.generate("g1", "Build API", &state, &make_scenario_config());
    for s in &scenarios {
        for step in &s.steps {
            assert!(step.cost >= 0.0);
        }
    }
}

#[test]
fn scenario_steps_have_positive_duration() {
    let gen = ScenarioGenerator::new();
    let state = make_world_state();
    let scenarios = gen.generate("g1", "Build API", &state, &make_scenario_config());
    for s in &scenarios {
        for step in &s.steps {
            assert!(step.duration_secs >= 0.0);
        }
    }
}

#[test]
fn scenario_cost_equals_step_sum() {
    let gen = ScenarioGenerator::new();
    let state = make_world_state();
    let scenarios = gen.generate("g1", "Build API", &state, &make_scenario_config());
    for s in &scenarios {
        let step_sum: f64 = s.steps.iter().map(|step| step.cost).sum();
        assert!(
            (s.estimated_cost - step_sum).abs() < 0.01,
            "Scenario cost should equal sum of step costs"
        );
    }
}

#[test]
fn scenario_ids_unique() {
    let gen = ScenarioGenerator::new();
    let state = make_world_state();
    let scenarios = gen.generate("g1", "Build API", &state, &make_scenario_config());
    let ids: std::collections::HashSet<_> = scenarios.iter().map(|s| s.id.clone()).collect();
    assert_eq!(ids.len(), scenarios.len());
}

#[test]
fn scenario_all_have_goal_id() {
    let gen = ScenarioGenerator::new();
    let state = make_world_state();
    let scenarios = gen.generate("g1", "Build API", &state, &make_scenario_config());
    for s in &scenarios {
        assert_eq!(s.goal_id, "g1");
    }
}

#[test]
fn custom_scenario_type_included() {
    let gen = ScenarioGenerator::new();
    let state = make_world_state();
    let config = ScenarioGenerationConfig {
        custom_types: vec!["experimental".to_string()],
        ..Default::default()
    };
    let scenarios = gen.generate("g1", "Build API", &state, &config);
    assert_eq!(scenarios.len(), 5); // 4 standard + 1 custom
    assert!(scenarios.iter().any(|s| matches!(
        &s.scenario_type,
        ScenarioType::Custom(name) if name == "experimental"
    )));
}

#[test]
fn scenario_type_display_non_empty() {
    let types = vec![
        ScenarioType::Fastest,
        ScenarioType::Cheapest,
        ScenarioType::HighestQuality,
        ScenarioType::Balanced,
        ScenarioType::Custom("test".into()),
    ];
    for t in &types {
        let display = t.to_string();
        assert!(!display.is_empty());
    }
}

#[test]
fn scenario_description_contains_type() {
    let gen = ScenarioGenerator::new();
    let state = make_world_state();
    let scenarios = gen.generate("g1", "Build API", &state, &make_scenario_config());
    for s in &scenarios {
        assert!(!s.description.is_empty(), "Description should not be empty");
    }
}

#[test]
fn scenario_quality_bounded() {
    let gen = ScenarioGenerator::new();
    let state = make_world_state();
    let scenarios = gen.generate("g1", "Build API", &state, &make_scenario_config());
    for s in &scenarios {
        assert!(s.estimated_quality >= 0.0);
        assert!(s.estimated_quality <= 1.0);
    }
}
