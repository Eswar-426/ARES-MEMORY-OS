use crate::db::Store;
use ares_core::{
    types::decision::ReasoningStep, types::event::now_micros, AresError, CreateDecisionInput,
    Decision, DecisionFilter, DecisionId, DecisionSearchResult, DecisionStatus, MemoryId,
    ProjectId,
};
use rusqlite::params;
use tracing::debug;

pub struct SqliteDecisionRepository {
    store: Store,
}

impl SqliteDecisionRepository {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    pub fn create(&self, input: CreateDecisionInput) -> Result<Decision, AresError> {
        validate_decision_input(&input)?;
        let now = now_micros();
        let decision_id = DecisionId::new();

        // Files impacted — validate all paths are relative
        if let Some(ref files) = input.files_impacted {
            for path in files {
                validate_relative_path(path)?;
            }
        }

        let conn = self.store.get_conn()?;
        conn.execute(
            "INSERT INTO decisions
               (id, project_id, memory_id, decision_text, reason, status, confidence,
                alternatives, risks, context_snapshot, future_impact,
                files_impacted, services_impacted, supersedes, superseded_by,
                decided_by, discussed_in, review_due_at, created_at, updated_at)
             VALUES (?1,?2,?3,?4,?5,'accepted',?6,?7,?8,?9,?10,?11,?12,?13,NULL,?14,?15,?16,?17,?18)",
            params![
                decision_id.as_str(),
                input.project_id.as_str(),
                input.memory_id.as_str(),
                input.decision_text,
                input.reason,
                input.confidence.unwrap_or(1.0),
                serde_json::to_string(&input.alternatives.unwrap_or_default()).unwrap_or_default(),
                serde_json::to_string(&input.risks.unwrap_or_default()).unwrap_or_default(),
                serde_json::to_string(&input.context_snapshot.unwrap_or_default()).unwrap_or_default(),
                serde_json::to_string(&input.future_impact.unwrap_or_default()).unwrap_or_default(),
                serde_json::to_string(&input.files_impacted.unwrap_or_default()).unwrap_or_default(),
                serde_json::to_string(&input.services_impacted.unwrap_or_default()).unwrap_or_default(),
                serde_json::to_string(&input.supersedes.unwrap_or_default().iter().map(|id| id.as_str().to_string()).collect::<Vec<_>>()).unwrap_or_default(),
                input.decided_by.unwrap_or_default(),
                serde_json::to_string(&input.discussed_in.unwrap_or_default()).unwrap_or_default(),
                input.review_due_at,
                now,
                now,
            ],
        ).map_err(AresError::db)?;

        debug!(decision_id = %decision_id, "Decision created");
        self.get_by_id(&decision_id)?
            .ok_or_else(|| AresError::not_found("decision", decision_id.as_str()))
    }

    pub fn get_by_id(&self, id: &DecisionId) -> Result<Option<Decision>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn.prepare(DECISION_SELECT_SQL).map_err(AresError::db)?;
        let result = stmt.query_row(params![id.as_str()], row_to_decision_base);
        let decision = match result {
            Ok(d) => d,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
            Err(e) => return Err(AresError::db(e)),
        };

