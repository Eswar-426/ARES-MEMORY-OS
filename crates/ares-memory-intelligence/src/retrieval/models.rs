use serde::{Deserialize, Serialize};

/// Type of retrieval query.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetrievalQueryType {
    SimilarMission,
    FailureSearch,
    SuccessSearch,
    LessonSearch,
    PrincipleSearch,
    General,
}

impl RetrievalQueryType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SimilarMission => "similar_mission",
            Self::FailureSearch => "failure_search",
            Self::SuccessSearch => "success_search",
            Self::LessonSearch => "lesson_search",
            Self::PrincipleSearch => "principle_search",
            Self::General => "general",
        }
    }

    pub fn from_str_val(s: &str) -> Self {
        match s {
            "similar_mission" => Self::SimilarMission,
            "failure_search" => Self::FailureSearch,
            "success_search" => Self::SuccessSearch,
            "lesson_search" => Self::LessonSearch,
            "principle_search" => Self::PrincipleSearch,
            _ => Self::General,
        }
    }
}

/// A retrieval request — what we want from memory.
#[derive(Debug, Clone)]
pub struct RetrievalRequest {
    pub query_text: String,
    pub query_type: RetrievalQueryType,
    pub tags: Vec<String>,
    pub max_results: u32,
    pub min_confidence: f64,
}

impl Default for RetrievalRequest {
    fn default() -> Self {
        Self {
            query_text: String::new(),
            query_type: RetrievalQueryType::General,
            tags: vec![],
            max_results: 10,
            min_confidence: 0.0,
        }
    }
}

/// A single result from a retrieval query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalResult {
    pub id: String,
    pub source_type: String, // "episode", "semantic_memory", "principle", "experience"
    pub title: String,
    pub content: String,
    pub relevance_score: f64,
    pub confidence: f64,
}

/// The combined response from a retrieval query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalResponse {
    pub results: Vec<RetrievalResult>,
    pub total_count: usize,
    pub query_type: String,
    pub retrieval_ms: u64,
}

/// A log entry for retrieval auditing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalLogEntry {
    pub id: String,
    pub query_text: String,
    pub query_type: String,
    pub results_count: u32,
    pub result_ids: Vec<String>,
    pub relevance_score: f64,
    pub retrieval_ms: u64,
    pub created_at: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retrieval_query_type_roundtrip() {
        for t in &[
            RetrievalQueryType::SimilarMission,
            RetrievalQueryType::FailureSearch,
            RetrievalQueryType::SuccessSearch,
            RetrievalQueryType::LessonSearch,
            RetrievalQueryType::PrincipleSearch,
            RetrievalQueryType::General,
        ] {
            assert_eq!(&RetrievalQueryType::from_str_val(t.as_str()), t);
        }
    }

    #[test]
    fn retrieval_request_default() {
        let req = RetrievalRequest::default();
        assert_eq!(req.query_type, RetrievalQueryType::General);
        assert_eq!(req.max_results, 10);
    }

    #[test]
    fn retrieval_result_serialization() {
        let result = RetrievalResult {
            id: "r_1".into(),
            source_type: "episode".into(),
            title: "Deploy service".into(),
            content: "Deployed to production".into(),
            relevance_score: 0.85,
            confidence: 0.9,
        };
        let json = serde_json::to_string(&result).unwrap();
        let back: RetrievalResult = serde_json::from_str(&json).unwrap();
        assert_eq!(back.source_type, "episode");
    }

    #[test]
    fn retrieval_response_serialization() {
        let resp = RetrievalResponse {
            results: vec![],
            total_count: 0,
            query_type: "general".into(),
            retrieval_ms: 5,
        };
        let json = serde_json::to_string(&resp).unwrap();
        let back: RetrievalResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(back.retrieval_ms, 5);
    }

    #[test]
    fn retrieval_log_entry_serialization() {
        let entry = RetrievalLogEntry {
            id: "rl_1".into(),
            query_text: "deploy failures".into(),
            query_type: "failure_search".into(),
            results_count: 3,
            result_ids: vec!["r_1".into(), "r_2".into(), "r_3".into()],
            relevance_score: 0.7,
            retrieval_ms: 12,
            created_at: 1000,
        };
        let json = serde_json::to_string(&entry).unwrap();
        let back: RetrievalLogEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(back.result_ids.len(), 3);
    }
}
