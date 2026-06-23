use super::dataset::BenchmarkQuery;
use ares_context::models::pack::ContextPack;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryEvaluation {
    pub query: String,
    pub expected_intent: String,
    pub actual_intent: String,
    pub recall: f32,
    pub precision: f32,
    pub latency_ms: u64,
    pub returned_nodes: usize,
    pub passed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextBenchmarkReport {
    pub total_queries: usize,
    pub avg_recall: f32,
    pub avg_precision: f32,
    pub avg_latency_ms: u64,
    pub avg_nodes_returned: f32,
    pub passing_queries: usize,
    pub evaluations: Vec<QueryEvaluation>,
}

pub struct ContextEvaluator;

impl ContextEvaluator {
    pub fn evaluate_pack(query: &BenchmarkQuery, pack: &ContextPack) -> QueryEvaluation {
        let expected_set: HashSet<_> = query.expected_files.iter().collect();
        let actual_set: HashSet<_> = pack.relevant_files.iter().collect();

        // Calculate Recall
        let recall = if expected_set.is_empty() {
            1.0
        } else {
            let intersection = expected_set.intersection(&actual_set).count();
            intersection as f32 / expected_set.len() as f32
        };

        // Calculate Precision
        let precision = if actual_set.is_empty() {
            if expected_set.is_empty() {
                1.0
            } else {
                0.0
            }
        } else {
            let intersection = expected_set.intersection(&actual_set).count();
            intersection as f32 / actual_set.len() as f32
        };

        // Check if passed minimum node expectation
        let passed_nodes = pack.relevant_nodes.len() >= query.expected_min_nodes;

        // Intent match
        let actual_intent_str = format!("{:?}", pack.intent);
        let passed_intent = actual_intent_str == query.intent;

        QueryEvaluation {
            query: query.query.clone(),
            expected_intent: query.intent.clone(),
            actual_intent: actual_intent_str,
            recall,
            precision,
            latency_ms: pack.retrieval_time_ms,
            returned_nodes: pack.relevant_nodes.len(),
            passed: recall >= 0.9 && passed_nodes && passed_intent, // Stricter requirement
        }
    }

    pub fn generate_report(evaluations: Vec<QueryEvaluation>) -> ContextBenchmarkReport {
        let total = evaluations.len();
        if total == 0 {
            return ContextBenchmarkReport {
                total_queries: 0,
                avg_recall: 0.0,
                avg_precision: 0.0,
                avg_latency_ms: 0,
                avg_nodes_returned: 0.0,
                passing_queries: 0,
                evaluations,
            };
        }

        let sum_recall: f32 = evaluations.iter().map(|e| e.recall).sum();
        let sum_precision: f32 = evaluations.iter().map(|e| e.precision).sum();
        let sum_latency: u64 = evaluations.iter().map(|e| e.latency_ms).sum();
        let sum_nodes: usize = evaluations.iter().map(|e| e.returned_nodes).sum();
        let passed = evaluations.iter().filter(|e| e.passed).count();

        ContextBenchmarkReport {
            total_queries: total,
            avg_recall: sum_recall / total as f32,
            avg_precision: sum_precision / total as f32,
            avg_latency_ms: sum_latency / total as u64,
            avg_nodes_returned: sum_nodes as f32 / total as f32,
            passing_queries: passed,
            evaluations,
        }
    }
}
