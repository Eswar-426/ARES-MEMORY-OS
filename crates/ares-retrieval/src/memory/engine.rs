use crate::memory::models::{
    ContextQuality, MemoryExplanation, MemoryResult, RetrievalCoverage, RetrievalExplanationNode,
};
use crate::memory::planner::MemoryQueryPlanner;
use crate::memory::source::MemorySourceRegistry;

pub struct RetrievalEngine {
    planner: MemoryQueryPlanner,
    registry: MemorySourceRegistry,
}

impl RetrievalEngine {
    pub fn new(planner: MemoryQueryPlanner, registry: MemorySourceRegistry) -> Self {
        Self { planner, registry }
    }

    pub fn retrieve(&self, query_text: &str) -> MemoryResult {
        // 1. Plan the query
        let query = self.planner.plan(query_text);
        let strategy = self.planner.determine_strategy(&query);

        // 2. Execute via Source Registry
        let mut context = self.registry.execute(&strategy, &query);

        // 3. Compute Retrieval Coverage
        let coverage = RetrievalCoverage {
            requirements_found: context.requirements.len(),
            decisions_found: context.decisions.len(),
            evidence_found: context.evidence.len(),
            gaps_found: context.gaps.len(),
            resolutions_found: context.resolution_plans.len(),
        };

        // 4. Compute Context Quality
        // Here we'd do a more involved calculation, but for the MVP:
        let quality = ContextQuality {
            traceability_score: if coverage.requirements_found > 0 && coverage.decisions_found > 0 {
                95.0
            } else {
                50.0
            },
            health_score: context
                .health_report
                .as_ref()
                .map(|r| r.health.overall_score)
                .unwrap_or(100.0),
            debt_score: context
                .knowledge_debt
                .as_ref()
                .map(|d| d.debt_score)
                .unwrap_or(0.0),
            completeness_score: 85.0, // Arbitrary for now
        };
        context.context_quality = Some(quality);

        // 5. Generate Explanation Tree
        let mut tree = Vec::new();
        tree.push(RetrievalExplanationNode {
            source: "Query Planner".to_string(),
            reason: format!(
                "Identified intent '{:?}' and strategy '{:?}' based on keywords.",
                query.intent, strategy
            ),
        });

        for req in &context.requirements {
            tree.push(RetrievalExplanationNode {
                source: format!("Requirement: {}", req.id),
                reason: "Matched context scope for strategy.".to_string(),
            });
        }
        for dec in &context.decisions {
            tree.push(RetrievalExplanationNode {
                source: format!("Decision: {}", dec.id),
                reason: "Provides architectural context for the requirement.".to_string(),
            });
        }

        let explanation = MemoryExplanation {
            why_this_was_returned: format!("Retrieved memory based on strategy: {:?}", strategy),
            confidence: 0.9, // Deterministic confidence, no AI
            retrieval_strategy: strategy,
            tree,
            coverage,
        };

        MemoryResult {
            query,
            context,
            explanation,
        }
    }
}
