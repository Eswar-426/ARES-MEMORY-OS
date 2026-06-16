use crate::models::{Decision, DecisionStatus, DecisionConfidence};
use ares_core::{AresError, DecisionId};
use ares_store::db::Store;
use ares_traceability::{EdgeProvider, TraceabilityEdge, TraceTargetType};
use rusqlite::{params, OptionalExtension};

pub struct DecisionStore {
    store: Store,
}

impl DecisionStore {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    pub fn insert_decision(&self, decision: &Decision) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        
        let rejected_options_json = serde_json::to_string(&decision.rejected_options)
            .map_err(|e| AresError::db(e.to_string()))?;
        let assumptions_json = serde_json::to_string(&decision.assumptions)
            .map_err(|e| AresError::db(e.to_string()))?;
        let consequences_json = serde_json::to_string(&decision.consequences)
            .map_err(|e| AresError::db(e.to_string()))?;
            
        let status_json = serde_json::to_string(&decision.approval_status)
            .map_err(|e| AresError::db(e.to_string()))?;
        let confidence_json = serde_json::to_string(&decision.confidence)
            .map_err(|e| AresError::db(e.to_string()))?;

        conn.execute(
            "INSERT INTO decision_records (
                id, title, context, problem, chosen_option, 
                rejected_options, assumptions, consequences, confidence, 
                owner, approval_status, approved_by, approved_at, 
                created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            params![
                decision.id.as_str(),
                decision.title,
                decision.context,
                decision.problem,
                decision.chosen_option,
                rejected_options_json,
                assumptions_json,
                consequences_json,
                confidence_json,
                decision.owner,
                status_json,
                decision.approved_by,
                decision.approved_at,
                decision.created_at,
                decision.updated_at
            ],
        )
        .map_err(AresError::db)?;

