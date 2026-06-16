#[cfg(test)]
mod tests {
    use crate::memory::models::{MemoryQuery, QueryIntent, RetrievalStrategy};
    use crate::memory::planner::MemoryQueryPlanner;
    use crate::memory::source::{
        DecisionMemorySource, GapMemorySource, MemorySourceRegistry, RequirementMemorySource,
    };
    use std::sync::Arc;

    #[test]
    fn test_query_planner_intents() {
        let planner = MemoryQueryPlanner::default();

        assert_eq!(
            planner.plan("Why does authentication exist?").intent,
            QueryIntent::Why
        );
        assert_eq!(
            planner.plan("Who owns the payment service?").intent,
            QueryIntent::Who
        );
        assert_eq!(
            planner.plan("What breaks if I change this?").intent,
            QueryIntent::Impact
        );
        assert_eq!(
            planner.plan("What is the knowledge debt here?").intent,
            QueryIntent::Debt
        );
    }

    #[test]
    fn test_query_planner_strategies() {
        let planner = MemoryQueryPlanner::default();

        let q1 = MemoryQuery {
            query: "Why?".into(),
            intent: QueryIntent::Why,
        };
        assert_eq!(
            planner.determine_strategy(&q1),
            RetrievalStrategy::RequirementFocused
        );

        let q2 = MemoryQuery {
            query: "Who?".into(),
            intent: QueryIntent::Who,
        };
        assert_eq!(
            planner.determine_strategy(&q2),
            RetrievalStrategy::GovernanceFocused
        );
    }


}
