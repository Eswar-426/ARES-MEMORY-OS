use serde::{Deserialize, Serialize};

/// Type of decision recorded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DecisionType {
    Strategic,
    Tactical,
    Technical,
    Resource,
    Retry,
}

impl DecisionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Strategic => "strategic",
            Self::Tactical => "tactical",
            Self::Technical => "technical",
            Self::Resource => "resource",
            Self::Retry => "retry",
        }
    }

    pub fn from_str_val(s: &str) -> Self {
        match s {
            "strategic" => Self::Strategic,
            "tactical" => Self::Tactical,
            "technical" => Self::Technical,
            "resource" => Self::Resource,
            "retry" => Self::Retry,
            _ => Self::Strategic,
        }
    }
}

/// Outcome of a past decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DecisionOutcomeType {
    Positive,
    Negative,
    Neutral,
    Unknown,
}

impl DecisionOutcomeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Positive => "positive",
            Self::Negative => "negative",
            Self::Neutral => "neutral",
            Self::Unknown => "unknown",
        }
    }

    pub fn from_str_val(s: &str) -> Self {
        match s {
            "positive" => Self::Positive,
            "negative" => Self::Negative,
            "neutral" => Self::Neutral,
            _ => Self::Unknown,
        }
    }
}

/// A rejected alternative with the reason it was rejected.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionAlternative {
    pub option: String,
    pub reason_rejected: String,
}

/// A complete decision record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRecord {
    pub id: String,
    pub episode_id: Option<String>,
    pub decision_type: DecisionType,
    pub question: String,
    pub chosen_option: String,
    pub alternatives: Vec<DecisionAlternative>,
    pub reasoning: String,
    pub confidence: f64,
    pub outcome: Option<DecisionOutcomeType>,
    pub context: serde_json::Value,
    pub created_at: i64,
    pub resolved_at: Option<i64>,
}

/// Query for searching decision history.
#[derive(Debug, Clone, Default)]
pub struct DecisionQuery {
    pub episode_id: Option<String>,
    pub decision_type: Option<DecisionType>,
    pub outcome: Option<DecisionOutcomeType>,
    pub search_text: Option<String>,
    pub limit: Option<u32>,
}

/// An explanation of why a decision was made.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionExplanation {
    pub decision: DecisionRecord,
    pub similar_past_decisions: Vec<DecisionRecord>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decision_type_roundtrip() {
        for t in &[
            DecisionType::Strategic,
            DecisionType::Tactical,
            DecisionType::Technical,
            DecisionType::Resource,
            DecisionType::Retry,
        ] {
            assert_eq!(&DecisionType::from_str_val(t.as_str()), t);
        }
    }

    #[test]
    fn decision_outcome_roundtrip() {
        for o in &[
            DecisionOutcomeType::Positive,
            DecisionOutcomeType::Negative,
            DecisionOutcomeType::Neutral,
            DecisionOutcomeType::Unknown,
        ] {
            assert_eq!(&DecisionOutcomeType::from_str_val(o.as_str()), o);
        }
    }

    #[test]
    fn decision_record_serialization() {
        let rec = DecisionRecord {
            id: "d_1".into(),
            episode_id: Some("ep_1".into()),
            decision_type: DecisionType::Technical,
            question: "Which database to use?".into(),
            chosen_option: "SQLite".into(),
            alternatives: vec![DecisionAlternative {
                option: "PostgreSQL".into(),
                reason_rejected: "Too complex for embedded use".into(),
            }],
            reasoning: "SQLite is embedded and zero-config".into(),
            confidence: 0.85,
            outcome: Some(DecisionOutcomeType::Positive),
            context: serde_json::json!({"project": "ares"}),
            created_at: 1000000,
            resolved_at: Some(2000000),
        };
        let json = serde_json::to_string(&rec).unwrap();
        let back: DecisionRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(back.chosen_option, "SQLite");
        assert_eq!(back.alternatives.len(), 1);
    }

    #[test]
    fn decision_alternative_serialization() {
        let alt = DecisionAlternative {
            option: "MongoDB".into(),
            reason_rejected: "Not suitable for embedded".into(),
        };
        let json = serde_json::to_string(&alt).unwrap();
        let back: DecisionAlternative = serde_json::from_str(&json).unwrap();
        assert_eq!(back.option, "MongoDB");
    }

    #[test]
    fn decision_query_default() {
        let q = DecisionQuery::default();
        assert!(q.episode_id.is_none());
        assert!(q.decision_type.is_none());
    }
}
