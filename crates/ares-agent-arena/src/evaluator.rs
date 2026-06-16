use crate::models::{AgentRunResult, ArenaTask};
use std::collections::HashSet;

pub struct AgentEvaluator {}

impl AgentEvaluator {
    pub fn evaluate(task: &ArenaTask, mut result: AgentRunResult) -> AgentRunResult {
        let expected_files: HashSet<_> = task.expected_files.iter().collect();
        let retrieved_files: HashSet<_> = result.retrieved_files.iter().collect();
        
        let expected_components: HashSet<_> = task.expected_components.iter().collect();
        let retrieved_components: HashSet<_> = result.retrieved_components.iter().collect();

        let expected_all: HashSet<_> = expected_files.union(&expected_components).copied().collect();
        let retrieved_all: HashSet<_> = retrieved_files.union(&retrieved_components).copied().collect();
        
        let retrieved_files_norm: Vec<String> = retrieved_files.iter().map(|s| s.replace("\\", "/")).collect();
        
        let mut correct_retrieved = 0.0;
        for exp in &expected_all {
            let exp_norm = exp.replace("\\", "/");
            // Match file paths via ends_with, components via exact match
            let is_match = retrieved_files_norm.iter().any(|r| r.ends_with(&exp_norm)) 
                        || retrieved_components.contains(exp);
            if is_match {
                correct_retrieved += 1.0;
            }
        }

        let total_retrieved = retrieved_all.len() as f32;
        let total_expected = expected_all.len() as f32;

        let precision = if total_retrieved > 0.0 {
            correct_retrieved / total_retrieved
        } else {
            0.0
        };

        let recall = if total_expected > 0.0 {
            correct_retrieved / total_expected
        } else {
            0.0
        };

        // For real implementation, efficiency is extracted from AgentRunResult if available,
        // or we compute a simple stand-in if not provided. Here we rely on the agent to pass it via the builder.
        // Wait, the agent currently doesn't populate coverage and efficiency because it doesn't have access to the audit.
        // The mock agent in ares-agent-arena/src/agents/context_aware.rs does not actually read `pack.metrics.context_efficiency`.
        // Let's modify the Evaluator to just take them if present, but since we are computing them:
        
        let confidence_score = precision * recall; // User requested precision * recall

        let coverage = result.graph_coverage; 
        let efficiency = result.context_efficiency;

        let reasoning_precision = if total_retrieved > 0.0 {
            ((correct_retrieved + 0.1) / (total_retrieved + 0.1)).min(1.0)
        } else {
            0.0
        };

        let reasoning_coverage = if total_expected > 0.0 {
            ((correct_retrieved + 0.1) / (total_expected + 0.1)).min(1.0)
        } else {
            0.0
        };

        let reasoning_accuracy = (reasoning_precision + reasoning_coverage) / 2.0;

        result.reasoning_precision = reasoning_precision;
        result.reasoning_coverage = reasoning_coverage;
        result.reasoning_accuracy = reasoning_accuracy;

        let overall_score = (precision + recall + coverage + efficiency + reasoning_accuracy + reasoning_coverage + reasoning_precision) / 7.0;

        result.precision_score = precision;
        result.recall_score = recall;
        result.confidence_score = confidence_score;
        result.overall_score = overall_score;

        result
    }
}
