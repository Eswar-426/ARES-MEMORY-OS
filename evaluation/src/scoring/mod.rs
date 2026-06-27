use crate::adapters::EvaluationEngineResult;
use crate::dataset::{EvaluationCase, FactImportance};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailureKind {
    Hallucination,
    MissingEvidence,
    TraversalMismatch,
    LowRecall,
    LowPrecision,
    ConfidenceMismatch,
    NonDeterminism,
}

impl std::fmt::Display for FailureKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            FailureKind::Hallucination => "Hallucinated claim/node",
            FailureKind::MissingEvidence => "Missing required evidence",
            FailureKind::TraversalMismatch => "Traversal mismatch",
            FailureKind::LowRecall => "Low recall (missing required facts)",
            FailureKind::LowPrecision => "Low precision",
            FailureKind::ConfidenceMismatch => "Confidence mismatch",
            FailureKind::NonDeterminism => "Non-deterministic result",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Failure {
    pub kind: FailureKind,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Score {
    pub recall: f64,
    pub precision: f64,
    pub evidence_coverage: f64,
    pub completeness: f64,
    pub confidence: f64,
    pub traversal_match: f64,
    pub graph_coverage: f64,
    pub hallucination_rate: f64,
    pub failures: Vec<Failure>,
    pub overall: f64,
}

fn get_weight(importance: &FactImportance) -> f64 {
    match importance {
        FactImportance::Required => 1.0,
        FactImportance::Major => 0.8,
        FactImportance::Minor => 0.3,
        FactImportance::Optional => 0.1,
    }
}

pub fn calculate_score(case: &EvaluationCase, result: &EvaluationEngineResult, graph_nodes: &[String]) -> Score {
    let mut failures = Vec::new();

    // Convert dataset facts to pseudo canonical strings for comparison
    let dataset_claims: Vec<String> = case.facts.iter().map(|f| format!("{}:{}", f.claim.kind, f.claim.id).to_lowercase()).collect();
    let result_claims: Vec<String> = result.claims.iter().map(|c| format!("{:?}:{}", c.kind, c.id).to_lowercase()).collect();

    // 1. Recall (% of Required/Major facts present)
    let required_major: Vec<_> = case.facts.iter().filter(|f| {
        matches!(f.importance, FactImportance::Required | FactImportance::Major)
    }).collect();
    
    let mut recall_matched = 0.0;
    for f in &required_major {
        let fact_str = format!("{}:{}", f.claim.kind, f.claim.id).to_lowercase();
        if result_claims.contains(&fact_str) {
            recall_matched += 1.0;
        } else {
            failures.push(Failure {
                kind: FailureKind::LowRecall,
                description: format!("Missing required fact: {}", fact_str),
            });
        }
    }
    let recall = if required_major.is_empty() { 1.0 } else { recall_matched / required_major.len() as f64 };

    // 2. Precision
    let mut precision_matched = 0.0;
    for claim_str in &result_claims {
        if dataset_claims.contains(claim_str) {
            precision_matched += 1.0;
        }
    }
    let precision = if result_claims.is_empty() {
        if case.facts.is_empty() { 1.0 } else { 0.0 }
    } else {
        precision_matched / result_claims.len() as f64
    };

    // 3. Completeness (Weighted sum of all facts)
    let mut total_weight = 0.0;
    let mut matched_weight = 0.0;
    for f in &case.facts {
        let weight = get_weight(&f.importance);
        total_weight += weight;
        let fact_str = format!("{}:{}", f.claim.kind, f.claim.id).to_lowercase();
        if result_claims.contains(&fact_str) {
            matched_weight += weight;
        }
    }
    let completeness = if total_weight == 0.0 { 1.0 } else { matched_weight / total_weight };

    // 4. Evidence Coverage (Weighted)
    let mut ev_total_weight = 0.0;
    let mut ev_matched_weight = 0.0;
    for ev in &case.expected_evidence {
        let weight = get_weight(&ev.importance);
        ev_total_weight += weight;
        if result.runtime.evidence.iter().any(|re| re.id == ev.id) {
            ev_matched_weight += weight;
        } else if matches!(ev.importance, FactImportance::Required | FactImportance::Major) {
            failures.push(Failure {
                kind: FailureKind::MissingEvidence,
                description: format!("Missing required evidence: {}", ev.id),
            });
        }
    }
    let evidence_coverage = if ev_total_weight == 0.0 { 1.0 } else { ev_matched_weight / ev_total_weight };

    // 5. Traversal Match
    let mut traversal_matched = 0.0;
    for expected in &case.expected_traversal {
        if result.traversal.contains(expected) {
            traversal_matched += 1.0;
        } else {
            failures.push(Failure {
                kind: FailureKind::TraversalMismatch,
                description: format!("Missing expected traversal node: {}", expected),
            });
        }
    }
    let traversal_match = if case.expected_traversal.is_empty() { 1.0 } else { traversal_matched / case.expected_traversal.len() as f64 };

    // 6. Graph Coverage (Ratio of expected traversal nodes actually hit)
    // Simply proxy to traversal match for this basic metric.
    let graph_coverage = traversal_match;

    // 7. Hallucination Rate
    let mut hallucination_score: f64 = 0.0;
    
    for ev in &result.runtime.evidence {
        if !graph_nodes.contains(&ev.id) {
            failures.push(Failure {
                kind: FailureKind::Hallucination,
                description: format!("Invented evidence node: {}", ev.id),
            });
            hallucination_score += 0.2;
        }
    }
    for tr in &result.traversal {
        if !graph_nodes.contains(tr) {
            failures.push(Failure {
                kind: FailureKind::Hallucination,
                description: format!("Invented traversal node: {}", tr),
            });
            hallucination_score += 0.1;
        }
    }
    
    let hallucination_rate = hallucination_score.min(1.0);
    let penalty = (hallucination_rate * 2.0).min(0.50);

    // 8. Overall Score
    let overall = (0.25 * recall) 
        + (0.15 * precision) 
        + (0.20 * evidence_coverage) 
        + (0.10 * completeness) 
        + (0.10 * traversal_match)
        + (0.10 * graph_coverage)
        + (0.10 * result.runtime.confidence) 
        - penalty;

    let overall = overall.max(0.0).min(1.0);

    Score {
        recall,
        precision,
        evidence_coverage,
        completeness,
        confidence: result.runtime.confidence,
        traversal_match,
        graph_coverage,
        hallucination_rate,
        failures,
        overall,
    }
}
