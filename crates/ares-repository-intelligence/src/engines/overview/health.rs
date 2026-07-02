use super::models::RepositoryDashboardResponse;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthContribution {
    pub score_delta: i32,
    pub severity: String, // "Info", "Warning", "Critical", "Success"
    pub title: String,
    pub description: String,
}

pub trait HealthRule {
    fn score(&self, dash: &RepositoryDashboardResponse) -> HealthContribution;
}

pub struct GraphIntegrityRule;
impl HealthRule for GraphIntegrityRule {
    fn score(&self, dash: &RepositoryDashboardResponse) -> HealthContribution {
        if !dash.integrity.foreign_keys_passed {
            HealthContribution {
                score_delta: -15,
                severity: "Critical".to_string(),
                title: "Foreign Key Violations".to_string(),
                description: "The graph contains edges with missing source or target nodes."
                    .to_string(),
            }
        } else if dash.integrity.missing_sources > 0 || dash.integrity.missing_targets > 0 {
            HealthContribution {
                score_delta: -10,
                severity: "Warning".to_string(),
                title: "Missing References".to_string(),
                description: format!(
                    "Found {} missing sources and {} missing targets.",
                    dash.integrity.missing_sources, dash.integrity.missing_targets
                ),
            }
        } else {
            HealthContribution {
                score_delta: 5,
                severity: "Success".to_string(),
                title: "Graph Integrity Clean".to_string(),
                description: "All relationships in the graph are perfectly intact.".to_string(),
            }
        }
    }
}

pub struct CoverageRule;
impl HealthRule for CoverageRule {
    fn score(&self, dash: &RepositoryDashboardResponse) -> HealthContribution {
        if dash.coverage.requirements == 0 && dash.coverage.adrs == 0 {
            HealthContribution {
                score_delta: -5,
                severity: "Warning".to_string(),
                title: "Missing Documentation".to_string(),
                description: "No Requirements or ADRs found in the knowledge graph.".to_string(),
            }
        } else {
            HealthContribution {
                score_delta: 2,
                severity: "Success".to_string(),
                title: "Documentation Found".to_string(),
                description: "Knowledge graph contains structural documentation.".to_string(),
            }
        }
    }
}

pub struct OrphanRule;
impl HealthRule for OrphanRule {
    fn score(&self, dash: &RepositoryDashboardResponse) -> HealthContribution {
        if dash.integrity.orphans > 10 {
            HealthContribution {
                score_delta: -((dash.integrity.orphans / 10) as i32).min(10),
                severity: "Warning".to_string(),
                title: "High Orphan Count".to_string(),
                description: format!(
                    "Found {} unconnected nodes in the graph.",
                    dash.integrity.orphans
                ),
            }
        } else {
            HealthContribution {
                score_delta: 2,
                severity: "Success".to_string(),
                title: "Low Orphan Count".to_string(),
                description: "Most nodes are properly connected.".to_string(),
            }
        }
    }
}

pub fn evaluate_health(dash: &RepositoryDashboardResponse) -> (i32, Vec<HealthContribution>) {
    let rules: Vec<Box<dyn HealthRule>> = vec![
        Box::new(GraphIntegrityRule),
        Box::new(CoverageRule),
        Box::new(OrphanRule),
    ];

    let mut score = 90; // Base score
    let mut contributions = Vec::new();

    for rule in rules {
        let contrib = rule.score(dash);
        score += contrib.score_delta;
        contributions.push(contrib);
    }

    // Clamp score
    let score = score.clamp(0, 100);
    (score, contributions)
}
