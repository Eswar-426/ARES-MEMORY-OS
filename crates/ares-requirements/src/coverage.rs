use crate::models::{RequirementStatus};
use ares_core::RequirementId;
use ares_traceability::{TraceabilityGraph, TraceTargetType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CoverageStatus {
    Orphaned,
    Partial,
    Covered,
    Verified,
}


#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct RequirementGap {
    pub requirement_id: RequirementId,
    pub gap_type: crate::gaps::KnowledgeGapType,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct RequirementCoverage {
    pub approved: bool,
    pub implemented: bool,
    pub verified: bool,
    pub monitored: bool,
    pub coverage_score: f32,
    pub status: CoverageStatus,
    pub gaps: Vec<RequirementGap>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct RequirementCoverageSummary {
    pub total_requirements: usize,
    pub fully_covered: usize,
    pub partially_covered: usize,
    pub orphaned: usize,
    pub average_coverage: f32,
}


#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct RequirementCoverageTrend {
    pub previous_coverage: f32,
    pub current_coverage: f32,
    pub delta: f32,
}

pub struct RequirementCoverageEngine {
    // Engine dependencies like Store can be added here
}

impl RequirementCoverageEngine {
    pub fn new() -> Self {
        Self {}
    }

    pub fn evaluate(
        &self,
        req_id: &RequirementId,
        req_status: &RequirementStatus,
        has_owner: bool,
        resolver: &crate::trace_analysis::TraceAnalysisEngine,
    ) -> RequirementCoverage {
        let mut gaps = Vec::new();

        let approved = matches!(req_status, RequirementStatus::Approved | RequirementStatus::Implemented | RequirementStatus::Verified);

        let implemented = resolver.has_implementation(req_id.as_str());
        let verified = resolver.has_test(req_id.as_str());
        let monitored = resolver.has_runtime_metric(req_id.as_str());
        let has_decision = resolver.has_decision(req_id.as_str());

        if !has_owner {
            gaps.push(RequirementGap {
                requirement_id: req_id.clone(),
                gap_type: crate::gaps::KnowledgeGapType::MissingOwner,
            });
        }
        if !approved {
            gaps.push(RequirementGap {
                requirement_id: req_id.clone(),
                gap_type: crate::gaps::KnowledgeGapType::UnapprovedRequirement,
            });
        }
        if !has_decision {
            gaps.push(RequirementGap {
                requirement_id: req_id.clone(),
                gap_type: crate::gaps::KnowledgeGapType::MissingDecision,
            });
        }
        if !implemented {
            gaps.push(RequirementGap {
                requirement_id: req_id.clone(),
                gap_type: crate::gaps::KnowledgeGapType::MissingImplementation,
            });
        }
        if !verified {
            gaps.push(RequirementGap {
                requirement_id: req_id.clone(),
                gap_type: crate::gaps::KnowledgeGapType::MissingTest,
            });
        }
        if !monitored {
            gaps.push(RequirementGap {
                requirement_id: req_id.clone(),
                gap_type: crate::gaps::KnowledgeGapType::MissingRuntimeMetric,
            });
        }

        let mut score = 0.0;
        if has_decision {
            score = 25.0;
            if implemented {
                score = 50.0;
                if verified {
                    score = 75.0;
                    if monitored {
                        score = 100.0;
                    }
                }
            }
        }

        let status = match score as i32 {
            0 => CoverageStatus::Orphaned,
            1..=99 => {
                if implemented && verified && !monitored {
                    CoverageStatus::Covered
                } else {
                    CoverageStatus::Partial
                }
            },
            _ => CoverageStatus::Verified,
        };

        RequirementCoverage {
            approved,
            implemented,
            verified,
            monitored,
            coverage_score: score,
            status,
            gaps,
        }
    }

    pub fn generate_summary(&self, coverages: &[RequirementCoverage]) -> (RequirementCoverageSummary, Vec<crate::gaps::GapSummary>) {
        let total = coverages.len();
        let mut fully_covered = 0;
        let mut partially_covered = 0;
        let mut orphaned = 0;
        let mut sum_score = 0.0;

        let mut gap_counts = std::collections::HashMap::new();

        for c in coverages {
            match c.status {
                CoverageStatus::Verified | CoverageStatus::Covered => fully_covered += 1, // Depending on if we consider "Covered" fully covered. Let's say Verified is fully. Wait, the user said Level 4 is 100%. Let's check score == 100.0.
                CoverageStatus::Partial => partially_covered += 1,
                CoverageStatus::Orphaned => orphaned += 1,
            }
            if c.coverage_score == 100.0 {
                // If we want fully covered to be strictly 100%
            }
            
            sum_score += c.coverage_score;

            for gap in &c.gaps {
                *gap_counts.entry(gap.gap_type.clone()).or_insert(0) += 1;
            }
        }
        
        // Recalculate fully_covered as score == 100.0
        fully_covered = coverages.iter().filter(|c| c.coverage_score == 100.0).count();
        partially_covered = coverages.iter().filter(|c| c.coverage_score > 0.0 && c.coverage_score < 100.0).count();
        orphaned = coverages.iter().filter(|c| c.coverage_score == 0.0).count();

        let average_coverage = if total > 0 { sum_score / total as f32 } else { 0.0 };

        let mut top_gaps: Vec<crate::gaps::GapSummary> = gap_counts
            .into_iter()
            .map(|(gap_type, count)| crate::gaps::GapSummary { gap_type, count })
            .collect();
        top_gaps.sort_by(|a, b| b.count.cmp(&a.count));

        (
            RequirementCoverageSummary {
                total_requirements: total,
                fully_covered,
                partially_covered,
                orphaned,
                average_coverage,
            },
            top_gaps,
        )
    }
}
