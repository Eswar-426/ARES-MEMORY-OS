use crate::db::Store;
use ares_core::types::event::now_micros;
use ares_core::{
    AresError, CreateMemoryInput, Memory, MemoryFilter, MemoryId, MemoryPatch, MemorySearchResult,
    MemoryType, Page, Pagination, ProjectId,
};
use rusqlite::params;
use tracing::debug;

pub struct SqliteMemoryRepository {
    store: Store,
}

impl SqliteMemoryRepository {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    // ----------------------------------------------------------------
    // Create — inserts a new root-version memory record
    // ----------------------------------------------------------------
    pub fn create(&self, input: CreateMemoryInput) -> Result<Memory, AresError> {
        validate_memory_input(&input)?;
        let now = now_micros();
        let id = MemoryId::new();
        let conn = self.store.get_conn()?;

        conn.execute(
            "INSERT INTO memories
               (id, project_id, memory_type, title, content, status, version,
                parent_id, confidence, importance, source, ai_assisted, created_at, updated_at)
             VALUES (?1,?2,?3,?4,?5,'active',1,NULL,?6,?7,?8,?9,?10,?11)",
            params![
                id.as_str(),
                input.project_id.as_str(),
                input.memory_type.as_str(),
                input.title,
                input.content.to_string(),
                input.confidence.unwrap_or(1.0),
                input.importance.unwrap_or_default().as_str(),
                input.source.unwrap_or_default().as_str(),
                input.ai_assisted.unwrap_or(false) as i32,
                now,
                now,
            ],
        )
        .map_err(AresError::db)?;

        debug!(memory_id = %id, memory_type = %input.memory_type, "Memory created");
        self.get_by_id(&id)?
            .ok_or_else(|| AresError::not_found("memory", id.as_str()))
    }

