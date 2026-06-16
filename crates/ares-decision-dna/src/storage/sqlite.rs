use crate::models::{
    decision::{DecisionMemory, DecisionState, DecisionOutcome},
    requirement::Requirement,
    chain::ReasoningChain,
    impact::ImpactMap,
    provenance::ProvenanceRecord,
};
use anyhow::Result;
use rusqlite::{params, Connection};
use std::sync::Arc;

pub struct DecisionStorage {
    conn: Arc<std::sync::Mutex<Connection>>,
}

impl DecisionStorage {
    pub fn new(conn: Arc<std::sync::Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn save_decision(&self, decision: &DecisionMemory) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        let state_str = match decision.state {
            DecisionState::Proposed => "proposed",
            DecisionState::Accepted => "accepted",
            DecisionState::Rejected => "rejected",
            DecisionState::Superseded => "superseded",
            DecisionState::Deprecated => "deprecated",
        };

        conn.execute(
            "INSERT INTO decisions (
                id, title, context, state, version, confidence, ai_assisted, human_reviewed,
                review_due_at, superseded_by, created_at, updated_at, tags,
                -- Legacy columns
                project_id, memory_id, decision_text, reason
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13,
                '', '', '', ''
            )
            ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                context = excluded.context,
                state = excluded.state,
                version = excluded.version,
                confidence = excluded.confidence,
                ai_assisted = excluded.ai_assisted,
                human_reviewed = excluded.human_reviewed,
                review_due_at = excluded.review_due_at,
                superseded_by = excluded.superseded_by,
                updated_at = excluded.updated_at,
                tags = excluded.tags",
            params![
                decision.id.to_string(),
                decision.title,
                decision.context,
                state_str,
                decision.version,
                decision.confidence,
                decision.ai_assisted,
                decision.human_reviewed,
                decision.review_due_at.map(|d| d.timestamp_millis()),
                decision.superseded_by.map(|id| id.to_string()),
                decision.created_at.timestamp_millis(),
                decision.updated_at.timestamp_millis(),
                serde_json::to_string(&decision.tags)?,
            ],
        )?;

        // Save provenance
        let source_type_str = match decision.provenance.source_type {
            crate::models::provenance::SourceType::Human => "Human",
            crate::models::provenance::SourceType::AI => "AI",
            crate::models::provenance::SourceType::Imported => "Imported",
            crate::models::provenance::SourceType::Generated => "Generated",
        };

        conn.execute(
            "INSERT INTO decision_provenance (
                decision_id, source_type, author_id, created_by_agent, reviewed_by,
                confidence, source_system, original_commit, pull_request_url, evidence_links
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            ON CONFLICT(decision_id) DO UPDATE SET
                source_type = excluded.source_type,
                author_id = excluded.author_id,
                created_by_agent = excluded.created_by_agent,
                reviewed_by = excluded.reviewed_by,
                confidence = excluded.confidence,
                source_system = excluded.source_system,
                original_commit = excluded.original_commit,
                pull_request_url = excluded.pull_request_url,
                evidence_links = excluded.evidence_links",
            params![
                decision.id.to_string(),
                source_type_str,
                decision.provenance.author_id.map(|id| id.to_string()),
                decision.provenance.created_by_agent,
                decision.provenance.reviewed_by.map(|id| id.to_string()),
                decision.provenance.confidence,
                decision.provenance.source_system,
                decision.provenance.original_commit,
                decision.provenance.pull_request_url,
                serde_json::to_string(&decision.provenance.evidence_links)?,
            ],
        )?;

