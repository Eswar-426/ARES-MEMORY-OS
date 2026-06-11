use crate::evaluation::models::MissionScore;
use crate::models::MissionId;
use crate::reflection::mission_reflection::MissionReflection;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Phase of the self-improvement cycle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImprovementPhase {
    Execute,
    Reflect,
    Evaluate,
    Learn,
    Improve,
    Replan,
}

/// Actions taken during improvement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImprovementAction {
    UpdatedStrategy(String),
    AdjustedParameters(String),
    RebuiltDag,
    SkippedImprovement(String),
    RecordedLearning,
}

/// A single improvement cycle record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementCycle {
    pub cycle_id: String,
    pub mission_id: MissionId,
    pub phase: ImprovementPhase,
    pub reflection: Option<MissionReflection>,
    pub score: Option<MissionScore>,
    pub actions_taken: Vec<ImprovementAction>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// The result of running a complete improvement cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementOutcome {
    pub cycle_id: String,
    pub improved: bool,
    pub score_delta: f64,
    pub actions: Vec<ImprovementAction>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_ordering() {
        // Verify all phases can be compared
        assert_eq!(ImprovementPhase::Execute, ImprovementPhase::Execute);
        assert_ne!(ImprovementPhase::Execute, ImprovementPhase::Reflect);
    }

    #[test]
    fn action_serialization() {
        let action = ImprovementAction::UpdatedStrategy("fastest".to_string());
        let json = serde_json::to_string(&action).unwrap();
        let back: ImprovementAction = serde_json::from_str(&json).unwrap();
        if let ImprovementAction::UpdatedStrategy(s) = back {
            assert_eq!(s, "fastest");
        } else {
            panic!("Deserialized wrong variant");
        }
    }

    #[test]
    fn outcome_serialization() {
        let outcome = ImprovementOutcome {
            cycle_id: "cycle_1".to_string(),
            improved: true,
            score_delta: 0.15,
            actions: vec![ImprovementAction::RecordedLearning],
        };
        let json = serde_json::to_string(&outcome).unwrap();
        let back: ImprovementOutcome = serde_json::from_str(&json).unwrap();
        assert!(back.improved);
        assert!((back.score_delta - 0.15).abs() < f64::EPSILON);
    }

    #[test]
    fn cycle_serialization() {
        let cycle = ImprovementCycle {
            cycle_id: "c1".to_string(),
            mission_id: MissionId::new(),
            phase: ImprovementPhase::Learn,
            reflection: None,
            score: None,
            actions_taken: vec![],
            started_at: Utc::now(),
            completed_at: None,
        };
        let json = serde_json::to_string(&cycle).unwrap();
        let back: ImprovementCycle = serde_json::from_str(&json).unwrap();
        assert_eq!(back.phase, ImprovementPhase::Learn);
    }
}
