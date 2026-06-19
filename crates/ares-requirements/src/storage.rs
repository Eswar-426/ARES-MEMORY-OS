use crate::models::{
    CreateRequirementInput, LinkTarget, LinkTargetType, Requirement, RequirementFilter,
    RequirementLink, RequirementPriority, RequirementStatus, RequirementType,
    UpdateRequirementInput,
};
use ares_core::{AresError, ProjectId, RequirementId, RequirementLinkId};
use ares_store::db::Store;
use chrono::Utc;
use rusqlite::params;
use tracing::debug;

pub struct RequirementStore {
    store: Store,
}

pub struct LinkCounts {
    pub decision_links: usize,
    pub architecture_links: usize,
    pub code_links: usize,
    pub requirement_links: usize,
    pub total: usize,
}

impl RequirementStore {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    // --- CRUD ---

    pub fn create(&self, input: CreateRequirementInput) -> Result<Requirement, AresError> {
        let req_id = RequirementId::new();
        let now = Utc::now().timestamp_micros();
        let conn = self.store.get_conn()?;

        let type_str = serde_json::to_string(&input.requirement_type)
            .unwrap_or_else(|_| "\"functional\"".to_string())
            .replace("\"", "");
        let priority_str = serde_json::to_string(&input.priority)
            .unwrap_or_else(|_| "\"medium\"".to_string())
            .replace("\"", "");
        let source_str = serde_json::to_string(&input.source)
            .unwrap_or_else(|_| "\"product\"".to_string())
            .replace("\"", "");
        let tags_json = serde_json::to_string(&input.tags).unwrap_or_else(|_| "[]".to_string());

        conn.execute(
            "INSERT INTO requirements (
                id, title, description, priority, status, source, created_at,
                requirement_type, owner, updated_at, tags, project_id
            ) VALUES (
                ?1, ?2, ?3, ?4, 'draft', ?5, ?6,
                ?7, ?8, ?9, ?10, ?11
            )",
            params![
                req_id.as_str(),
                input.title,
                input.description,
                priority_str,
                source_str,
                now,
                type_str,
                input.owner,
                now,
                tags_json,
                input.project_id.as_str(),
            ],
        )
        .map_err(AresError::db)?;

