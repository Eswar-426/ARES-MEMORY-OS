use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum QueryIntent {
    Search,
    Explain,
    Why,
    History,
    Contradiction,
    Dependency,
    Impact,
    Evolution,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentAnalysisResult {
    pub intent: QueryIntent,
    pub confidence: f32,
}

pub struct IntentAnalyzer;

impl IntentAnalyzer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for IntentAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl IntentAnalyzer {
    /// Classifies query into one of the known intents. Must run in <10ms.
    pub fn analyze(&self, query: &str) -> IntentAnalysisResult {
        let q = query.to_lowercase();

        let (intent, confidence) = if q.contains("why") {
            (QueryIntent::Why, 0.9)
        } else if q.contains("explain") || q.contains("how") {
            (QueryIntent::Explain, 0.8)
        } else if q.contains("history") || q.contains("past") || q.contains("before") {
            (QueryIntent::History, 0.8)
        } else if q.contains("conflict")
            || q.contains("contradiction")
            || q.contains("versus")
            || q.contains("vs")
        {
            (QueryIntent::Contradiction, 0.85)
        } else if q.contains("depend") || q.contains("relies on") || q.contains("uses") {
            (QueryIntent::Dependency, 0.8)
        } else if q.contains("affect")
            || q.contains("impact")
            || q.contains("if i change")
            || q.contains("breaks")
        {
            (QueryIntent::Impact, 0.85)
        } else if q.contains("evolve")
            || q.contains("change")
            || q.contains("timeline")
            || q.contains("version")
        {
            (QueryIntent::Evolution, 0.8)
        } else if q.contains("find")
            || q.contains("search")
            || q.contains("show me")
            || q.contains("where")
            || q.contains("what is")
        {
            (QueryIntent::Search, 0.7)
        } else {
            // Default to Search with lower confidence if no strong keywords
            (QueryIntent::Search, 0.5)
        };

        IntentAnalysisResult { intent, confidence }
    }
}