        // Load reasoning steps
        let steps = self.get_reasoning_steps(&DecisionId::from(decision.id.as_str()))?;
        Ok(Some(Decision {
            reasoning_steps: steps,
            ..decision
        }))
    }

    pub fn list(
        &self,
        project_id: &ProjectId,
        filter: DecisionFilter,
    ) -> Result<Vec<Decision>, AresError> {
        let conn = self.store.get_conn()?;
        let mut where_clauses = vec!["project_id = ?1".to_string()];
        let mut bind_values: Vec<Box<dyn rusqlite::ToSql>> =
            vec![Box::new(project_id.as_str().to_string())];
        let mut idx = 2usize;

        if let Some(status) = &filter.status {
            where_clauses.push(format!("status = ?{idx}"));
            bind_values.push(Box::new(status.as_str().to_string()));
            idx += 1;
        }
        if let Some(file) = &filter.file_path {
            // JSON search in files_impacted (SQLite JSON functions)
            where_clauses.push(format!(
                "EXISTS (SELECT 1 FROM json_each(files_impacted) WHERE value LIKE ?{idx})"
            ));
            bind_values.push(Box::new(format!("%{file}%")));
            idx += 1;
        }

        let where_sql = where_clauses.join(" AND ");
        let sql = format!(
            "SELECT id, project_id, memory_id, decision_text, reason, status, confidence,
                    alternatives, risks, context_snapshot, future_impact,
                    files_impacted, services_impacted, supersedes, superseded_by,
                    decided_by, discussed_in, review_due_at, last_reviewed_at,
                    created_at, updated_at
             FROM decisions WHERE {where_sql}
             ORDER BY created_at DESC LIMIT 200"
        );

        let mut stmt = conn.prepare(&sql).map_err(AresError::db)?;
        let refs: Vec<&dyn rusqlite::ToSql> = bind_values.iter().map(|b| b.as_ref()).collect();
        let rows = stmt
            .query_map(refs.as_slice(), row_to_decision_base)
            .map_err(AresError::db)?;

        let mut decisions = vec![];
        for row in rows {
            let d = row.map_err(AresError::db)?;
            let steps = self.get_reasoning_steps(&d.id)?;
            decisions.push(Decision {
                reasoning_steps: steps,
                ..d
            });
        }
        let _ = idx; // suppress unused warning
        Ok(decisions)
    }

    pub fn list_paginated(
        &self,
        project_id: &ProjectId,
        filter: DecisionFilter,
        pagination: &ares_core::types::pagination::Pagination,
    ) -> Result<ares_core::types::pagination::Page<Decision>, AresError> {
        let conn = self.store.get_conn()?;
        let mut where_clauses = vec!["project_id = ?1".to_string()];
        let mut bind_values: Vec<Box<dyn rusqlite::ToSql>> =
            vec![Box::new(project_id.as_str().to_string())];
        let mut idx = 2usize;

        if let Some(status) = &filter.status {
            where_clauses.push(format!("status = ?{idx}"));
            bind_values.push(Box::new(status.as_str().to_string()));
            idx += 1;
        }
        if let Some(file) = &filter.file_path {
            where_clauses.push(format!(
                "EXISTS (SELECT 1 FROM json_each(files_impacted) WHERE value LIKE ?{idx})"
            ));
            bind_values.push(Box::new(format!("%{file}%")));
            idx += 1;
        }
        if let Some(since) = &filter.since {
            where_clauses.push(format!("created_at >= ?{idx}"));
            bind_values.push(Box::new(*since));
            idx += 1;
        }
        if let Some(until) = &filter.until {
            where_clauses.push(format!("created_at <= ?{idx}"));
            bind_values.push(Box::new(*until));
        }

        let _ = idx; // suppress unused warning

        let where_sql = where_clauses.join(" AND ");

        // Count total
        let count_sql = format!("SELECT COUNT(*) FROM decisions WHERE {where_sql}");
        let mut count_stmt = conn.prepare(&count_sql).map_err(AresError::db)?;
        let refs: Vec<&dyn rusqlite::ToSql> = bind_values.iter().map(|b| b.as_ref()).collect();
        let total: u64 = count_stmt
            .query_row(refs.as_slice(), |row| row.get(0))
            .map_err(AresError::db)?;

        // Fetch paginated
        let offset = pagination.offset();
        let limit = pagination.limit();
        let sql = format!(
            "SELECT id, project_id, memory_id, decision_text, reason, status, confidence,
                    alternatives, risks, context_snapshot, future_impact,
                    files_impacted, services_impacted, supersedes, superseded_by,
                    decided_by, discussed_in, review_due_at, last_reviewed_at,
                    created_at, updated_at
             FROM decisions WHERE {where_sql}
             ORDER BY created_at DESC
             LIMIT {limit} OFFSET {offset}"
        );

        let mut stmt = conn.prepare(&sql).map_err(AresError::db)?;
        let rows = stmt
            .query_map(refs.as_slice(), row_to_decision_base)
            .map_err(AresError::db)?;

        let mut decisions = vec![];
        for row in rows {
            let d = row.map_err(AresError::db)?;
            let steps = self.get_reasoning_steps(&d.id)?;
            decisions.push(Decision {
                reasoning_steps: steps,
                ..d
            });
        }

        Ok(ares_core::types::pagination::Page::new(decisions, total, pagination.page, pagination.page_size))
    }

    pub fn supersede(&self, old_id: &DecisionId, new_id: &DecisionId) -> Result<(), AresError> {
        let now = now_micros();
        let conn = self.store.get_conn()?;

        // Verify old decision exists and is not already superseded
        let status: Option<String> = {
            let mut stmt = conn
                .prepare("SELECT status FROM decisions WHERE id = ?1")
                .map_err(AresError::db)?;
            match stmt.query_row(params![old_id.as_str()], |r| r.get(0)) {
                Ok(s) => Some(s),
                Err(rusqlite::Error::QueryReturnedNoRows) => None,
                Err(e) => return Err(AresError::db(e)),
            }
        };

        match status.as_deref() {
            None => return Err(AresError::not_found("decision", old_id.as_str())),
            Some("superseded") => {
                return Err(AresError::AlreadySuperseded {
                    id: old_id.to_string(),
                })
            }
            _ => {}
        }

        // Update old decision
        conn.execute(
            "UPDATE decisions SET status = 'superseded', superseded_by = ?1, updated_at = ?2 WHERE id = ?3",
            params![new_id.as_str(), now, old_id.as_str()],
        ).map_err(AresError::db)?;

        debug!(old = %old_id, new = %new_id, "Decision superseded");
        Ok(())
    }

    pub fn get_stale(
        &self,
        project_id: &ProjectId,
        threshold_days: u32,
    ) -> Result<Vec<Decision>, AresError> {
        let now = now_micros();
        let threshold_micros = (threshold_days as i64) * 24 * 60 * 60 * 1_000_000i64;
        let cutoff = now - threshold_micros;

        self.list(
            project_id,
            DecisionFilter {
                status: Some(DecisionStatus::Accepted),
                ..Default::default()
            },
        )
        .map(|decisions| {
            decisions
                .into_iter()
                .filter(|d| {
                    let last_activity = d.last_reviewed_at.unwrap_or(d.created_at);
                    last_activity < cutoff
                })
                .collect()
        })
    }

    pub fn search(
        &self,
        project_id: &ProjectId,
        query: &str,
    ) -> Result<Vec<DecisionSearchResult>, AresError> {
        // Delegate to memories FTS, then join with decisions
        let conn = self.store.get_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT d.id, fts.rank,
                    snippet(memories_fts, 1, '[', ']', '...', 10) as snip
             FROM memories_fts fts
             JOIN memories m ON m.id = fts.memory_id
             JOIN decisions d ON d.memory_id = m.id
             WHERE memories_fts MATCH ?1
               AND m.project_id = ?2
               AND m.deleted_at IS NULL
               AND m.memory_type = 'decision'
               AND d.status != 'superseded'
             ORDER BY fts.rank
             LIMIT 20",
            )
            .map_err(AresError::db)?;

        let rows = stmt
            .query_map(params![query, project_id.as_str()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, f64>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(AresError::db)?;

        let mut results = vec![];
        for row in rows {
            let (id_str, rank, snippet) = row.map_err(AresError::db)?;
            if let Some(decision) = self.get_by_id(&DecisionId::from(id_str))? {
                results.push(DecisionSearchResult {
                    decision,
                    score: -rank,
                    snippet,
                });
            }
        }
        Ok(results)
    }

    pub fn add_reasoning_step(&self, step: ReasoningStep) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        conn.execute(
            "INSERT INTO decision_reasoning (id, decision_id, step_order, observation, inference, confidence, created_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![
                step.id,
                step.decision_id.as_str(),
                step.step_order,
                step.observation,
                step.inference,
                step.confidence,
                step.created_at,
            ],
        ).map_err(AresError::db)?;
        Ok(())
    }

    fn get_reasoning_steps(
        &self,
        decision_id: &DecisionId,
    ) -> Result<Vec<ReasoningStep>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, decision_id, step_order, observation, inference, confidence, created_at
             FROM decision_reasoning WHERE decision_id = ?1 ORDER BY step_order ASC",
            )
            .map_err(AresError::db)?;

        let rows = stmt
            .query_map(params![decision_id.as_str()], |row| {
                Ok(ReasoningStep {
                    id: row.get(0)?,
                    decision_id: DecisionId::from(row.get::<_, String>(1)?),
                    step_order: row.get::<_, i64>(2)? as u32,
                    observation: row.get(3)?,
                    inference: row.get(4)?,
                    confidence: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })
            .map_err(AresError::db)?;

        rows.collect::<Result<Vec<_>, _>>().map_err(AresError::db)
    }
}

