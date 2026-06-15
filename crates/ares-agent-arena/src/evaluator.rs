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

        // Context Efficiency: Useful Nodes / Retrieved Nodes
        // Here nodes = files + components combined
        let efficiency = precision; // Efficiency is exactly Precision in this mock, as "Useful" means "Correct"

        let confidence = (precision + recall + efficiency) / 3.0;

        result.precision_score = precision;
        result.recall_score = recall;
        result.confidence_score = confidence;

        result
    }
}
