use crate::memory::models::{MemoryQuery, QueryIntent, QueryPattern, RetrievalStrategy};

pub struct MemoryQueryPlanner {
    patterns: Vec<QueryPattern>,
}

impl MemoryQueryPlanner {
    pub fn new(patterns: Vec<QueryPattern>) -> Self {
        Self { patterns }
    }

    pub fn default_patterns() -> Vec<QueryPattern> {
        vec![
            QueryPattern {
                intent: QueryIntent::Why,
                keywords: vec!["why".into(), "reason".into(), "purpose".into()],
            },
            QueryPattern {
                intent: QueryIntent::Who,
                keywords: vec!["who".into(), "owner".into(), "author".into()],
            },
            QueryPattern {
                intent: QueryIntent::Impact,
                keywords: vec!["impact".into(), "break".into(), "depend".into(), "change".into()],
            },
            QueryPattern {
                intent: QueryIntent::Debt,
                keywords: vec!["debt".into(), "gap".into(), "missing".into(), "resolution".into()],
            },
            QueryPattern {
                intent: QueryIntent::Traceability,
                keywords: vec!["trace".into(), "link".into(), "connect".into()],
            },
            QueryPattern {
                intent: QueryIntent::Health,
                keywords: vec!["health".into(), "score".into()],
            },
            QueryPattern {
                intent: QueryIntent::Governance,
                keywords: vec!["govern".into(), "approve".into(), "evidence".into()],
            },
        ]
    }

    pub fn plan(&self, query: &str) -> MemoryQuery {
        let q_lower = query.to_lowercase();
        let mut best_intent = QueryIntent::What; // Default fallback

        for pattern in &self.patterns {
            for kw in &pattern.keywords {
                if q_lower.contains(kw) {
                    best_intent = pattern.intent.clone();
                    break;
                }
            }
        }

        MemoryQuery {
            query: query.to_string(),
            intent: best_intent,
        }
    }

    pub fn determine_strategy(&self, query: &MemoryQuery) -> RetrievalStrategy {
        match query.intent {
            QueryIntent::Why | QueryIntent::What => RetrievalStrategy::RequirementFocused,
            QueryIntent::Who | QueryIntent::When | QueryIntent::Governance => {
                RetrievalStrategy::GovernanceFocused
            }
            QueryIntent::Impact | QueryIntent::Traceability => {
                RetrievalStrategy::TraceabilityFocused
            }
            QueryIntent::Debt | QueryIntent::Resolution => RetrievalStrategy::DebtFocused,
            QueryIntent::Health => RetrievalStrategy::HealthFocused,
        }
    }
}

impl Default for MemoryQueryPlanner {
    fn default() -> Self {
        Self::new(Self::default_patterns())
    }
}