        debug!(id = %req_id, "Requirement created");
        self.get(&req_id)?.ok_or_else(|| AresError::not_found("requirement", req_id.as_str()))
    }

    pub fn update(
        &self,
        id: &RequirementId,
        input: UpdateRequirementInput,
    ) -> Result<Requirement, AresError> {
        let current = self
            .get(id)?
            .ok_or_else(|| AresError::not_found("requirement", id.as_str()))?;

        let mut sets = Vec::new();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        let mut idx = 1;

        if let Some(title) = input.title {
            sets.push(format!("title = ?{idx}"));
            params_vec.push(Box::new(title));
            idx += 1;
        }
        if let Some(description) = input.description {
            sets.push(format!("description = ?{idx}"));
            params_vec.push(Box::new(description));
            idx += 1;
        }
        if let Some(req_type) = input.requirement_type {
            sets.push(format!("requirement_type = ?{idx}"));
            let val = serde_json::to_string(&req_type).unwrap().replace("\"", "");
            params_vec.push(Box::new(val));
            idx += 1;
        }
        if let Some(source) = input.source {
            sets.push(format!("source = ?{idx}"));
            let val = serde_json::to_string(&source).unwrap().replace("\"", "");
            params_vec.push(Box::new(val));
            idx += 1;
        }
        if let Some(status) = input.status {
            sets.push(format!("status = ?{idx}"));
            let val = serde_json::to_string(&status).unwrap().replace("\"", "");
            params_vec.push(Box::new(val));
            idx += 1;
        }
        if let Some(priority) = input.priority {
            sets.push(format!("priority = ?{idx}"));
            let val = serde_json::to_string(&priority).unwrap().replace("\"", "");
            params_vec.push(Box::new(val));
            idx += 1;
        }
        if let Some(owner_opt) = input.owner {
            sets.push(format!("owner = ?{idx}"));
            params_vec.push(Box::new(owner_opt));
            idx += 1;
        }
        if let Some(tags) = input.tags {
            sets.push(format!("tags = ?{idx}"));
            let val = serde_json::to_string(&tags).unwrap();
            params_vec.push(Box::new(val));
            idx += 1;
        }

        if sets.is_empty() {
            return Ok(current);
        }

        let now = Utc::now().timestamp_micros();
        sets.push(format!("updated_at = ?{idx}"));
        params_vec.push(Box::new(now));
        idx += 1;

        let sets_str = sets.join(", ");
        let query = format!("UPDATE requirements SET {sets_str} WHERE id = ?{idx}");
        params_vec.push(Box::new(id.as_str().to_string()));

        let conn = self.store.get_conn()?;
        let refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();
        conn.execute(&query, refs.as_slice()).map_err(AresError::db)?;

        debug!(id = %id, "Requirement updated");
        let updated_req = self.get(id)?.ok_or_else(|| AresError::not_found("requirement", id.as_str()))?;

        let history = crate::history::RequirementHistory::new(self.store.clone());
        history.record_revision(
            id,
            &current,
            &updated_req,
            input.changed_by.as_deref(),
            input.change_reason.as_deref(),
        )?;

        Ok(updated_req)
    }

    pub fn delete(&self, id: &RequirementId) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        conn.execute(
            "DELETE FROM requirements WHERE id = ?1",
            params![id.as_str()],
        )
        .map_err(AresError::db)?;
        
        debug!(id = %id, "Requirement deleted");
        Ok(())
    }

    pub fn get(&self, id: &RequirementId) -> Result<Option<Requirement>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, project_id, title, description, requirement_type, status, priority, owner, created_at, updated_at, tags
                 FROM requirements WHERE id = ?1"
            )
            .map_err(AresError::db)?;

        let result = stmt.query_row(params![id.as_str()], row_to_requirement);

        match result {
            Ok(req) => Ok(Some(req)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AresError::db(e)),
        }
    }

    pub fn list(
        &self,
        project_id: &ProjectId,
        filter: RequirementFilter,
    ) -> Result<Vec<Requirement>, AresError> {
        let conn = self.store.get_conn()?;
        let mut where_clauses = vec!["project_id = ?1".to_string()];
        let mut bind_values: Vec<Box<dyn rusqlite::ToSql>> =
            vec![Box::new(project_id.as_str().to_string())];
        let mut idx = 2usize;

        if let Some(status) = filter.status {
            where_clauses.push(format!("status = ?{idx}"));
            let val = serde_json::to_string(&status).unwrap().replace("\"", "");
            bind_values.push(Box::new(val));
            idx += 1;
        }
        if let Some(priority) = filter.priority {
            where_clauses.push(format!("priority = ?{idx}"));
            let val = serde_json::to_string(&priority).unwrap().replace("\"", "");
            bind_values.push(Box::new(val));
            idx += 1;
        }
        if let Some(req_type) = filter.requirement_type {
            where_clauses.push(format!("requirement_type = ?{idx}"));
            let val = serde_json::to_string(&req_type).unwrap().replace("\"", "");
            bind_values.push(Box::new(val));
            idx += 1;
        }
        if let Some(owner) = filter.owner {
            where_clauses.push(format!("owner = ?{idx}"));
            bind_values.push(Box::new(owner));
            idx += 1;
        }
        if let Some(tag) = filter.tag {
            where_clauses.push(format!(
                "EXISTS (SELECT 1 FROM json_each(tags) WHERE value = ?{idx})"
            ));
            bind_values.push(Box::new(tag));
            idx += 1;
        }
        if let Some(since) = filter.since {
            where_clauses.push(format!("created_at >= ?{idx}"));
            bind_values.push(Box::new(since));
            idx += 1;
        }
        if let Some(until) = filter.until {
            where_clauses.push(format!("created_at <= ?{idx}"));
            bind_values.push(Box::new(until));
        }

        let where_sql = where_clauses.join(" AND ");
        let sql = format!(
            "SELECT id, project_id, title, description, requirement_type, status, priority, owner, created_at, updated_at, tags
             FROM requirements WHERE {where_sql}
             ORDER BY created_at DESC"
        );

        let mut stmt = conn.prepare(&sql).map_err(AresError::db)?;
        let refs: Vec<&dyn rusqlite::ToSql> = bind_values.iter().map(|b| b.as_ref()).collect();
        let rows = stmt.query_map(refs.as_slice(), row_to_requirement).map_err(AresError::db)?;

        rows.collect::<Result<Vec<_>, _>>().map_err(AresError::db)
    }

    pub fn search(&self, project_id: &ProjectId, query: &str) -> Result<Vec<Requirement>, AresError> {
        let conn = self.store.get_conn()?;
        let search_pattern = format!("%{}%", query);
        let mut stmt = conn
            .prepare(
                "SELECT id, project_id, title, description, requirement_type, status, priority, owner, created_at, updated_at, tags
                 FROM requirements 
                 WHERE project_id = ?1 AND (title LIKE ?2 OR description LIKE ?2)
                 ORDER BY created_at DESC"
            )
            .map_err(AresError::db)?;

        let rows = stmt
            .query_map(params![project_id.as_str(), search_pattern], row_to_requirement)
            .map_err(AresError::db)?;

        rows.collect::<Result<Vec<_>, _>>().map_err(AresError::db)
    }

    // --- Link Management (single source of truth) ---

    pub fn create_link(&self, link: &RequirementLink) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        let target_type_str = match link.target.target_type() {
            LinkTargetType::Requirement => "requirement",
            LinkTargetType::Decision => "decision",
            LinkTargetType::Architecture => "architecture",
            LinkTargetType::Code => "code",
            LinkTargetType::RuntimeMetric => "runtime_metric",
        };

        let target_id_val = match &link.target {
            LinkTarget::RuntimeMetric(r) => serde_json::to_string(r).unwrap_or_else(|_| r.id.as_str().to_string()),
            _ => link.target.target_id().to_string(),
        };

        conn.execute(
            "INSERT INTO requirement_links (
                id, source_requirement_id, target_id, target_type, relationship, created_at, created_by
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                link.id.as_str(),
                link.source_requirement_id.as_str(),
                target_id_val,
                target_type_str,
                link.relationship,
                link.created_at,
                link.created_by,
            ],
        )
        .map_err(AresError::db)?;

        debug!(id = %link.id, "Requirement link created");
        Ok(())
    }

    pub fn delete_link(&self, link_id: &RequirementLinkId) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        conn.execute(
            "DELETE FROM requirement_links WHERE id = ?1",
            params![link_id.as_str()],
        )
        .map_err(AresError::db)?;

        debug!(id = %link_id, "Requirement link deleted");
        Ok(())
    }

    pub fn get_links_from(
        &self,
        requirement_id: &RequirementId,
    ) -> Result<Vec<RequirementLink>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, source_requirement_id, target_id, target_type, relationship, created_at, created_by
                 FROM requirement_links WHERE source_requirement_id = ?1"
            )
            .map_err(AresError::db)?;

        let rows = stmt
            .query_map(params![requirement_id.as_str()], row_to_requirement_link)
            .map_err(AresError::db)?;

        rows.collect::<Result<Vec<_>, _>>().map_err(AresError::db)
    }

    pub fn get_links_to(
        &self,
        target_id: &str,
        target_type: LinkTargetType,
    ) -> Result<Vec<RequirementLink>, AresError> {
        let conn = self.store.get_conn()?;
        let target_type_str = match target_type {
            LinkTargetType::Requirement => "requirement",
            LinkTargetType::Decision => "decision",
            LinkTargetType::Architecture => "architecture",
            LinkTargetType::Code => "code",
            LinkTargetType::RuntimeMetric => "runtime_metric",
        };

        let mut stmt = conn
            .prepare(
                "SELECT id, source_requirement_id, target_id, target_type, relationship, created_at, created_by
                 FROM requirement_links WHERE target_id = ?1 AND target_type = ?2"
            )
            .map_err(AresError::db)?;

        let rows = stmt
            .query_map(params![target_id, target_type_str], row_to_requirement_link)
            .map_err(AresError::db)?;

        rows.collect::<Result<Vec<_>, _>>().map_err(AresError::db)
    }

    pub fn count_links_by_type(
        &self,
        requirement_id: &RequirementId,
    ) -> Result<LinkCounts, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT target_type, COUNT(*) 
                 FROM requirement_links 
                 WHERE source_requirement_id = ?1 
                 GROUP BY target_type"
            )
            .map_err(AresError::db)?;

        let mut rows = stmt
            .query(params![requirement_id.as_str()])
            .map_err(AresError::db)?;

        let mut counts = LinkCounts {
            decision_links: 0,
            architecture_links: 0,
            code_links: 0,
            requirement_links: 0,
            total: 0,
        };

        while let Some(row) = rows.next().map_err(AresError::db)? {
            let t_type: String = row.get(0).map_err(AresError::db)?;
            let count: usize = row.get(1).map_err(AresError::db)?;
            match t_type.as_str() {
                "decision" => counts.decision_links += count,
                "architecture" => counts.architecture_links += count,
                "code" => counts.code_links += count,
                "requirement" => counts.requirement_links += count,
                _ => {}
            }
            counts.total += count;
        }

        Ok(counts)
    }
}