    // ----------------------------------------------------------------
    // Read
    // ----------------------------------------------------------------
    pub fn get_by_id(&self, id: &MemoryId) -> Result<Option<Memory>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn.prepare(MEMORY_SELECT_SQL).map_err(AresError::db)?;
        let result = stmt.query_row(params![id.as_str()], row_to_memory);
        match result {
            Ok(m) => Ok(Some(m)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AresError::db(e)),
        }
    }

    // ----------------------------------------------------------------
    // List with filtering and pagination
    // ----------------------------------------------------------------
    pub fn list(
        &self,
        project_id: &ProjectId,
        filter: MemoryFilter,
        page: Pagination,
    ) -> Result<Page<Memory>, AresError> {
        let conn = self.store.get_conn()?;

        // Build dynamic WHERE clause
        let mut where_clauses = vec![
            "project_id = ?1".to_string(),
            "deleted_at IS NULL".to_string(),
            "parent_id IS NULL".to_string(), // only root versions in list
        ];
        let mut bind_values: Vec<Box<dyn rusqlite::ToSql>> =
            vec![Box::new(project_id.as_str().to_string())];
        let mut param_idx = 2usize;

        if let Some(mt) = &filter.memory_type {
            where_clauses.push(format!("memory_type = ?{param_idx}"));
            bind_values.push(Box::new(mt.as_str().to_string()));
            param_idx += 1;
        }
        if let Some(status) = &filter.status {
            where_clauses.push(format!("status = ?{param_idx}"));
            bind_values.push(Box::new(status.as_str().to_string()));
            param_idx += 1;
        }
        if let Some(since) = filter.since {
            where_clauses.push(format!("created_at >= ?{param_idx}"));
            bind_values.push(Box::new(since));
            param_idx += 1;
        }
        if let Some(until) = filter.until {
            where_clauses.push(format!("created_at <= ?{param_idx}"));
            bind_values.push(Box::new(until));
            param_idx += 1;
        }

        let where_sql = where_clauses.join(" AND ");

        // Count total
        let count_sql = format!("SELECT COUNT(*) FROM memories WHERE {where_sql}");
        let total: u64 = {
            let mut stmt = conn.prepare(&count_sql).map_err(AresError::db)?;
            let refs: Vec<&dyn rusqlite::ToSql> = bind_values.iter().map(|b| b.as_ref()).collect();
            stmt.query_row(refs.as_slice(), |r| r.get::<_, i64>(0))
                .map_err(AresError::db)? as u64
        };

        // Fetch page
        let offset = page.offset();
        let limit = page.limit();
        let select_sql = format!(
            "SELECT id, project_id, memory_type, title, content, status, version,
                    parent_id, confidence, importance, source, ai_assisted, created_at, updated_at, deleted_at
             FROM memories WHERE {where_sql}
             ORDER BY updated_at DESC
             LIMIT ?{param_idx} OFFSET ?{}",
            param_idx + 1
        );
        bind_values.push(Box::new(limit as i64));
        bind_values.push(Box::new(offset as i64));

        let mut stmt = conn.prepare(&select_sql).map_err(AresError::db)?;
        let refs: Vec<&dyn rusqlite::ToSql> = bind_values.iter().map(|b| b.as_ref()).collect();
        let rows = stmt
            .query_map(refs.as_slice(), row_to_memory)
            .map_err(AresError::db)?;
        let items: Vec<Memory> = rows.collect::<Result<_, _>>().map_err(AresError::db)?;

        Ok(Page::new(items, total, page.page, page.page_size))
    }

    // ----------------------------------------------------------------
    // Update — creates a new version record, keeps old as parent
    // ----------------------------------------------------------------
    pub fn update(&self, id: &MemoryId, patch: MemoryPatch) -> Result<Memory, AresError> {
        let existing = self
            .get_by_id(id)?
            .ok_or_else(|| AresError::not_found("memory", id.as_str()))?;

        let now = now_micros();
        let new_id = MemoryId::new();

        // New version inherits from existing, applying patch
        let new_title = patch.title.unwrap_or(existing.title.clone());
        let new_content = patch.content.unwrap_or(existing.content.clone());
        let new_status = patch.status.unwrap_or(existing.status.clone());
        let new_confidence = patch.confidence.unwrap_or(existing.confidence);
        let new_importance = patch.importance.unwrap_or(existing.importance.clone());
        let new_version = existing.version + 1;

        let conn = self.store.get_conn()?;
        conn.execute(
            "INSERT INTO memories
               (id, project_id, memory_type, title, content, status, version,
                parent_id, confidence, importance, source, ai_assisted, created_at, updated_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)",
            params![
                new_id.as_str(),
                existing.project_id.as_str(),
                existing.memory_type.as_str(),
                new_title,
                new_content.to_string(),
                new_status.as_str(),
                new_version,
                existing.id.as_str(), // parent_id = old ID
                new_confidence,
                new_importance.as_str(),
                existing.source.as_str(),
                existing.ai_assisted as i32,
                now,
                now,
            ],
        )
        .map_err(AresError::db)?;

        debug!(
            old_id = %id, new_id = %new_id,
            version = new_version, "Memory version created"
        );
        self.get_by_id(&new_id)?
            .ok_or_else(|| AresError::not_found("memory", new_id.as_str()))
    }

    // ----------------------------------------------------------------
    // Soft delete
    // ----------------------------------------------------------------
    pub fn soft_delete(&self, id: &MemoryId) -> Result<(), AresError> {
        let now = now_micros();
        let conn = self.store.get_conn()?;
        let rows = conn.execute(
            "UPDATE memories SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2 AND deleted_at IS NULL",
            params![now, id.as_str()],
        ).map_err(AresError::db)?;

        if rows == 0 {
            return Err(AresError::not_found("memory", id.as_str()));
        }
        Ok(())
    }

    // ----------------------------------------------------------------
    // Full-text search (FTS5)
    // ----------------------------------------------------------------
    pub fn search(
        &self,
        project_id: &ProjectId,
        query: &str,
        limit: u32,
    ) -> Result<Vec<MemorySearchResult>, AresError> {
        if query.trim().is_empty() {
            return Ok(vec![]);
        }

        let conn = self.store.get_conn()?;
        // FTS5 rank() returns negative values (more negative = better match)
        let mut stmt = conn
            .prepare(
                "SELECT m.id, m.project_id, m.memory_type, m.title, m.content,
                    m.status, m.version, m.parent_id, m.confidence, m.importance, m.source,
                    m.ai_assisted, m.created_at, m.updated_at, m.deleted_at,
                    fts.rank,
                    snippet(memories_fts, 1, '[', ']', '...', 10) as snip
             FROM memories_fts fts
             JOIN memories m ON m.id = fts.memory_id
             WHERE memories_fts MATCH ?1
               AND m.project_id = ?2
               AND m.deleted_at IS NULL
               AND m.status = 'active'
             ORDER BY fts.rank
             LIMIT ?3",
            )
            .map_err(AresError::db)?;

        let limit_i = limit as i64;
        let rows = stmt
            .query_map(params![query, project_id.as_str(), limit_i], |row| {
                let memory = row_to_memory_extended(row)?;
                let rank: f64 = row.get(15)?;
                let snippet: String = row.get(16)?;
                Ok(MemorySearchResult {
                    memory,
                    score: -rank, // flip sign: higher = better
                    snippet,
                })
            })
            .map_err(AresError::db)?;

        rows.collect::<Result<Vec<_>, _>>().map_err(AresError::db)
    }

    // ----------------------------------------------------------------
    // Version history — returns all versions of a memory (root first)
    // ----------------------------------------------------------------
    pub fn get_version_history(&self, root_id: &MemoryId) -> Result<Vec<Memory>, AresError> {
        let conn = self.store.get_conn()?;
        // Walk the parent_id chain using recursive CTE
        let mut stmt = conn
            .prepare(
                "WITH RECURSIVE chain AS (
               SELECT id, parent_id, version FROM memories WHERE id = ?1
               UNION ALL
               SELECT m.id, m.parent_id, m.version FROM memories m
               JOIN chain c ON m.parent_id = c.id
             )
             SELECT m.id, m.project_id, m.memory_type, m.title, m.content,
                    m.status, m.version, m.parent_id, m.confidence, m.importance, m.source,
                    m.ai_assisted, m.created_at, m.updated_at, m.deleted_at
             FROM memories m
             JOIN chain c ON m.id = c.id
             ORDER BY m.version ASC",
            )
            .map_err(AresError::db)?;

        let rows = stmt
            .query_map(params![root_id.as_str()], row_to_memory)
            .map_err(AresError::db)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(AresError::db)
    }
}

