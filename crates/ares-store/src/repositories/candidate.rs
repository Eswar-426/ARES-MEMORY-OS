use std::str::FromStr;
use async_trait::async_trait;
use rusqlite::{params, OptionalExtension};

use ares_candidates::{
    Candidate, CandidateConfidence, CandidatePromotion, CandidateRepository, CandidateReview,
    CandidateSource, CandidateStatus, CandidateType, DecisionCategory, ArchitectureCategory,
    TraceabilityCategory, TraceabilityEndpointType, TraceabilityEndpoint, TraceabilityStrength,
};
use ares_core::{GraphEdge, GraphNode};

use crate::db::Store;

pub struct SqliteCandidateRepository {
    store: Store,
}

impl SqliteCandidateRepository {
    pub fn new(store: Store) -> Self {
        Self { store }
    }
}

#[async_trait]
impl CandidateRepository for SqliteCandidateRepository {
    // ----------------------------------------------------------------
    // Candidates
    // ----------------------------------------------------------------

    async fn insert_candidate(&self, candidate: &Candidate) -> Result<(), String> {
        let conn = self.store.get_conn().map_err(|e| e.to_string())?;

        let type_str = match candidate.candidate_type {
            CandidateType::Requirement => "Requirement",
            CandidateType::Decision => "Decision",
            CandidateType::Architecture => "Architecture",
            CandidateType::Traceability => "Traceability",
        };

        let status_str = match candidate.status {
            CandidateStatus::Proposed => "Proposed",
            CandidateStatus::UnderReview => "UnderReview",
            CandidateStatus::Approved => "Approved",
            CandidateStatus::Rejected => "Rejected",
            CandidateStatus::Superseded => "Superseded",
        };

        
        let architecture_category_str = match &candidate.architecture_category {
            Some(ArchitectureCategory::Service) => Some("Service"),
            Some(ArchitectureCategory::Component) => Some("Component"),
            Some(ArchitectureCategory::Module) => Some("Module"),
            Some(ArchitectureCategory::Workspace) => Some("Workspace"),
            Some(ArchitectureCategory::Domain) => Some("Domain"),
            Some(ArchitectureCategory::Integration) => Some("Integration"),
            None => None,
        };

        let dependent_components_str = candidate.dependent_components.join(",");
        let ownership_domains_str = candidate.ownership_domains.join(",");


        let traceability_category_str = match &candidate.traceability_category {
            Some(TraceabilityCategory::RequirementToDecision) => Some("RequirementToDecision"),
            Some(TraceabilityCategory::DecisionToArchitecture) => Some("DecisionToArchitecture"),
            Some(TraceabilityCategory::ArchitectureToCode) => Some("ArchitectureToCode"),
            Some(TraceabilityCategory::RequirementToCode) => Some("RequirementToCode"),
            None => None,
        };

        let (source_endpoint_type, source_endpoint_id) = match &candidate.source_endpoint {
            Some(ep) => {
                let ep_type = match ep.endpoint_type {
                    TraceabilityEndpointType::Candidate => "Candidate",
                    TraceabilityEndpointType::GraphNode => "GraphNode",
                    TraceabilityEndpointType::File => "File",
                    TraceabilityEndpointType::Commit => "Commit",
                };
                (Some(ep_type), Some(ep.endpoint_id.as_str()))
            },
            None => (None, None),
        };

        let (target_endpoint_type, target_endpoint_id) = match &candidate.target_endpoint {
            Some(ep) => {
                let ep_type = match ep.endpoint_type {
                    TraceabilityEndpointType::Candidate => "Candidate",
                    TraceabilityEndpointType::GraphNode => "GraphNode",
                    TraceabilityEndpointType::File => "File",
                    TraceabilityEndpointType::Commit => "Commit",
                };
                (Some(ep_type), Some(ep.endpoint_id.as_str()))
            },
            None => (None, None),
        };

        let traceability_strength_str = match &candidate.traceability_strength {
            Some(TraceabilityStrength::Weak) => Some("Weak"),
            Some(TraceabilityStrength::Moderate) => Some("Moderate"),
            Some(TraceabilityStrength::Strong) => Some("Strong"),
            Some(TraceabilityStrength::Definitive) => Some("Definitive"),
            None => None,
        };

        let decision_category_str = match &candidate.decision_category {
            Some(DecisionCategory::TechnologyAdoption) => Some("TechnologyAdoption"),
            Some(DecisionCategory::TechnologyRemoval) => Some("TechnologyRemoval"),
            Some(DecisionCategory::DependencyMigration) => Some("DependencyMigration"),
            Some(DecisionCategory::ArchitectureChange) => Some("ArchitectureChange"),
            Some(DecisionCategory::PlatformChoice) => Some("PlatformChoice"),
            None => None,
        };

        conn.execute(
            "INSERT INTO candidates (
                id, project_id, title, description, candidate_type, status,
                evidence_count, source_diversity, temporal_consistency, cluster_strength,
                created_at, updated_at, decision_category, architecture_category, dependent_components, ownership_domains, traceability_category, source_endpoint_type, source_endpoint_id, target_endpoint_type, target_endpoint_id, traceability_strength
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22)",
            params![
                candidate.id,
                candidate.project_id,
                candidate.title,
                candidate.description,
                type_str,
                status_str,
                candidate.confidence.evidence_count,
                candidate.confidence.source_diversity,
                candidate.confidence.temporal_consistency,
                candidate.confidence.cluster_strength,
                candidate.created_at,
                candidate.updated_at,
                decision_category_str,
                architecture_category_str,
                dependent_components_str,
                ownership_domains_str,
                traceability_category_str,
                source_endpoint_type,
                source_endpoint_id,
                target_endpoint_type,
                target_endpoint_id,
                traceability_strength_str,
            ],
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    async fn get_candidate(&self, project_id: &str, id: &str) -> Result<Option<Candidate>, String> {
        let conn = self.store.get_conn().map_err(|e| e.to_string())?;

        let candidate = conn
            .query_row(
                "SELECT id, project_id, title, description, candidate_type, status,
                 evidence_count, source_diversity, temporal_consistency, cluster_strength,
                 created_at, updated_at, decision_category, architecture_category, dependent_components, ownership_domains, traceability_category, source_endpoint_type, source_endpoint_id, target_endpoint_type, target_endpoint_id, traceability_strength
                 FROM candidates WHERE project_id = ?1 AND id = ?2",
                params![project_id, id],
                |row| {
                    let c_type_str: String = row.get(4)?;
                    let status_str: String = row.get(5)?;

                    let c_type = match c_type_str.as_str() {
                        "Requirement" => CandidateType::Requirement,
                        "Decision" => CandidateType::Decision,
                        "Architecture" => CandidateType::Architecture,
                        "Traceability" => CandidateType::Traceability,
                        _ => CandidateType::Requirement,
                    };

                    let status = match status_str.as_str() {
                        "Proposed" => CandidateStatus::Proposed,
                        "UnderReview" => CandidateStatus::UnderReview,
                        "Approved" => CandidateStatus::Approved,
                        "Rejected" => CandidateStatus::Rejected,
                        "Superseded" => CandidateStatus::Superseded,
                        _ => CandidateStatus::Proposed,
                    };

                    let dec_cat_str: Option<String> = row.get(12)?;
                    let decision_category = dec_cat_str.and_then(|s| match s.as_str() {
                        "TechnologyAdoption" => Some(DecisionCategory::TechnologyAdoption),
                        "TechnologyRemoval" => Some(DecisionCategory::TechnologyRemoval),
                        "DependencyMigration" => Some(DecisionCategory::DependencyMigration),
                        "ArchitectureChange" => Some(DecisionCategory::ArchitectureChange),
                        "PlatformChoice" => Some(DecisionCategory::PlatformChoice),
                        _ => None,
                    });

                    
                    let arch_cat_str: Option<String> = row.get(13)?;
                    let architecture_category = arch_cat_str.and_then(|s| match s.as_str() {
                        "Service" => Some(ArchitectureCategory::Service),
                        "Component" => Some(ArchitectureCategory::Component),
                        "Module" => Some(ArchitectureCategory::Module),
                        "Workspace" => Some(ArchitectureCategory::Workspace),
                        "Domain" => Some(ArchitectureCategory::Domain),
                        "Integration" => Some(ArchitectureCategory::Integration),
                        _ => None,
                    });

                    let dep_comp: String = row.get(14).unwrap_or_default();
                    let owner_dom: String = row.get(15).unwrap_or_default();
                    let dependent_components = if dep_comp.is_empty() { vec![] } else { dep_comp.split(",").map(|s| s.to_string()).collect() };
                    let ownership_domains = if owner_dom.is_empty() { vec![] } else { owner_dom.split(",").map(|s| s.to_string()).collect() };


                    let trace_cat_str: Option<String> = row.get(16).unwrap_or(None);
                    let traceability_category = trace_cat_str.and_then(|s| match s.as_str() {
                        "RequirementToDecision" => Some(TraceabilityCategory::RequirementToDecision),
                        "DecisionToArchitecture" => Some(TraceabilityCategory::DecisionToArchitecture),
                        "ArchitectureToCode" => Some(TraceabilityCategory::ArchitectureToCode),
                        "RequirementToCode" => Some(TraceabilityCategory::RequirementToCode),
                        _ => None,
                    });

                    let src_ep_type_str: Option<String> = row.get(17).unwrap_or(None);
                    let src_ep_id_str: Option<String> = row.get(18).unwrap_or(None);
                    let source_endpoint = match (src_ep_type_str, src_ep_id_str) {
                        (Some(t), Some(id)) => {
                            let ep_type = match t.as_str() {
                                "Candidate" => TraceabilityEndpointType::Candidate,
                                "GraphNode" => TraceabilityEndpointType::GraphNode,
                                "File" => TraceabilityEndpointType::File,
                                "Commit" => TraceabilityEndpointType::Commit,
                                _ => TraceabilityEndpointType::Candidate,
                            };
                            Some(TraceabilityEndpoint { endpoint_type: ep_type, endpoint_id: id })
                        },
                        _ => None,
                    };

                    let tgt_ep_type_str: Option<String> = row.get(19).unwrap_or(None);
                    let tgt_ep_id_str: Option<String> = row.get(20).unwrap_or(None);
                    let target_endpoint = match (tgt_ep_type_str, tgt_ep_id_str) {
                        (Some(t), Some(id)) => {
                            let ep_type = match t.as_str() {
                                "Candidate" => TraceabilityEndpointType::Candidate,
                                "GraphNode" => TraceabilityEndpointType::GraphNode,
                                "File" => TraceabilityEndpointType::File,
                                "Commit" => TraceabilityEndpointType::Commit,
                                _ => TraceabilityEndpointType::Candidate,
                            };
                            Some(TraceabilityEndpoint { endpoint_type: ep_type, endpoint_id: id })
                        },
                        _ => None,
                    };

                    let trace_strength_str: Option<String> = row.get(21).unwrap_or(None);
                    let traceability_strength = trace_strength_str.and_then(|s| match s.as_str() {
                        "Weak" => Some(TraceabilityStrength::Weak),
                        "Moderate" => Some(TraceabilityStrength::Moderate),
                        "Strong" => Some(TraceabilityStrength::Strong),
                        "Definitive" => Some(TraceabilityStrength::Definitive),
                        _ => None,
                    });

                    Ok(Candidate {
                        id: row.get(0)?,
                        project_id: row.get(1)?,
                        title: row.get(2)?,
                        description: row.get(3)?,
                        candidate_type: c_type,
                        decision_category,
                        architecture_category,
                        traceability_category,
                        source_endpoint,
                        target_endpoint,
                        traceability_strength,
                        dependent_components,
                        ownership_domains,
                        status,
                        confidence: CandidateConfidence {
                            evidence_count: row.get(6)?,
                            source_diversity: row.get(7)?,
                            temporal_consistency: row.get(8)?,
                            cluster_strength: row.get(9)?,
                        },
                        created_at: row.get(10)?,
                        updated_at: row.get(11)?,
                    })
                },
            )
            .optional()
            .map_err(|e| e.to_string())?;

        Ok(candidate)
    }

    async fn update_candidate(&self, candidate: &Candidate) -> Result<(), String> {
        let conn = self.store.get_conn().map_err(|e| e.to_string())?;

        let type_str = match candidate.candidate_type {
            CandidateType::Requirement => "Requirement",
            CandidateType::Decision => "Decision",
            CandidateType::Architecture => "Architecture",
            CandidateType::Traceability => "Traceability",
        };

        let status_str = match candidate.status {
            CandidateStatus::Proposed => "Proposed",
            CandidateStatus::UnderReview => "UnderReview",
            CandidateStatus::Approved => "Approved",
            CandidateStatus::Rejected => "Rejected",
            CandidateStatus::Superseded => "Superseded",
        };

        
        let architecture_category_str = match &candidate.architecture_category {
            Some(ArchitectureCategory::Service) => Some("Service"),
            Some(ArchitectureCategory::Component) => Some("Component"),
            Some(ArchitectureCategory::Module) => Some("Module"),
            Some(ArchitectureCategory::Workspace) => Some("Workspace"),
            Some(ArchitectureCategory::Domain) => Some("Domain"),
            Some(ArchitectureCategory::Integration) => Some("Integration"),
            None => None,
        };

        let dependent_components_str = candidate.dependent_components.join(",");
        let ownership_domains_str = candidate.ownership_domains.join(",");


        let traceability_category_str = match &candidate.traceability_category {
            Some(TraceabilityCategory::RequirementToDecision) => Some("RequirementToDecision"),
            Some(TraceabilityCategory::DecisionToArchitecture) => Some("DecisionToArchitecture"),
            Some(TraceabilityCategory::ArchitectureToCode) => Some("ArchitectureToCode"),
            Some(TraceabilityCategory::RequirementToCode) => Some("RequirementToCode"),
            None => None,
        };

        let (source_endpoint_type, source_endpoint_id) = match &candidate.source_endpoint {
            Some(ep) => {
                let ep_type = match ep.endpoint_type {
                    TraceabilityEndpointType::Candidate => "Candidate",
                    TraceabilityEndpointType::GraphNode => "GraphNode",
                    TraceabilityEndpointType::File => "File",
                    TraceabilityEndpointType::Commit => "Commit",
                };
                (Some(ep_type), Some(ep.endpoint_id.as_str()))
            },
            None => (None, None),
        };

        let (target_endpoint_type, target_endpoint_id) = match &candidate.target_endpoint {
            Some(ep) => {
                let ep_type = match ep.endpoint_type {
                    TraceabilityEndpointType::Candidate => "Candidate",
                    TraceabilityEndpointType::GraphNode => "GraphNode",
                    TraceabilityEndpointType::File => "File",
                    TraceabilityEndpointType::Commit => "Commit",
                };
                (Some(ep_type), Some(ep.endpoint_id.as_str()))
            },
            None => (None, None),
        };

        let traceability_strength_str = match &candidate.traceability_strength {
            Some(TraceabilityStrength::Weak) => Some("Weak"),
            Some(TraceabilityStrength::Moderate) => Some("Moderate"),
            Some(TraceabilityStrength::Strong) => Some("Strong"),
            Some(TraceabilityStrength::Definitive) => Some("Definitive"),
            None => None,
        };

        let decision_category_str = match &candidate.decision_category {
            Some(DecisionCategory::TechnologyAdoption) => Some("TechnologyAdoption"),
            Some(DecisionCategory::TechnologyRemoval) => Some("TechnologyRemoval"),
            Some(DecisionCategory::DependencyMigration) => Some("DependencyMigration"),
            Some(DecisionCategory::ArchitectureChange) => Some("ArchitectureChange"),
            Some(DecisionCategory::PlatformChoice) => Some("PlatformChoice"),
            None => None,
        };

        conn.execute(
            "UPDATE candidates SET
                project_id = ?2, title = ?3, description = ?4, candidate_type = ?5, status = ?6,
                evidence_count = ?7, source_diversity = ?8, temporal_consistency = ?9, cluster_strength = ?10,
                updated_at = ?11, decision_category = ?12, architecture_category = ?13, dependent_components = ?14, ownership_domains = ?15, traceability_category = ?16, source_endpoint_type = ?17, source_endpoint_id = ?18, target_endpoint_type = ?19, target_endpoint_id = ?20, traceability_strength = ?21
             WHERE id = ?1",
            params![
                candidate.id,
                candidate.project_id,
                candidate.title,
                candidate.description,
                type_str,
                status_str,
                candidate.confidence.evidence_count,
                candidate.confidence.source_diversity,
                candidate.confidence.temporal_consistency,
                candidate.confidence.cluster_strength,
                candidate.updated_at,
                decision_category_str,
                architecture_category_str,
                dependent_components_str,
                ownership_domains_str,
                traceability_category_str,
                source_endpoint_type,
                source_endpoint_id,
                target_endpoint_type,
                target_endpoint_id,
                traceability_strength_str,
            ],
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    async fn list_candidates(
        &self,
        project_id: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Candidate>, String> {
        let conn = self.store.get_conn().map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare(
                "SELECT id, project_id, title, description, candidate_type, status,
                 evidence_count, source_diversity, temporal_consistency, cluster_strength,
                 created_at, updated_at, decision_category, architecture_category, dependent_components, ownership_domains, traceability_category, source_endpoint_type, source_endpoint_id, target_endpoint_type, target_endpoint_id, traceability_strength
                 FROM candidates WHERE project_id = ?1 ORDER BY created_at DESC LIMIT ?2 OFFSET ?3",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![project_id, limit as i64, offset as i64], |row| {
                let c_type_str: String = row.get(4)?;
                let status_str: String = row.get(5)?;

                let c_type = match c_type_str.as_str() {
                    "Requirement" => CandidateType::Requirement,
                    "Decision" => CandidateType::Decision,
                    "Architecture" => CandidateType::Architecture,
                    "Traceability" => CandidateType::Traceability,
                    _ => CandidateType::Requirement,
                };

                let status = match status_str.as_str() {
                    "Proposed" => CandidateStatus::Proposed,
                    "UnderReview" => CandidateStatus::UnderReview,
                    "Approved" => CandidateStatus::Approved,
                    "Rejected" => CandidateStatus::Rejected,
                    "Superseded" => CandidateStatus::Superseded,
                    _ => CandidateStatus::Proposed,
                };

                let dec_cat_str: Option<String> = row.get(12)?;
                let decision_category = dec_cat_str.and_then(|s| match s.as_str() {
                    "TechnologyAdoption" => Some(DecisionCategory::TechnologyAdoption),
                    "TechnologyRemoval" => Some(DecisionCategory::TechnologyRemoval),
                    "DependencyMigration" => Some(DecisionCategory::DependencyMigration),
                    "ArchitectureChange" => Some(DecisionCategory::ArchitectureChange),
                    "PlatformChoice" => Some(DecisionCategory::PlatformChoice),
                    _ => None,
                });

                let dep_comp: String = row.get(14).unwrap_or_default();
                let owner_dom: String = row.get(15).unwrap_or_default();

                
                    let arch_cat_str: Option<String> = row.get(13)?;
                    let architecture_category = arch_cat_str.and_then(|s| match s.as_str() {
                        "Service" => Some(ArchitectureCategory::Service),
                        "Component" => Some(ArchitectureCategory::Component),
                        "Module" => Some(ArchitectureCategory::Module),
                        "Workspace" => Some(ArchitectureCategory::Workspace),
                        "Domain" => Some(ArchitectureCategory::Domain),
                        "Integration" => Some(ArchitectureCategory::Integration),
                        _ => None,
                    });

                    let dep_comp: String = row.get(14).unwrap_or_default();
                    let owner_dom: String = row.get(15).unwrap_or_default();
                    let dependent_components = if dep_comp.is_empty() { vec![] } else { dep_comp.split(",").map(|s| s.to_string()).collect() };
                    let ownership_domains = if owner_dom.is_empty() { vec![] } else { owner_dom.split(",").map(|s| s.to_string()).collect() };


                    let trace_cat_str: Option<String> = row.get(16).unwrap_or(None);
                    let traceability_category = trace_cat_str.and_then(|s| match s.as_str() {
                        "RequirementToDecision" => Some(TraceabilityCategory::RequirementToDecision),
                        "DecisionToArchitecture" => Some(TraceabilityCategory::DecisionToArchitecture),
                        "ArchitectureToCode" => Some(TraceabilityCategory::ArchitectureToCode),
                        "RequirementToCode" => Some(TraceabilityCategory::RequirementToCode),
                        _ => None,
                    });

                    let src_ep_type_str: Option<String> = row.get(17).unwrap_or(None);
                    let src_ep_id_str: Option<String> = row.get(18).unwrap_or(None);
                    let source_endpoint = match (src_ep_type_str, src_ep_id_str) {
                        (Some(t), Some(id)) => {
                            let ep_type = match t.as_str() {
                                "Candidate" => TraceabilityEndpointType::Candidate,
                                "GraphNode" => TraceabilityEndpointType::GraphNode,
                                "File" => TraceabilityEndpointType::File,
                                "Commit" => TraceabilityEndpointType::Commit,
                                _ => TraceabilityEndpointType::Candidate,
                            };
                            Some(TraceabilityEndpoint { endpoint_type: ep_type, endpoint_id: id })
                        },
                        _ => None,
                    };

                    let tgt_ep_type_str: Option<String> = row.get(19).unwrap_or(None);
                    let tgt_ep_id_str: Option<String> = row.get(20).unwrap_or(None);
                    let target_endpoint = match (tgt_ep_type_str, tgt_ep_id_str) {
                        (Some(t), Some(id)) => {
                            let ep_type = match t.as_str() {
                                "Candidate" => TraceabilityEndpointType::Candidate,
                                "GraphNode" => TraceabilityEndpointType::GraphNode,
                                "File" => TraceabilityEndpointType::File,
                                "Commit" => TraceabilityEndpointType::Commit,
                                _ => TraceabilityEndpointType::Candidate,
                            };
                            Some(TraceabilityEndpoint { endpoint_type: ep_type, endpoint_id: id })
                        },
                        _ => None,
                    };

                    let trace_strength_str: Option<String> = row.get(21).unwrap_or(None);
                    let traceability_strength = trace_strength_str.and_then(|s| match s.as_str() {
                        "Weak" => Some(TraceabilityStrength::Weak),
                        "Moderate" => Some(TraceabilityStrength::Moderate),
                        "Strong" => Some(TraceabilityStrength::Strong),
                        "Definitive" => Some(TraceabilityStrength::Definitive),
                        _ => None,
                    });

                    Ok(Candidate {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    title: row.get(2)?,
                    description: row.get(3)?,
                    candidate_type: c_type,
                    decision_category,
                    architecture_category,
                    traceability_category,
                    source_endpoint,
                    target_endpoint,
                    traceability_strength,
                    dependent_components,
                    ownership_domains,
                    status,
                    confidence: CandidateConfidence {
                        evidence_count: row.get(6)?,
                        source_diversity: row.get(7)?,
                        temporal_consistency: row.get(8)?,
                        cluster_strength: row.get(9)?,
                    },
                    created_at: row.get(10)?,
                    updated_at: row.get(11)?,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r.map_err(|e| e.to_string())?);
        }
        Ok(results)
    }

    // ----------------------------------------------------------------
    // Candidate Sources
    // ----------------------------------------------------------------

    async fn insert_source(&self, source: &CandidateSource) -> Result<(), String> {
        let conn = self.store.get_conn().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO candidate_sources (id, candidate_id, source_type, source_id, confidence)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                source.id,
                source.candidate_id,
                source.source_type,
                source.source_id,
                source.confidence
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn get_sources(&self, project_id: &str, candidate_id: &str) -> Result<Vec<CandidateSource>, String> {
        let conn = self.store.get_conn().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT s.id, s.candidate_id, s.source_type, s.source_id, s.confidence
                 FROM candidate_sources s
                 JOIN candidates c ON s.candidate_id = c.id
                 WHERE c.project_id = ?1 AND s.candidate_id = ?2",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![project_id, candidate_id], |row| {
                Ok(CandidateSource {
                    id: row.get(0)?,
                    candidate_id: row.get(1)?,
                    source_type: row.get(2)?,
                    source_id: row.get(3)?,
                    confidence: row.get(4)?,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r.map_err(|e| e.to_string())?);
        }
        Ok(results)
    }

    // ----------------------------------------------------------------
    // Reviews
    // ----------------------------------------------------------------

    async fn insert_review(&self, review: &CandidateReview) -> Result<(), String> {
        let conn = self.store.get_conn().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO candidate_reviews (id, candidate_id, reviewer, comment, status_changed_to, review_date)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                review.id,
                review.candidate_id,
                review.reviewer,
                review.comment,
                review.status_changed_to,
                review.review_date
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn get_reviews(&self, project_id: &str, candidate_id: &str) -> Result<Vec<CandidateReview>, String> {
        let conn = self.store.get_conn().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT r.id, r.candidate_id, r.reviewer, r.comment, r.status_changed_to, r.review_date
                 FROM candidate_reviews r
                 JOIN candidates c ON r.candidate_id = c.id
                 WHERE c.project_id = ?1 AND r.candidate_id = ?2",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![project_id, candidate_id], |row| {
                Ok(CandidateReview {
                    id: row.get(0)?,
                    candidate_id: row.get(1)?,
                    reviewer: row.get(2)?,
                    comment: row.get(3)?,
                    status_changed_to: row.get(4)?,
                    review_date: row.get(5)?,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r.map_err(|e| e.to_string())?);
        }
        Ok(results)
    }

    // ----------------------------------------------------------------
    // Promotions
    // ----------------------------------------------------------------

    async fn insert_promotion(&self, promotion: &CandidatePromotion) -> Result<(), String> {
        let conn = self.store.get_conn().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO candidate_promotions (id, candidate_id, promoted_node_id, promoted_by, promoted_at, promotion_reason)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                promotion.id,
                promotion.candidate_id,
                promotion.promoted_node_id.as_str(),
                promotion.promoted_by,
                promotion.promoted_at,
                promotion.promotion_reason
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn get_promotion(&self, project_id: &str, candidate_id: &str) -> Result<Option<CandidatePromotion>, String> {
        let conn = self.store.get_conn().map_err(|e| e.to_string())?;
        let promotion = conn
            .query_row(
                "SELECT p.id, p.candidate_id, p.promoted_node_id, p.promoted_by, p.promoted_at, p.promotion_reason
                 FROM candidate_promotions p
                 JOIN candidates c ON p.candidate_id = c.id
                 WHERE c.project_id = ?1 AND p.candidate_id = ?2",
                params![project_id, candidate_id],
                |row| {
                    Ok(CandidatePromotion {
                        id: row.get(0)?,
                        candidate_id: row.get(1)?,
                        promoted_node_id: ares_core::NodeId::from(row.get::<_, String>(2)?),
                        promoted_by: row.get(3)?,
                        promoted_at: row.get(4)?,
                        promotion_reason: row.get(5)?,
                    })
                },
            )
            .optional()
            .map_err(|e| e.to_string())?;
        Ok(promotion)
    }

    // ----------------------------------------------------------------
    // Transactional Promotion
    // ----------------------------------------------------------------

    async fn promote_candidate(
        &self,
        candidate: &Candidate,
        promotion: &CandidatePromotion,
        node: &GraphNode,
        edges: &[GraphEdge],
    ) -> Result<(), String> {
        if candidate.project_id != node.project_id.as_str() {
            return Err("Repository mismatch: Candidate and Node must belong to the same repository.".to_string());
        }

        let mut conn = self.store.get_conn().map_err(|e| e.to_string())?;

        // Candidate Evidence Completeness Rule
        let evidence_count: i64 = conn.query_row(
            "SELECT count(*) FROM candidate_sources WHERE candidate_id = ?1",
            params![candidate.id],
            |row| row.get(0)
        ).unwrap_or(0);

        if evidence_count == 0 {
            return Err("Promotion rejected: Candidate has no evidence sources. ARES requires evidence for all memory promotions.".to_string());
        }

        let tx = conn.transaction().map_err(|e| e.to_string())?;

        // 1. Create Authoritative Node
        tx.execute(
            "INSERT INTO graph_nodes (id, project_id, node_type, label, properties, file_path, created_at, updated_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8)
             ON CONFLICT(id) DO UPDATE SET
               label      = excluded.label,
               properties = excluded.properties,
               updated_at = excluded.updated_at,
               deleted_at = NULL",
            params![
                node.id.as_str(),
                node.project_id.as_str(),
                node.node_type.as_str(),
                node.label,
                node.properties.to_string(),
                node.file_path,
                node.created_at,
                node.updated_at,
            ],
        )
        .map_err(|e| format!("Failed to insert GraphNode: {}", e))?;

        // 2. Insert all Edges
        for edge in edges {
            tx.execute(
                "UPDATE graph_edges SET valid_until = ?1 
                 WHERE from_node_id = ?2 AND to_node_id = ?3 AND edge_type = ?4 AND valid_until IS NULL",
                params![
                    edge.created_at,
                    edge.from_node_id.as_str(),
                    edge.to_node_id.as_str(),
                    edge.edge_type.as_str()
                ],
            )
            .map_err(|e| format!("Failed to expire GraphEdge: {}", e))?;

            tx.execute(
                "INSERT INTO graph_edges (id, project_id, from_node_id, to_node_id, edge_type, weight, confidence, source, valid_from, valid_until, created_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
                params![
                    edge.id,
                    edge.project_id.as_str(),
                    edge.from_node_id.as_str(),
                    edge.to_node_id.as_str(),
                    edge.edge_type.as_str(),
                    edge.weight,
                    edge.confidence,
                    edge.source,
                    edge.valid_from,
                    edge.valid_until,
                    edge.created_at,
                ],
            )
            .map_err(|e| format!("Failed to insert GraphEdge: {}", e))?;
        }

        // 3. Create Promotion Record
        tx.execute(
            "INSERT INTO candidate_promotions (id, candidate_id, promoted_node_id, promoted_by, promoted_at, promotion_reason)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                promotion.id,
                promotion.candidate_id,
                promotion.promoted_node_id.as_str(),
                promotion.promoted_by,
                promotion.promoted_at,
                promotion.promotion_reason
            ],
        )
        .map_err(|e| format!("Failed to insert CandidatePromotion: {}", e))?;

        // 4. Update Candidate Status
        tx.execute(
            "UPDATE candidates SET status = 'Approved', updated_at = ?2 WHERE id = ?1",
            params![candidate.id, promotion.promoted_at],
        )
        .map_err(|e| format!("Failed to update Candidate status: {}", e))?;

        // Commit transaction
        tx.commit().map_err(|e| e.to_string())?;

        Ok(())
    }
}

include!("candidate_tests.rs");