// ─────────────────────────────────────────────────────────────────
// Row mappers
// ─────────────────────────────────────────────────────────────────

fn row_to_requirement(row: &rusqlite::Row<'_>) -> Result<Requirement, rusqlite::Error> {
    let type_str: String = row.get(4)?;
    let type_val = serde_json::from_str(&format!("\"{}\"", type_str)).unwrap_or(RequirementType::Functional);

    let status_str: String = row.get(5)?;
    let status_val = serde_json::from_str(&format!("\"{}\"", status_str)).unwrap_or(RequirementStatus::Draft);

    let priority_str: String = row.get(6)?;
    let priority_val = serde_json::from_str(&format!("\"{}\"", priority_str)).unwrap_or(RequirementPriority::Medium);

    let source_str: String = row.get(11).unwrap_or_else(|_| "product".to_string());
    let source_val = serde_json::from_str(&format!("\"{}\"", source_str)).unwrap_or(crate::models::RequirementSource::Product);

    let tags_str: String = row.get(10)?;
    let tags_val: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();

    Ok(Requirement {
        id: RequirementId::from(row.get::<_, String>(0)?),
        project_id: ProjectId::from(row.get::<_, String>(1)?),
        title: row.get(2)?,
        description: row.get(3)?,
        requirement_type: type_val,
        status: status_val,
        source: source_val,
        priority: priority_val,
        owner: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
        tags: tags_val,
    })
}