// ─────────────────────────────────────────────────────────────────
// Validation helpers
// ─────────────────────────────────────────────────────────────────

fn validate_decision_input(input: &CreateDecisionInput) -> Result<(), AresError> {
    if input.title.trim().is_empty() {
        return Err(AresError::validation("title cannot be empty"));
    }
    if input.title.len() > 500 {
        return Err(AresError::validation("title exceeds 500 character limit"));
    }
    if input.decision_text.trim().is_empty() {
        return Err(AresError::validation("decision_text cannot be empty"));
    }
    if input.decision_text.len() > 50_000 {
        return Err(AresError::validation("decision_text exceeds 50KB limit"));
    }
    if input.reason.trim().is_empty() {
        return Err(AresError::validation("reason cannot be empty"));
    }
    if let Some(c) = input.confidence {
        if !(0.0..=1.0).contains(&c) {
            return Err(AresError::validation(
                "confidence must be between 0.0 and 1.0",
            ));
        }
    }
    Ok(())
}

fn validate_relative_path(path: &str) -> Result<(), AresError> {
    use std::path::Path;
    let p = Path::new(path);
    if p.is_absolute() {
        return Err(AresError::invalid_path(format!(
            "path must be relative: {path}"
        )));
    }
    for component in p.components() {
        if matches!(component, std::path::Component::ParentDir) {
            return Err(AresError::invalid_path(format!(
                "path contains '..': {path}"
            )));
        }
    }
    Ok(())
}

