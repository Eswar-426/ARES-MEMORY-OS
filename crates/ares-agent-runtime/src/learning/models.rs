use crate::models::MissionId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A recorded mission outcome for learning purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionOutcome {
    pub mission_id: MissionId,
    pub strategy_used: String,
    pub success: bool,
    pub score: f64,
    pub cost: f64,
    pub duration_secs: f64,
    pub completed_at: DateTime<Utc>,
}

/// EMA-tracked performance record for a strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyPerformanceRecord {
    pub strategy: String,
    pub ema_success_rate: f64,
    pub ema_cost: f64,
    pub ema_duration: f64,
    pub sample_count: u32,
    pub last_updated: DateTime<Utc>,
}

impl StrategyPerformanceRecord {
    pub fn new(strategy: String) -> Self {
        Self {
            strategy,
            ema_success_rate: 0.0,
            ema_cost: 0.0,
            ema_duration: 0.0,
            sample_count: 0,
            last_updated: Utc::now(),
        }
    }
}

/// EMA-tracked effectiveness record for an agent role.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentEffectivenessRecord {
    pub agent_role: String,
    pub ema_quality: f64,
    pub ema_latency: f64,
    pub task_count: u32,
    pub last_updated: DateTime<Utc>,
}

impl AgentEffectivenessRecord {
    pub fn new(agent_role: String) -> Self {
        Self {
            agent_role,
            ema_quality: 0.0,
            ema_latency: 0.0,
            task_count: 0,
            last_updated: Utc::now(),
        }
    }
}

/// Aggregated learning profile spanning all missions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningProfile {
    pub strategy_records: HashMap<String, StrategyPerformanceRecord>,
    pub agent_records: HashMap<String, AgentEffectivenessRecord>,
    pub total_missions: u32,
    pub overall_ema_score: f64,
}

impl Default for LearningProfile {
    fn default() -> Self {
        Self {
            strategy_records: HashMap::new(),
            agent_records: HashMap::new(),
            total_missions: 0,
            overall_ema_score: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strategy_record_new() {
        let rec = StrategyPerformanceRecord::new("fastest".to_string());
        assert_eq!(rec.strategy, "fastest");
        assert_eq!(rec.sample_count, 0);
        assert!((rec.ema_success_rate - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn agent_record_new() {
        let rec = AgentEffectivenessRecord::new("Coder".to_string());
        assert_eq!(rec.agent_role, "Coder");
        assert_eq!(rec.task_count, 0);
    }

    #[test]
    fn learning_profile_default() {
        let profile = LearningProfile::default();
        assert_eq!(profile.total_missions, 0);
        assert!(profile.strategy_records.is_empty());
        assert!(profile.agent_records.is_empty());
    }

    #[test]
    fn outcome_serialization() {
        let o = MissionOutcome {
            mission_id: MissionId::new(),
            strategy_used: "balanced".to_string(),
            success: true,
            score: 0.85,
            cost: 15.0,
            duration_secs: 120.0,
            completed_at: Utc::now(),
        };
        let json = serde_json::to_string(&o).unwrap();
        let back: MissionOutcome = serde_json::from_str(&json).unwrap();
        assert_eq!(back.strategy_used, "balanced");
        assert!(back.success);
    }

    #[test]
    fn profile_serialization() {
        let mut profile = LearningProfile {
            total_missions: 5,
            overall_ema_score: 0.75,
            ..Default::default()
        };
        profile.strategy_records.insert(
            "fastest".to_string(),
            StrategyPerformanceRecord::new("fastest".to_string()),
        );

        let json = serde_json::to_string(&profile).unwrap();
        let back: LearningProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(back.total_missions, 5);
        assert!(back.strategy_records.contains_key("fastest"));
    }
}
