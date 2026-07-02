use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Intent {
    ExplainEntity,
    AnalyzeImpact,
    FindPath,
    Dashboard,
    Traceability,
    GeneralQuestion,
    Unknown,
}

pub struct IntentRouter;

impl IntentRouter {
    #[tracing::instrument(name = "IntentRouter::parse")]
    pub fn parse(query: &str) -> Intent {
        let start = std::time::Instant::now();
        let q = query.to_lowercase();

        let intent =
            if q.starts_with("intent:why") || q.contains("why does") || q.contains("why exists") {
                Intent::ExplainEntity
            } else if q.starts_with("intent:impact")
                || q.contains("impact")
                || q.contains("blast radius")
                || q.contains("what breaks")
            {
                Intent::AnalyzeImpact
            } else if q.starts_with("intent:path") || q.contains("path") || q.contains("shortest") {
                Intent::FindPath
            } else if q.starts_with("intent:dashboard")
                || q == "dashboard"
                || q.contains("overview")
                || q.contains("analyze this")
            {
                Intent::Dashboard
            } else if q.starts_with("intent:trace")
                || q.contains("traceability")
                || q.contains("knowledge debt")
            {
                Intent::Traceability
            } else {
                Intent::GeneralQuestion
            };

        tracing::debug!(
            duration_ms = start.elapsed().as_millis(),
            ?intent,
            "Parsed intent"
        );
        intent
    }
}