// ─────────────────────────────────────────────────────────────────
// Row mapper
// ─────────────────────────────────────────────────────────────────

const DECISION_SELECT_SQL: &str =
    "SELECT id, project_id, memory_id, decision_text, reason, status, confidence,
            alternatives, risks, context_snapshot, future_impact,
            files_impacted, services_impacted, supersedes, superseded_by,
            decided_by, discussed_in, review_due_at, last_reviewed_at,
            created_at, updated_at
     FROM decisions WHERE id = ?1";

fn row_to_decision_base(row: &rusqlite::Row<'_>) -> Result<Decision, rusqlite::Error> {
    let status_str: String = row.get(5)?;
    let superseded_by_str: Option<String> = row.get(14)?;

    let parse_json_vec =
        |s: String| -> Vec<String> { serde_json::from_str(&s).unwrap_or_default() };

    Ok(Decision {
        id: DecisionId::from(row.get::<_, String>(0)?),
        project_id: ProjectId::from(row.get::<_, String>(1)?),
        memory_id: MemoryId::from(row.get::<_, String>(2)?),
        title: String::new(), // filled by caller from memory
        decision_text: row.get(3)?,
        reason: row.get(4)?,
        status: status_str.parse().unwrap_or_default(),
        confidence: row.get(6)?,
        reasoning_steps: vec![], // loaded separately
        alternatives: serde_json::from_str(&row.get::<_, String>(7)?).unwrap_or_default(),
        risks: serde_json::from_str(&row.get::<_, String>(8)?).unwrap_or_default(),
        context_snapshot: serde_json::from_str(&row.get::<_, String>(9)?).unwrap_or_default(),
        future_impact: serde_json::from_str(&row.get::<_, String>(10)?).unwrap_or_default(),
        files_impacted: parse_json_vec(row.get(11)?),
        services_impacted: parse_json_vec(row.get(12)?),
        supersedes: parse_json_vec(row.get(13)?)
            .into_iter()
            .map(DecisionId::from)
            .collect(),
        superseded_by: superseded_by_str.map(DecisionId::from),
        decided_by: row.get(15)?,
        discussed_in: parse_json_vec(row.get(16)?),
        review_due_at: row.get(17)?,
        last_reviewed_at: row.get(18)?,
        created_at: row.get(19)?,
        updated_at: row.get(20)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_helpers::test_store;
    use crate::repositories::memory::SqliteMemoryRepository;
    use ares_core::{new_id, CreateDecisionInput, CreateMemoryInput, MemoryType};

    fn setup(store: &Store) -> (ProjectId, MemoryId) {
        use crate::repositories::project::SqliteProjectRepository;
        use ares_core::ProjectMaturity;

        let now = now_micros();
        let project_id = ProjectId::new();
        let project_repo = SqliteProjectRepository::new(store.clone());
        project_repo
            .create(&ares_core::Project {
                id: project_id.clone(),
                name: "test".into(),
                description: "".into(),
                root_path: format!("/tmp/{}", new_id()),
                primary_language: "ts".into(),
                domain: "".into(),
                maturity: ProjectMaturity::Greenfield,
                created_at: now,
                updated_at: now,
                deleted_at: None,
            })
            .unwrap();

        let mem_repo = SqliteMemoryRepository::new(store.clone());
        let mem = mem_repo
            .create(CreateMemoryInput {
                project_id: project_id.clone(),
                memory_type: MemoryType::Decision,
                title: "Use JWT for auth".into(),
                content: serde_json::json!({}),
                confidence: None,
                importance: None,
                source: None,
                ai_assisted: None,
            })
            .unwrap();

        (project_id, mem.id)
    }

    fn make_decision_input(project_id: ProjectId, memory_id: MemoryId) -> CreateDecisionInput {
        CreateDecisionInput {
            project_id,
            title: "Use JWT for auth".into(),
            memory_id,
            decision_text: "We will use JWT tokens for stateless auth".into(),
            reason: "Enables horizontal scaling without session state".into(),
            confidence: Some(0.9),
            alternatives: None,
            risks: None,
            context_snapshot: None,
            future_impact: None,
            files_impacted: Some(vec!["src/auth/middleware.ts".into()]),
            services_impacted: None,
            supersedes: None,
            decided_by: Some("alice".into()),
            discussed_in: None,
            review_due_at: None,
        }
    }

    #[test]
    fn create_and_get_decision() {
        let (store, _dir) = test_store();
        let (project_id, memory_id) = setup(&store);
        let repo = SqliteDecisionRepository::new(store);
        let decision = repo
            .create(make_decision_input(project_id, memory_id))
            .unwrap();
        assert_eq!(decision.status, DecisionStatus::Accepted);
        assert_eq!(decision.decided_by, "alice");
        assert_eq!(decision.files_impacted, vec!["src/auth/middleware.ts"]);
    }

    #[test]
    fn supersede_marks_old_as_superseded() {
        let (store, _dir) = test_store();
        let (project_id, memory_id) = setup(&store);
        let mem_repo = SqliteMemoryRepository::new(store.clone());

        // Create second memory for new decision
        let mem2 = mem_repo
            .create(CreateMemoryInput {
                project_id: project_id.clone(),
                memory_type: MemoryType::Decision,
                title: "Use OAuth".into(),
                content: serde_json::json!({}),
                confidence: None,
                importance: None,
                source: None,
                ai_assisted: None,
            })
            .unwrap();

        let repo = SqliteDecisionRepository::new(store);
        let old = repo
            .create(make_decision_input(project_id.clone(), memory_id))
            .unwrap();
        let new_input = CreateDecisionInput {
            title: "Use OAuth instead".into(),
            memory_id: mem2.id,
            decision_text: "Switch to OAuth 2.0".into(),
            reason: "Better third-party support".into(),
            supersedes: Some(vec![old.id.clone()]),
            ..make_decision_input(project_id, MemoryId::new())
        };
        let new_dec = repo.create(new_input).unwrap();
        repo.supersede(&old.id, &new_dec.id).unwrap();

        let updated_old = repo.get_by_id(&old.id).unwrap().unwrap();
        assert_eq!(updated_old.status, DecisionStatus::Superseded);
        assert_eq!(updated_old.superseded_by, Some(new_dec.id));
    }

    #[test]
    fn path_traversal_rejected() {
        let (store, _dir) = test_store();
        let (project_id, memory_id) = setup(&store);
        let repo = SqliteDecisionRepository::new(store);
        let input = CreateDecisionInput {
            files_impacted: Some(vec!["../../etc/passwd".into()]),
            ..make_decision_input(project_id, memory_id)
        };
        let err = repo.create(input).unwrap_err();
        assert!(matches!(err, AresError::InvalidPath(_)));
    }
}
