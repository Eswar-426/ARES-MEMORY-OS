use crate::forecast::models::HistoricalMission;
use crate::forecast::similarity::SimilarityEngine;

fn make_history() -> Vec<HistoricalMission> {
    vec![
        HistoricalMission {
            id: "m1".into(),
            title: "Build React Dashboard".into(),
            keywords: vec!["build".into(), "react".into(), "dashboard".into()],
            cost: 45.0,
            duration_secs: 3600.0,
            success: true,
            agent_count: 2,
            step_count: 5,
            completed_at: 1000,
        },
        HistoricalMission {
            id: "m2".into(),
            title: "Build NextJS Dashboard".into(),
            keywords: vec!["build".into(), "nextjs".into(), "dashboard".into()],
            cost: 55.0,
            duration_secs: 4200.0,
            success: true,
            agent_count: 3,
            step_count: 7,
            completed_at: 2000,
        },
        HistoricalMission {
            id: "m3".into(),
            title: "Build Internal Tool".into(),
            keywords: vec!["build".into(), "internal".into(), "tool".into()],
            cost: 30.0,
            duration_secs: 2400.0,
            success: false,
            agent_count: 1,
            step_count: 4,
            completed_at: 500,
        },
        HistoricalMission {
            id: "m4".into(),
            title: "Fix CSS Bug".into(),
            keywords: vec!["fix".into(), "css".into(), "bug".into()],
            cost: 5.0,
            duration_secs: 600.0,
            success: true,
            agent_count: 1,
            step_count: 2,
            completed_at: 3000,
        },
        HistoricalMission {
            id: "m5".into(),
            title: "Deploy Production Server".into(),
            keywords: vec!["deploy".into(), "production".into(), "server".into()],
            cost: 20.0,
            duration_secs: 1800.0,
            success: true,
            agent_count: 2,
            step_count: 3,
            completed_at: 4000,
        },
    ]
}

#[test]
fn find_similar_returns_matches() {
    let engine = SimilarityEngine::new();
    let history = make_history();
    let kw = vec!["build".into(), "dashboard".into()];
    let matches = engine.find_similar("Build React Dashboard", &kw, &history, 10);
    assert!(!matches.is_empty());
}

#[test]
fn find_similar_scores_highest_for_exact_match() {
    let engine = SimilarityEngine::new();
    let history = make_history();
    let kw = vec!["build".into(), "react".into(), "dashboard".into()];
    let matches = engine.find_similar("Build React Dashboard", &kw, &history, 10);
    assert_eq!(matches[0].mission.id, "m1");
}

#[test]
fn find_similar_respects_max_results() {
    let engine = SimilarityEngine::new();
    let history = make_history();
    let kw = vec!["build".into()];
    let matches = engine.find_similar("Build something", &kw, &history, 2);
    assert!(matches.len() <= 2);
}

#[test]
fn find_similar_sorted_by_score() {
    let engine = SimilarityEngine::new();
    let history = make_history();
    let kw = vec!["build".into(), "dashboard".into()];
    let matches = engine.find_similar("Build React Dashboard", &kw, &history, 10);
    for w in matches.windows(2) {
        assert!(w[0].similarity_score >= w[1].similarity_score);
    }
}

#[test]
fn find_similar_filters_low_scores() {
    let engine = SimilarityEngine::new();
    let history = make_history();
    let kw = vec!["build".into(), "dashboard".into()];
    let matches = engine.find_similar("Build React Dashboard", &kw, &history, 10);
    for m in &matches {
        assert!(m.similarity_score > 0.05);
    }
}

#[test]
fn matching_keywords_correct() {
    let engine = SimilarityEngine::new();
    let history = make_history();
    let kw = vec!["build".into(), "dashboard".into()];
    let matches = engine.find_similar("Build Dashboard", &kw, &history, 10);
    let react_match = matches.iter().find(|m| m.mission.id == "m1").unwrap();
    assert!(react_match.matching_keywords.contains(&"build".to_string()));
    assert!(react_match
        .matching_keywords
        .contains(&"dashboard".to_string()));
}

#[test]
fn empty_history_returns_empty() {
    let engine = SimilarityEngine::new();
    let matches = engine.find_similar("Build API", &["build".into()], &[], 10);
    assert!(matches.is_empty());
}

#[test]
fn unrelated_goal_low_similarity() {
    let engine = SimilarityEngine::new();
    let history = make_history();
    let kw = vec!["quantum".into(), "physics".into()];
    let matches = engine.find_similar("Quantum Physics Simulation", &kw, &history, 10);
    if !matches.is_empty() {
        assert!(matches[0].similarity_score < 0.5);
    }
}

#[test]
fn extract_keywords_removes_stopwords() {
    let engine = SimilarityEngine::new();
    let kw = engine.extract_keywords("Build a REST API for the dashboard");
    assert!(!kw.contains(&"a".to_string()));
    assert!(!kw.contains(&"for".to_string()));
    assert!(!kw.contains(&"the".to_string()));
    assert!(kw.contains(&"build".to_string()));
    assert!(kw.contains(&"rest".to_string()));
}

#[test]
fn extract_keywords_lowercased() {
    let engine = SimilarityEngine::new();
    let kw = engine.extract_keywords("Build REST API");
    for k in &kw {
        assert_eq!(*k, k.to_lowercase());
    }
}

#[test]
fn aggregate_stats_empty() {
    let engine = SimilarityEngine::new();
    let stats = engine.aggregate_stats(&[]);
    assert_eq!(stats.match_count, 0);
}

#[test]
fn aggregate_stats_computed() {
    let engine = SimilarityEngine::new();
    let history = make_history();
    let kw = vec!["build".into(), "dashboard".into()];
    let matches = engine.find_similar("Build Dashboard", &kw, &history, 10);
    let stats = engine.aggregate_stats(&matches);
    assert!(stats.match_count > 0);
    assert!(stats.success_rate >= 0.0 && stats.success_rate <= 1.0);
    assert!(stats.avg_cost > 0.0);
    assert!(stats.avg_duration_secs > 0.0);
}

#[test]
fn aggregate_stats_success_rate_correct() {
    let engine = SimilarityEngine::new();
    let history = make_history();
    let kw = vec!["build".into()];
    let matches = engine.find_similar("Build", &kw, &history, 10);
    let stats = engine.aggregate_stats(&matches);
    // History has 4/5 successes among "build" keyword matches
    assert!(stats.success_rate > 0.0);
}

#[test]
fn similarity_stats_serialization() {
    let stats = crate::forecast::similarity::SimilarityStats {
        match_count: 5,
        success_rate: 0.8,
        avg_cost: 40.0,
        avg_duration_secs: 3000.0,
        avg_similarity: 0.6,
    };
    let json = serde_json::to_string(&stats).unwrap();
    let back: crate::forecast::similarity::SimilarityStats = serde_json::from_str(&json).unwrap();
    assert_eq!(back.match_count, 5);
}

#[test]
fn similarity_match_serialization() {
    let m = crate::forecast::models::SimilarityMatch {
        mission: HistoricalMission {
            id: "m1".into(),
            title: "Test".into(),
            keywords: vec!["test".into()],
            cost: 10.0,
            duration_secs: 100.0,
            success: true,
            agent_count: 1,
            step_count: 2,
            completed_at: 1000,
        },
        similarity_score: 0.9,
        matching_keywords: vec!["test".into()],
    };
    let json = serde_json::to_string(&m).unwrap();
    let back: crate::forecast::models::SimilarityMatch = serde_json::from_str(&json).unwrap();
    assert!((back.similarity_score - 0.9).abs() < f64::EPSILON);
}