// ─────────────────────────────────────────────────────────────────
// Input validation
// ─────────────────────────────────────────────────────────────────

fn validate_memory_input(input: &CreateMemoryInput) -> Result<(), AresError> {
    if input.title.trim().is_empty() {
        return Err(AresError::validation("title cannot be empty"));
    }
    if input.title.len() > 500 {
        return Err(AresError::validation("title exceeds 500 character limit"));
    }
    let content_str = input.content.to_string();
    if content_str.len() > 100_000 {
        return Err(AresError::validation("content exceeds 100KB limit"));
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

// ─────────────────────────────────────────────────────────────────
// Row mappers
// ─────────────────────────────────────────────────────────────────

const MEMORY_SELECT_SQL: &str =
    "SELECT id, project_id, memory_type, title, content, status, version,
            parent_id, confidence, importance, source, ai_assisted, created_at, updated_at, deleted_at
     FROM memories WHERE id = ?1";

fn row_to_memory(row: &rusqlite::Row<'_>) -> Result<Memory, rusqlite::Error> {
    row_to_memory_extended(row)
}

fn row_to_memory_extended(row: &rusqlite::Row<'_>) -> Result<Memory, rusqlite::Error> {
    let memory_type_str: String = row.get(2)?;
    let content_str: String = row.get(4)?;
    let status_str: String = row.get(5)?;
    let parent_id_str: Option<String> = row.get(7)?;
    let importance_str: String = row.get(9)?;
    let source_str: String = row.get(10)?;
    let ai_assisted: i32 = row.get(11)?;

    Ok(Memory {
        id: MemoryId::from(row.get::<_, String>(0)?),
        project_id: ProjectId::from(row.get::<_, String>(1)?),
        memory_type: memory_type_str.parse().unwrap_or(MemoryType::Feature),
        title: row.get(3)?,
        content: serde_json::from_str(&content_str).unwrap_or(serde_json::Value::Null),
        status: status_str.parse().unwrap_or_default(),
        version: row.get::<_, i64>(6)? as u32,
        parent_id: parent_id_str.map(MemoryId::from),
        confidence: row.get(8)?,
        importance: importance_str.parse().unwrap_or_default(),
        source: source_str.parse().unwrap_or_default(),
        ai_assisted: ai_assisted != 0,
        created_at: row.get(12)?,
        updated_at: row.get(13)?,
        deleted_at: row.get(14)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_helpers::test_store;
    use ares_core::{new_id, CreateMemoryInput, MemoryFilter, MemoryType, Pagination};

    fn make_input(project_id: ProjectId, title: &str) -> CreateMemoryInput {
        CreateMemoryInput {
            project_id,
            memory_type: MemoryType::Decision,
            title: title.into(),
            content: serde_json::json!({ "text": "test content" }),
            confidence: None,
            importance: None,
            source: None,
            ai_assisted: None,
        }
    }

    fn setup_project(store: &Store) -> ProjectId {
        use crate::repositories::project::SqliteProjectRepository;
        use ares_core::types::event::now_micros;
        use ares_core::{Project, ProjectMaturity};
        let repo = SqliteProjectRepository::new(store.clone());
        let now = now_micros();
        let project = Project {
            id: ProjectId::new(),
            name: "test".into(),
            description: "".into(),
            root_path: format!("/tmp/test-{}", new_id()),
            primary_language: "ts".into(),
            domain: "".into(),
            maturity: ProjectMaturity::Greenfield,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };
        let created = repo.create(&project).unwrap();
        created.id
    }

    #[test]
    fn create_and_get_memory() {
        let (store, _dir) = test_store();
        let project_id = setup_project(&store);
        let repo = SqliteMemoryRepository::new(store);
        let input = make_input(project_id.clone(), "JWT Authentication Decision");
        let memory = repo.create(input).unwrap();
        assert_eq!(memory.version, 1);
        assert!(memory.parent_id.is_none());

        let fetched = repo.get_by_id(&memory.id).unwrap().unwrap();
        assert_eq!(fetched.title, "JWT Authentication Decision");
    }

    #[test]
    fn update_creates_new_version() {
        let (store, _dir) = test_store();
        let project_id = setup_project(&store);
        let repo = SqliteMemoryRepository::new(store);
        let input = make_input(project_id, "Original Title");
        let v1 = repo.create(input).unwrap();
        assert_eq!(v1.version, 1);

        let v2 = repo
            .update(
                &v1.id,
                MemoryPatch {
                    title: Some("Updated Title".into()),
                    ..Default::default()
                },
            )
            .unwrap();
        assert_eq!(v2.version, 2);
        assert_eq!(v2.parent_id, Some(v1.id.clone()));
        assert_eq!(v2.title, "Updated Title");
    }

    #[test]
    fn version_history_returns_chain() {
        let (store, _dir) = test_store();
        let project_id = setup_project(&store);
        let repo = SqliteMemoryRepository::new(store);
        let v1 = repo.create(make_input(project_id, "v1")).unwrap();
        let v2 = repo
            .update(
                &v1.id,
                MemoryPatch {
                    title: Some("v2".into()),
                    ..Default::default()
                },
            )
            .unwrap();
        let v3 = repo
            .update(
                &v2.id,
                MemoryPatch {
                    title: Some("v3".into()),
                    ..Default::default()
                },
            )
            .unwrap();

        let history = repo.get_version_history(&v1.id).unwrap();
        assert_eq!(history.len(), 3);
        assert_eq!(history[0].version, 1);
        assert_eq!(history[2].version, 3);
        let _ = v3; // used
    }

    #[test]
    fn soft_delete_hides_memory() {
        let (store, _dir) = test_store();
        let project_id = setup_project(&store);
        let repo = SqliteMemoryRepository::new(store);
        let mem = repo
            .create(make_input(project_id.clone(), "To Delete"))
            .unwrap();
        repo.soft_delete(&mem.id).unwrap();

        let result = repo
            .list(&project_id, MemoryFilter::default(), Pagination::default())
            .unwrap();
        assert_eq!(result.total, 0);
    }

    #[test]
    fn fts_search_finds_created_memory() {
        let (store, _dir) = test_store();
        let project_id = setup_project(&store);
        let repo = SqliteMemoryRepository::new(store);
        repo.create(make_input(
            project_id.clone(),
            "JWT Authentication Decision",
        ))
        .unwrap();

        let results = repo.search(&project_id, "authentication", 10).unwrap();
        assert!(!results.is_empty());
        assert!(results[0].memory.title.contains("Authentication"));
    }

    #[test]
    fn validation_rejects_empty_title() {
        let (store, _dir) = test_store();
        let project_id = setup_project(&store);
        let repo = SqliteMemoryRepository::new(store);
        let input = CreateMemoryInput {
            project_id,
            memory_type: MemoryType::Decision,
            title: "  ".into(),
            content: serde_json::json!({}),
            confidence: None,
            importance: None,
            source: None,
            ai_assisted: None,
        };
        let err = repo.create(input).unwrap_err();
        assert!(matches!(err, AresError::Validation(_)));
    }
}