        Ok(())
    }

    pub fn get_decision(&self, id: &crate::models::DecisionId) -> Result<Option<DecisionMemory>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("
            SELECT d.id, d.title, d.context, d.state, d.version, d.confidence, 
                   d.ai_assisted, d.human_reviewed, d.review_due_at, d.superseded_by, 
                   d.created_at, d.updated_at, d.tags,
                   p.source_type, p.author_id, p.created_by_agent, p.reviewed_by,
                   p.confidence as p_confidence, p.source_system, p.original_commit,
                   p.pull_request_url, p.evidence_links
            FROM decisions d
            LEFT JOIN decision_provenance p ON d.id = p.decision_id
            WHERE d.id = ?1
        ")?;

        let mut rows = stmt.query(params![id.to_string()])?;

        if let Some(row) = rows.next()? {
            let state_str: String = row.get(3)?;
            let state = match state_str.as_str() {
                "proposed" => DecisionState::Proposed,
                "accepted" => DecisionState::Accepted,
                "rejected" => DecisionState::Rejected,
                "superseded" => DecisionState::Superseded,
                "deprecated" => DecisionState::Deprecated,
                _ => DecisionState::Proposed,
            };

            let tags_str: String = row.get(12)?;
            let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();

            let source_type_str: Option<String> = row.get(13)?;
            let source_type = match source_type_str.as_deref() {
                Some("Human") => crate::models::SourceType::Human,
                Some("AI") => crate::models::SourceType::AI,
                Some("Imported") => crate::models::SourceType::Imported,
                Some("Generated") => crate::models::SourceType::Generated,
                _ => crate::models::SourceType::Human,
            };

            let evidence_links_str: Option<String> = row.get(21)?;
            let evidence_links: Vec<String> = evidence_links_str
                .map(|s| serde_json::from_str(&s).unwrap_or_default())
                .unwrap_or_default();

            let provenance = ProvenanceRecord {
                source_type,
                author_id: row.get::<_, Option<String>>(14)?.map(|s| uuid::Uuid::parse_str(&s).unwrap()),
                created_by_agent: row.get(15).unwrap_or(None),
                reviewed_by: row.get::<_, Option<String>>(16)?.map(|s| uuid::Uuid::parse_str(&s).unwrap()),
                confidence: row.get(17).unwrap_or(1.0),
                source_system: row.get(18).unwrap_or_else(|_| "ARES".to_string()),
                original_commit: row.get(19).unwrap_or(None),
                pull_request_url: row.get(20).unwrap_or(None),
                evidence_links,
            };

            let created_at_ts: i64 = row.get(10)?;
            let updated_at_ts: i64 = row.get(11)?;

            let decision = DecisionMemory {
                id: *id,
                title: row.get(1)?,
                context: row.get(2)?,
                state,
                version: row.get(4)?,
                created_at: chrono::DateTime::from_timestamp_millis(created_at_ts).unwrap_or_default(),
                updated_at: chrono::DateTime::from_timestamp_millis(updated_at_ts).unwrap_or_default(),
                confidence: row.get(5)?,
                ai_assisted: row.get(6)?,
                human_reviewed: row.get(7)?,
                review_due_at: row.get::<_, Option<i64>>(8)?.map(|ts| chrono::DateTime::from_timestamp_millis(ts).unwrap()),
                approved_by: vec![],
                tags,
                supersedes: vec![],
                superseded_by: row.get::<_, Option<String>>(9)?.map(|s| uuid::Uuid::parse_str(&s).unwrap()),
                provenance,
                reasoning: ReasoningChain {
                    id: uuid::Uuid::new_v4(),
                    steps: vec!["Placeholder rationale for test".to_string()],
                    alternatives: vec![],
                    assumptions: vec![],
                    risks: vec![],
                },
                impact: ImpactMap {
                    files_affected: vec![],
                    systems_affected: vec![],
                    estimated_effort: crate::models::impact::EffortEstimation::Medium,
                },
            };
            Ok(Some(decision))
        } else {
            Ok(None)
        }
    }

    pub fn save_requirement(&self, req: &Requirement) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        let priority_str = match req.priority {
            crate::models::Priority::Low => "Low",
            crate::models::Priority::Medium => "Medium",
            crate::models::Priority::High => "High",
            crate::models::Priority::Critical => "Critical",
        };

        let status_str = match req.status {
            crate::models::RequirementStatus::Draft => "Draft",
            crate::models::RequirementStatus::Active => "Active",
            crate::models::RequirementStatus::Fulfilled => "Fulfilled",
            crate::models::RequirementStatus::Obsolete => "Obsolete",
        };

        let source_str = match req.source {
            crate::models::RequirementSource::User => "User",
            crate::models::RequirementSource::System => "System",
            crate::models::RequirementSource::Compliance => "Compliance",
            crate::models::RequirementSource::Security => "Security",
        };

        conn.execute(
            "INSERT INTO requirements (id, title, description, priority, status, source, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                description = excluded.description,
                priority = excluded.priority,
                status = excluded.status,
                source = excluded.source",
            params![
                req.id.to_string(),
                req.title,
                req.description,
                priority_str,
                status_str,
                source_str,
                req.created_at.timestamp_millis(),
            ],
        )?;
        Ok(())
    }

    pub fn save_outcome(&self, outcome: &DecisionOutcome) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO decision_dna_outcomes (decision_id, success_score, lessons_learned, measured_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(decision_id) DO UPDATE SET
                success_score = excluded.success_score,
                lessons_learned = excluded.lessons_learned,
                measured_at = excluded.measured_at",
            params![
                outcome.decision_id.to_string(),
                outcome.success_score,
                serde_json::to_string(&outcome.lessons_learned)?,
                outcome.measured_at.timestamp_millis(),
            ],
        )?;
        Ok(())
    }
}