fn row_to_requirement_link(row: &rusqlite::Row<'_>) -> Result<RequirementLink, rusqlite::Error> {
    let target_id_str: String = row.get(2)?;
    let target_type_str: String = row.get(3)?;

    let target = match target_type_str.as_str() {
        "requirement" => LinkTarget::Requirement(RequirementId::from(target_id_str)),
        "decision" => LinkTarget::Decision(ares_core::DecisionId::from(target_id_str)),
        "architecture" => LinkTarget::Architecture(ares_core::ArchComponentId::from(target_id_str)),
        "code" => LinkTarget::Code(ares_core::CodeArtifactId::from(target_id_str)),
        "runtime_metric" => {
            let parsed_ref: crate::models::RuntimeMetricRef = serde_json::from_str(&target_id_str)
                .unwrap_or_else(|_| crate::models::RuntimeMetricRef {
                    id: ares_core::RuntimeMetricId::from(target_id_str.clone()),
                    provider: crate::models::MetricProvider::Internal,
                    external_id: None,
                });
            LinkTarget::RuntimeMetric(parsed_ref)
        },
        _ => LinkTarget::Code(ares_core::CodeArtifactId::from(target_id_str)), // Fallback
    };

    Ok(RequirementLink {
        id: RequirementLinkId::from(row.get::<_, String>(0)?),
        source_requirement_id: RequirementId::from(row.get::<_, String>(1)?),
        target,
        relationship: row.get(4)?,
        created_at: row.get(5)?,
        created_by: row.get(6)?,
    })
}
