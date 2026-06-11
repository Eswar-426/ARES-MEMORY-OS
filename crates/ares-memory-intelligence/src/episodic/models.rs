use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Outcome of an episode / mission execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EpisodeOutcome {
    Success,
    PartialSuccess,
    Failure,
    Aborted,
    Unknown,
}

impl EpisodeOutcome {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::PartialSuccess => "partial_success",
            Self::Failure => "failure",
            Self::Aborted => "aborted",
            Self::Unknown => "unknown",
        }
    }

    pub fn from_str_val(s: &str) -> Self {
        match s {
            "success" => Self::Success,
            "partial_success" => Self::PartialSuccess,
            "failure" => Self::Failure,
            "aborted" => Self::Aborted,
            _ => Self::Unknown,
        }
    }

    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success | Self::PartialSuccess)
    }
}

/// The type of event within an episode timeline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EpisodeEventType {
    Action,
    Decision,
    Error,
    Milestone,
    Observation,
    Reflection,
}

impl EpisodeEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Action => "action",
            Self::Decision => "decision",
            Self::Error => "error",
            Self::Milestone => "milestone",
            Self::Observation => "observation",
            Self::Reflection => "reflection",
        }
    }

    pub fn from_str_val(s: &str) -> Self {
        match s {
            "action" => Self::Action,
            "decision" => Self::Decision,
            "error" => Self::Error,
            "milestone" => Self::Milestone,
            "observation" => Self::Observation,
            "reflection" => Self::Reflection,
            _ => Self::Observation,
        }
    }
}

/// A complete mission episode — what happened, who was involved, what was learned.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    pub id: String,
    pub mission_id: String,
    pub title: String,
    pub description: String,
    pub agents_involved: Vec<String>,
    pub decisions_made: Vec<String>,
    pub outcome: EpisodeOutcome,
    pub score: f64,
    pub cost: f64,
    pub duration_secs: f64,
    pub failures: Vec<String>,
    pub lessons_learned: Vec<String>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// An individual event within an episode timeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeEvent {
    pub id: String,
    pub episode_id: String,
    pub event_type: EpisodeEventType,
    pub description: String,
    pub agent_id: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// A compressed summary of an episode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeSummary {
    pub id: String,
    pub episode_id: String,
    pub summary_text: String,
    pub key_insights: Vec<String>,
    pub compression_ratio: f64,
    pub created_at: DateTime<Utc>,
}

/// Query parameters for searching episodes.
#[derive(Debug, Clone, Default)]
pub struct EpisodeQuery {
    pub outcome: Option<EpisodeOutcome>,
    pub mission_id: Option<String>,
    pub search_text: Option<String>,
    pub min_score: Option<f64>,
    pub tags: Vec<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

/// A ranked episode with its relevance score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankedEpisode {
    pub episode: Episode,
    pub relevance_score: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn episode_outcome_roundtrip() {
        for outcome in &[
            EpisodeOutcome::Success,
            EpisodeOutcome::PartialSuccess,
            EpisodeOutcome::Failure,
            EpisodeOutcome::Aborted,
            EpisodeOutcome::Unknown,
        ] {
            let s = outcome.as_str();
            let back = EpisodeOutcome::from_str_val(s);
            assert_eq!(&back, outcome);
        }
    }

    #[test]
    fn episode_outcome_is_success() {
        assert!(EpisodeOutcome::Success.is_success());
        assert!(EpisodeOutcome::PartialSuccess.is_success());
        assert!(!EpisodeOutcome::Failure.is_success());
        assert!(!EpisodeOutcome::Aborted.is_success());
        assert!(!EpisodeOutcome::Unknown.is_success());
    }

    #[test]
    fn episode_event_type_roundtrip() {
        for t in &[
            EpisodeEventType::Action,
            EpisodeEventType::Decision,
            EpisodeEventType::Error,
            EpisodeEventType::Milestone,
            EpisodeEventType::Observation,
            EpisodeEventType::Reflection,
        ] {
            let s = t.as_str();
            let back = EpisodeEventType::from_str_val(s);
            assert_eq!(&back, t);
        }
    }

    #[test]
    fn episode_serialization_roundtrip() {
        let ep = Episode {
            id: "ep_1".into(),
            mission_id: "m_1".into(),
            title: "Build auth system".into(),
            description: "Implemented JWT-based authentication".into(),
            agents_involved: vec!["agent_coder".into()],
            decisions_made: vec!["Use JWT over sessions".into()],
            outcome: EpisodeOutcome::Success,
            score: 0.95,
            cost: 12.5,
            duration_secs: 300.0,
            failures: vec![],
            lessons_learned: vec!["JWT requires token refresh logic".into()],
            tags: vec!["auth".into(), "jwt".into()],
            created_at: Utc::now(),
            completed_at: Some(Utc::now()),
        };
        let json = serde_json::to_string(&ep).unwrap();
        let back: Episode = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, "ep_1");
        assert_eq!(back.outcome, EpisodeOutcome::Success);
        assert_eq!(back.tags.len(), 2);
    }

    #[test]
    fn episode_event_serialization() {
        let ev = EpisodeEvent {
            id: "ev_1".into(),
            episode_id: "ep_1".into(),
            event_type: EpisodeEventType::Decision,
            description: "Chose JWT".into(),
            agent_id: Some("agent_1".into()),
            timestamp: Utc::now(),
            metadata: serde_json::json!({"confidence": 0.9}),
        };
        let json = serde_json::to_string(&ev).unwrap();
        let back: EpisodeEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(back.event_type, EpisodeEventType::Decision);
    }

    #[test]
    fn episode_summary_serialization() {
        let s = EpisodeSummary {
            id: "s_1".into(),
            episode_id: "ep_1".into(),
            summary_text: "Built auth with JWT".into(),
            key_insights: vec!["JWT needs refresh".into()],
            compression_ratio: 0.3,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: EpisodeSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(back.key_insights.len(), 1);
        assert!((back.compression_ratio - 0.3).abs() < f64::EPSILON);
    }

    #[test]
    fn episode_query_default() {
        let q = EpisodeQuery::default();
        assert!(q.outcome.is_none());
        assert!(q.mission_id.is_none());
        assert!(q.search_text.is_none());
        assert!(q.tags.is_empty());
    }

    #[test]
    fn ranked_episode_serialization() {
        let ep = Episode {
            id: "ep_2".into(),
            mission_id: "m_2".into(),
            title: "Deploy service".into(),
            description: "Deployed to production".into(),
            agents_involved: vec![],
            decisions_made: vec![],
            outcome: EpisodeOutcome::Failure,
            score: 0.2,
            cost: 50.0,
            duration_secs: 600.0,
            failures: vec!["Timeout on deploy".into()],
            lessons_learned: vec!["Check health endpoint first".into()],
            tags: vec!["deploy".into()],
            created_at: Utc::now(),
            completed_at: None,
        };
        let ranked = RankedEpisode {
            episode: ep,
            relevance_score: 0.87,
        };
        let json = serde_json::to_string(&ranked).unwrap();
        let back: RankedEpisode = serde_json::from_str(&json).unwrap();
        assert!((back.relevance_score - 0.87).abs() < f64::EPSILON);
        assert_eq!(back.episode.outcome, EpisodeOutcome::Failure);
    }
}