        Ok(())
    }

    pub fn get_decision(&self, id: &DecisionId) -> Result<Option<Decision>, AresError> {
        let conn = self.store.get_conn()?;
        
        let mut stmt = conn.prepare(
            "SELECT id, title, context, problem, chosen_option, 
                    rejected_options, assumptions, consequences, confidence, 
                    owner, approval_status, approved_by, approved_at, 
                    created_at, updated_at
             FROM decision_records
             WHERE id = ?1"
        ).map_err(AresError::db)?;

        let decision = stmt.query_row(params![id.as_str()], |row| {
            let rejected_options_str: String = row.get(5)?;
            let assumptions_str: String = row.get(6)?;
            let consequences_str: String = row.get(7)?;
            let confidence_str: String = row.get(8)?;
            let status_str: String = row.get(10)?;

            Ok(Decision {
                id: DecisionId::from(row.get::<_, String>(0)?),
                title: row.get(1)?,
                context: row.get(2)?,
                problem: row.get(3)?,
                chosen_option: row.get(4)?,
                rejected_options: serde_json::from_str(&rejected_options_str).unwrap_or_default(),
                assumptions: serde_json::from_str(&assumptions_str).unwrap_or_default(),
                consequences: serde_json::from_str(&consequences_str).unwrap_or_default(),
                confidence: serde_json::from_str(&confidence_str).unwrap_or(DecisionConfidence::Medium),
                owner: row.get(9)?,
                approval_status: serde_json::from_str(&status_str).unwrap_or(DecisionStatus::Proposed),
                approved_by: row.get(11)?,
                approved_at: row.get(12)?,
                created_at: row.get(13)?,
                updated_at: row.get(14)?,
            })
        })
        .optional()
        .map_err(AresError::db)?;

        Ok(decision)
    }

    pub fn insert_evidence(&self, evidence: &crate::models::DecisionEvidence) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        
        let source_json = serde_json::to_string(&evidence.source)
            .map_err(|e| AresError::db(e.to_string()))?;

        conn.execute(
            "INSERT INTO decision_evidence (
                id, decision_id, source, reference_url, description, confidence_score
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                evidence.id.as_str(),
                evidence.decision_id.as_str(),
                source_json,
                evidence.reference_url,
                evidence.description,
                evidence.confidence_score,
            ],
        )
        .map_err(AresError::db)?;

        Ok(())
    }

    pub fn get_evidence(&self, decision_id: &DecisionId) -> Result<Vec<crate::models::DecisionEvidence>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, source, reference_url, description, confidence_score 
                 FROM decision_evidence 
                 WHERE decision_id = ?1"
            )
            .map_err(AresError::db)?;

        let evidence = stmt.query_map(params![decision_id.as_str()], |row| {
            let source_str: String = row.get(1)?;
            Ok(crate::models::DecisionEvidence {
                id: ares_core::id::EvidenceId::from(row.get::<_, String>(0)?),
                decision_id: decision_id.clone(),
                source: serde_json::from_str(&source_str).unwrap_or(crate::models::EvidenceSource::Other("Unknown".to_string())),
                reference_url: row.get(2)?,
                description: row.get(3)?,
                confidence_score: row.get(4)?,
            })
        }).map_err(AresError::db)?
        .collect::<Result<Vec<_>, _>>().map_err(AresError::db)?;

        Ok(evidence)
    }

    pub fn insert_outcome(&self, outcome: &crate::models::DecisionOutcome) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        
        let type_json = serde_json::to_string(&outcome.outcome_type)
            .map_err(|e| AresError::db(e.to_string()))?;

        conn.execute(
            "INSERT INTO decision_outcomes (
                id, decision_id, observed_at, description, outcome_type, success_score
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                outcome.id,
                outcome.decision_id.as_str(),
                outcome.observed_at,
                outcome.description,
                type_json,
                outcome.success_score,
            ],
        )
        .map_err(AresError::db)?;

        Ok(())
    }

    pub fn get_outcomes(&self, decision_id: &DecisionId) -> Result<Vec<crate::models::DecisionOutcome>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, observed_at, description, outcome_type, success_score 
                 FROM decision_outcomes 
                 WHERE decision_id = ?1"
            )
            .map_err(AresError::db)?;

        let outcomes = stmt.query_map(params![decision_id.as_str()], |row| {
            let type_str: String = row.get(3)?;
            Ok(crate::models::DecisionOutcome {
                id: row.get(0)?,
                decision_id: decision_id.clone(),
                observed_at: row.get(1)?,
                description: row.get(2)?,
                outcome_type: serde_json::from_str(&type_str).unwrap_or(crate::models::OutcomeType::Other("Unknown".to_string())),
                success_score: row.get(4)?,
            })
        }).map_err(AresError::db)?
        .collect::<Result<Vec<_>, _>>().map_err(AresError::db)?;

        Ok(outcomes)
    }

    pub fn list(&self) -> Result<Vec<Decision>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, title, context, problem, chosen_option, 
                    rejected_options, assumptions, consequences, confidence, 
                    owner, approval_status, approved_by, approved_at, 
                    created_at, updated_at
             FROM decision_records"
        ).map_err(AresError::db)?;

        let decisions = stmt.query_map([], |row| {
            let rejected_options_str: String = row.get(5)?;
            let assumptions_str: String = row.get(6)?;
            let consequences_str: String = row.get(7)?;
            let confidence_str: String = row.get(8)?;
            let status_str: String = row.get(10)?;

            Ok(Decision {
                id: DecisionId::from(row.get::<_, String>(0)?),
                title: row.get(1)?,
                context: row.get(2)?,
                problem: row.get(3)?,
                chosen_option: row.get(4)?,
                rejected_options: serde_json::from_str(&rejected_options_str).unwrap_or_default(),
                assumptions: serde_json::from_str(&assumptions_str).unwrap_or_default(),
                consequences: serde_json::from_str(&consequences_str).unwrap_or_default(),
                confidence: serde_json::from_str(&confidence_str).unwrap_or(DecisionConfidence::Medium),
                owner: row.get(9)?,
                approval_status: serde_json::from_str(&status_str).unwrap_or(DecisionStatus::Proposed),
                approved_by: row.get(11)?,
                approved_at: row.get(12)?,
                created_at: row.get(13)?,
                updated_at: row.get(14)?,
            })
        }).map_err(AresError::db)?
        .collect::<Result<Vec<_>, _>>().map_err(AresError::db)?;

        Ok(decisions)
    }
}
pub struct DecisionEdgeProvider {
    store: Store,
}

impl DecisionEdgeProvider {
    pub fn new(store: Store) -> Self {
        Self { store }
    }
}

impl EdgeProvider for DecisionEdgeProvider {
    fn edges(&self) -> Result<Vec<TraceabilityEdge>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn
            .prepare("SELECT source_decision_id, target_id, target_type, relationship FROM decision_links")
            .map_err(AresError::db)?;

        let edges = stmt.query_map([], |row| {
            let source_id: String = row.get(0)?;
            let target_id: String = row.get(1)?;
            let target_type_str: String = row.get(2)?;
            let relationship: String = row.get(3)?;
            
            let target_type = match target_type_str.as_str() {
                "requirement" => TraceTargetType::Requirement,
                "decision" => TraceTargetType::Decision,
                "architecture" => TraceTargetType::Architecture,
                "code" => TraceTargetType::Code,
                other => TraceTargetType::Unknown(other.to_string()),
            };

            Ok(TraceabilityEdge {
                source_id,
                target_id,
                target_type,
                relationship,
            })
        }).map_err(AresError::db)?
        .collect::<Result<Vec<_>, _>>().map_err(AresError::db)?;

        Ok(edges)
    }
}
