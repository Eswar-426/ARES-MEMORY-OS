use std::str::FromStr;
use async_trait::async_trait;
use rusqlite::{params, OptionalExtension};

use ares_candidates::{
    Candidate, CandidateConfidence, CandidatePromotion, CandidateRepository, CandidateReview,
    CandidateSource, CandidateStatus, CandidateType,
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

        conn.execute(
            "INSERT INTO candidates (
                id, project_id, title, description, candidate_type, status,
                evidence_count, source_diversity, temporal_consistency, cluster_strength,
                created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
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
                candidate.updated_at
            ],
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    async fn get_candidate(&self, id: &str) -> Result<Option<Candidate>, String> {
        let conn = self.store.get_conn().map_err(|e| e.to_string())?;

        let candidate = conn
            .query_row(
                "SELECT id, project_id, title, description, candidate_type, status,
                 evidence_count, source_diversity, temporal_consistency, cluster_strength,
                 created_at, updated_at
                 FROM candidates WHERE id = ?1",
                params![id],
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

                    Ok(Candidate {
                        id: row.get(0)?,
                        project_id: row.get(1)?,
                        title: row.get(2)?,
                        description: row.get(3)?,
                        candidate_type: c_type,
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

        conn.execute(
            "UPDATE candidates SET
                project_id = ?2, title = ?3, description = ?4, candidate_type = ?5, status = ?6,
                evidence_count = ?7, source_diversity = ?8, temporal_consistency = ?9, cluster_strength = ?10,
                updated_at = ?11
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
                candidate.updated_at
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
                 created_at, updated_at
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

                Ok(Candidate {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    title: row.get(2)?,
                    description: row.get(3)?,
                    candidate_type: c_type,
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

    async fn get_sources(&self, candidate_id: &str) -> Result<Vec<CandidateSource>, String> {
        let conn = self.store.get_conn().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT id, candidate_id, source_type, source_id, confidence
                 FROM candidate_sources WHERE candidate_id = ?1",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![candidate_id], |row| {
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

    async fn get_reviews(&self, candidate_id: &str) -> Result<Vec<CandidateReview>, String> {
        let conn = self.store.get_conn().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT id, candidate_id, reviewer, comment, status_changed_to, review_date
                 FROM candidate_reviews WHERE candidate_id = ?1",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![candidate_id], |row| {
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

    async fn get_promotion(&self, candidate_id: &str) -> Result<Option<CandidatePromotion>, String> {
        let conn = self.store.get_conn().map_err(|e| e.to_string())?;
        let promotion = conn
            .query_row(
                "SELECT id, candidate_id, promoted_node_id, promoted_by, promoted_at, promotion_reason
                 FROM candidate_promotions WHERE candidate_id = ?1",
                params![candidate_id],
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
