use serde::{Deserialize, Serialize};

/// Type of knowledge evolution event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvolutionEventType {
    ConfidenceIncrease,
    ConfidenceDecrease,
    ContradictionDetected,
    DecayApplied,
    Reinforcement,
    EntityMerged,
    Deprecated,
}

impl EvolutionEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ConfidenceIncrease => "confidence_increase",
            Self::ConfidenceDecrease => "confidence_decrease",
            Self::ContradictionDetected => "contradiction_detected",
            Self::DecayApplied => "decay_applied",
            Self::Reinforcement => "reinforcement",
            Self::EntityMerged => "entity_merged",
            Self::Deprecated => "deprecated",
        }
    }

    pub fn from_str_val(s: &str) -> Self {
        match s {
            "confidence_increase" => Self::ConfidenceIncrease,
            "confidence_decrease" => Self::ConfidenceDecrease,
            "contradiction_detected" => Self::ContradictionDetected,
            "decay_applied" => Self::DecayApplied,
            "reinforcement" => Self::Reinforcement,
            "entity_merged" => Self::EntityMerged,
            "deprecated" => Self::Deprecated,
            _ => Self::ConfidenceIncrease,
        }
    }
}

/// A knowledge evolution entry — tracks changes to semantic memories.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEvolutionEntry {
    pub id: String,
    pub semantic_memory_id: String,
    pub event_type: EvolutionEventType,
    pub old_confidence: f64,
    pub new_confidence: f64,
    pub reason: String,
    pub source_episode_id: Option<String>,
    pub created_at: i64,
}

/// Configuration for knowledge decay.
#[derive(Debug, Clone)]
pub struct DecayConfig {
    /// Daily decay rate (multiplier). E.g., 0.999 means 0.1% decay per day.
    pub daily_rate: f64,
    /// Minimum confidence before an item is deprecated.
    pub min_confidence: f64,
    /// Maximum age in days before forcing decay.
    pub max_age_days: u32,
}

impl Default for DecayConfig {
    fn default() -> Self {
        Self {
            daily_rate: 0.999,
            min_confidence: 0.1,
            max_age_days: 365,
        }
    }
}

/// A detected contradiction between semantic memories.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContradictionDetection {
    pub memory_id_a: String,
    pub memory_id_b: String,
    pub subject: String,
    pub conflict_description: String,
    pub confidence: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evolution_event_type_roundtrip() {
        for t in &[
            EvolutionEventType::ConfidenceIncrease,
            EvolutionEventType::ConfidenceDecrease,
            EvolutionEventType::ContradictionDetected,
            EvolutionEventType::DecayApplied,
            EvolutionEventType::Reinforcement,
            EvolutionEventType::EntityMerged,
            EvolutionEventType::Deprecated,
        ] {
            assert_eq!(&EvolutionEventType::from_str_val(t.as_str()), t);
        }
    }

    #[test]
    fn evolution_entry_serialization() {
        let entry = KnowledgeEvolutionEntry {
            id: "ke_1".into(),
            semantic_memory_id: "sm_1".into(),
            event_type: EvolutionEventType::Reinforcement,
            old_confidence: 0.7,
            new_confidence: 0.85,
            reason: "Confirmed by episode ep_5".into(),
            source_episode_id: Some("ep_5".into()),
            created_at: 1000,
        };
        let json = serde_json::to_string(&entry).unwrap();
        let back: KnowledgeEvolutionEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(back.event_type, EvolutionEventType::Reinforcement);
        assert!((back.new_confidence - 0.85).abs() < f64::EPSILON);
    }

    #[test]
    fn decay_config_default() {
        let cfg = DecayConfig::default();
        assert!((cfg.daily_rate - 0.999).abs() < f64::EPSILON);
        assert!((cfg.min_confidence - 0.1).abs() < f64::EPSILON);
    }

    #[test]
    fn contradiction_detection_serialization() {
        let cd = ContradictionDetection {
            memory_id_a: "sm_1".into(),
            memory_id_b: "sm_2".into(),
            subject: "Authentication".into(),
            conflict_description: "sm_1 says JWT, sm_2 says session".into(),
            confidence: 0.9,
        };
        let json = serde_json::to_string(&cd).unwrap();
        let back: ContradictionDetection = serde_json::from_str(&json).unwrap();
        assert_eq!(back.subject, "Authentication");
    }
}
