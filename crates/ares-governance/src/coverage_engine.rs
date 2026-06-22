use serde::{Deserialize, Serialize};
use ares_core::AresError;
use ares_store::{Store, SqliteGraphRepository};
use crate::classifier::{ArtifactClassifier, MemoryEligibility, ArtifactCategory};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CoverageMetric {
    pub covered: u64,
    pub total: u64,
    pub percentage: f64,
}

impl CoverageMetric {
    pub fn new(covered: u64, total: u64) -> Self {
        let percentage = if total > 0 {
            (covered as f64 / total as f64) * 100.0
        } else {
            100.0 // If there are 0 required items, coverage is technically perfect (no gaps)
        };
        
        Self {
            covered,
            total,
            percentage,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MemoryCoverageMetrics {
    pub overall: CoverageMetric,
    pub requirements: CoverageMetric,
    pub decisions: CoverageMetric,
    pub architecture: CoverageMetric,
    pub ownership: CoverageMetric,
    pub tests: CoverageMetric,
    pub evidence: CoverageMetric,
    pub capture_rate: MemoryCaptureRate,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MemoryCaptureRate {
    pub git_blame: bool,
    pub git_commits: bool,
    pub git_releases: bool,
    pub codeowners: bool,
    pub captured_sources: u32,
    pub available_sources: u32,
    pub rate: f64,
}

pub struct CoverageEngine;

impl CoverageEngine {
    pub fn calculate(store: &Store, project_id: &ares_core::ProjectId) -> Result<MemoryCoverageMetrics, AresError> {
        let repo = SqliteGraphRepository::new(store.clone());
        let nodes = repo.get_all_nodes(project_id)?;
        let edges = repo.get_all_edges(project_id)?;
        
        // 1. Filter Nodes and Classify
        let mut memory_eligible_count = 0;
        let mut memory_eligible_with_reasoning = 0;
        
        let mut req_total = 0;
        let mut req_covered = 0;
        
        let mut dec_total = 0;
        let mut dec_covered = 0;
        let mut dec_with_evidence = 0;
        
        let mut arch_total = 0;
        let mut arch_covered = 0;
        
        let mut ownership_total = 0;
        let mut ownership_covered = 0;
        
        let mut code_total = 0;
        let mut code_with_tests = 0;
        
        for node in &nodes {
            let classification = ArtifactClassifier::classify(Some(&node.node_type), node.file_path.as_deref());

            println!("DEBUG: Node: {} | Category: {:?} | Path: {:?}", node.id, classification.category, node.file_path);
            
            // Check Eligibility for the primary "Overall" metric
            if classification.eligibility == MemoryEligibility::Required || classification.eligibility == MemoryEligibility::Recommended {
                memory_eligible_count += 1;
                ownership_total += 1;
                
                // Has Owner?
                                let has_owner = edges.iter().any(|e| e.from_node_id == node.id && e.edge_type == ares_core::EdgeType::OwnedBy);
                if has_owner { ownership_covered += 1; } else { println!("DEBUG_MISSING_OWNER: Node: {} | Category: {:?} | Path: {:?}", node.id.as_str(), classification.category, node.file_path); }
                
                // Has Upstream Reasoning (Drives, Satisfies, etc)?
                let has_reasoning = edges.iter().any(|e| e.to_node_id == node.id && (e.edge_type == ares_core::EdgeType::Drives || e.edge_type == ares_core::EdgeType::Satisfies || e.edge_type == ares_core::EdgeType::Implements));
                if has_reasoning || classification.category == ArtifactCategory::Requirement || classification.category == ArtifactCategory::Decision || classification.category == ArtifactCategory::Architecture {
                    // Reasoning nodes themselves count as having reasoning (they are the reasoning). 
                    memory_eligible_with_reasoning += 1;
                }
            }
            
            // Track subscores based on specific categories
            match classification.category {
                ArtifactCategory::Requirement => {
                    req_total += 1;
                    // Requirements covered = linked to downstream code/decisions
                    let is_linked = edges.iter().any(|e| e.from_node_id == node.id && (e.edge_type == ares_core::EdgeType::Drives || e.edge_type == ares_core::EdgeType::Satisfies));
                    if is_linked { req_covered += 1; }
                }
                ArtifactCategory::Decision => {
                    dec_total += 1;
                    // Decision covered = linked to requirements AND code
                    let links_to_code = edges.iter().any(|e| e.from_node_id == node.id && e.edge_type == ares_core::EdgeType::Drives);
                    let linked_from_req = edges.iter().any(|e| e.to_node_id == node.id && e.edge_type == ares_core::EdgeType::Drives);
                    if links_to_code && linked_from_req { dec_covered += 1; }
                    
                    // Decision with evidence
                    let has_evidence = edges.iter().any(|e| e.to_node_id == node.id && e.edge_type == ares_core::EdgeType::SupportedBy);
                    if has_evidence { dec_with_evidence += 1; }
                }
                ArtifactCategory::Architecture => {
                    arch_total += 1;
                    let is_linked = edges.iter().any(|e| e.from_node_id == node.id || e.to_node_id == node.id);
                    if is_linked { arch_covered += 1; }
                }
                ArtifactCategory::Code => {
                    code_total += 1;
                    let has_test = edges.iter().any(|e| e.to_node_id == node.id && e.edge_type == ares_core::EdgeType::ValidatedBy);
                    if has_test { code_with_tests += 1; }
                }
                _ => {}
            }
        }
        
        // For Memory Capture Rate, we check if there are any nodes from these sources
        let has_blame = edges.iter().any(|e| e.source == "git_blame");
        let has_commits = nodes.iter().any(|n| n.node_type == ares_core::NodeType::Commit);
        let has_releases = nodes.iter().any(|n| n.node_type == ares_core::NodeType::Release);
        let has_codeowners = edges.iter().any(|e| e.source == "codeowners");
        
        let mut available = 0;
        let mut captured = 0;
        
        // Every repo has commits
        available += 1;
        if has_commits { captured += 1; }
        
        // Assume every repo can have blame
        available += 1;
        if has_blame { captured += 1; }
        
        // Assume every repo can have releases
        available += 1;
        if has_releases { captured += 1; }
        
        // Assume CODEOWNERS is available to all
        available += 1;
        if has_codeowners { captured += 1; }

        let capture_rate = MemoryCaptureRate {
            git_blame: has_blame,
            git_commits: has_commits,
            git_releases: has_releases,
            codeowners: has_codeowners,
            captured_sources: captured,
            available_sources: available,
            rate: if available > 0 { (captured as f64 / available as f64) * 100.0 } else { 0.0 },
        };

        println!("DEBUG_COVERAGE: eligible={}, req_tot={}, dec_tot={}, code_tot={}, own_cov={}, capture_rate={}%", 
            memory_eligible_count, req_total, dec_total, code_total, ownership_covered, capture_rate.rate);
            
        Ok(MemoryCoverageMetrics {
            overall: CoverageMetric::new(memory_eligible_with_reasoning, memory_eligible_count),
            requirements: CoverageMetric::new(req_covered, req_total),
            decisions: CoverageMetric::new(dec_covered, dec_total),
            architecture: CoverageMetric::new(arch_covered, arch_total),
            ownership: CoverageMetric::new(ownership_covered, ownership_total),
            tests: CoverageMetric::new(code_with_tests, code_total),
            evidence: CoverageMetric::new(dec_with_evidence, dec_total),
            capture_rate,
        })
    }
}

