use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConfidenceResult<T> {
    pub data: T,
    pub confidence_score: f64,
    pub coverage_score: f64,
    pub health_score: f64,
    pub source_count: usize,
    pub evidence_count: usize,
    pub requirements_used: Vec<String>,
    pub decisions_used: Vec<String>,
    pub architecture_used: Vec<String>,
    pub evidence_used: Vec<String>,
    pub reasoning_path: Vec<String>,
}

pub struct ConfidenceEngine;

impl ConfidenceEngine {
    pub fn wrap<T>(
        data: T,
        coverage_score: f64,
        health_score: f64,
        requirements_used: Vec<String>,
        decisions_used: Vec<String>,
        architecture_used: Vec<String>,
        evidence_used: Vec<String>,
        reasoning_path: Vec<String>,
    ) -> ConfidenceResult<T> {
        let source_count = requirements_used.len() + decisions_used.len() + architecture_used.len();
        let evidence_count = evidence_used.len();

        // Base confidence is the health of the repository
        // But if the query pulls heavily from evidence and requirements, confidence increases.
        // If it relies on zero reasoning nodes, confidence drops drastically.

        let mut confidence_score = health_score;

        if source_count == 0 {
            confidence_score *= 0.5; // Halve the confidence if no sources are used
        } else {
            // Small bump for having evidence
            if evidence_count > 0 {
                confidence_score = (confidence_score + 5.0).min(100.0);
            }
        }

        ConfidenceResult {
            data,
            confidence_score,
            coverage_score,
            health_score,
            source_count,
            evidence_count,
            requirements_used,
            decisions_used,
            architecture_used,
            evidence_used,
            reasoning_path,
        }
    }
}
