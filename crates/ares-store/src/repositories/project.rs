use ares_core::{AresError, Project, ProjectId, ProjectMaturity};
use crate::db::Store;
use rusqlite::params;
use tracing::debug;

pub struct SqliteProjectRepository {
    store: Store,
}

impl SqliteProjectRepository {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    pub fn create(&self, project: &Project) -> Result<Project, AresError> {
        debug!(project_id = %project.id, name = %project.name, "Creating project");
        let conn = self.store.get_conn()?;
        conn.execute(
            "INSERT INTO projects (id, name, description, root_path, primary_language, domain, maturity, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                project.id.as_str(),
                project.name,
                project.description,
                project.root_path,
                project.primary_language,
                project.domain,
                project.maturity.as_str(),
                project.created_at,
                project.updated_at,
            ],
        ).map_err(AresError::db)?;
        self.get_by_id(&project.id)?.ok_or_else(|| AresError::not_found("project", project.id.as_str()))
    }

    pub fn get_by_id(&self, id: &ProjectId) -> Result<Option<Project>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, root_path, primary_language, domain, maturity,
                    created_at, updated_at, deleted_at
             FROM projects WHERE id = ?1 AND deleted_at IS NULL"
        ).map_err(AresError::db)?;

        let result = stmt.query_row(params![id.as_str()], row_to_project);
        match result {
            Ok(p)                                  => Ok(Some(p)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e)                                 => Err(AresError::db(e)),
        }
    }

    pub fn get_by_root_path(&self, root_path: &str) -> Result<Option<Project>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, root_path, primary_language, domain, maturity,
                    created_at, updated_at, deleted_at
             FROM projects WHERE root_path = ?1 AND deleted_at IS NULL"
        ).map_err(AresError::db)?;

        let result = stmt.query_row(params![root_path], row_to_project);
        match result {
            Ok(p)                                  => Ok(Some(p)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e)                                 => Err(AresError::db(e)),
        }
    }

    pub fn update(&self, project: &Project) -> Result<Project, AresError> {
        let conn = self.store.get_conn()?;
        let rows = conn.execute(
            "UPDATE projects SET name = ?1, description = ?2, primary_language = ?3,
                    domain = ?4, maturity = ?5, updated_at = ?6
             WHERE id = ?7 AND deleted_at IS NULL",
            params![
                project.name, project.description, project.primary_language,
                project.domain, project.maturity.as_str(), project.updated_at,
                project.id.as_str(),
            ],
        ).map_err(AresError::db)?;

        if rows == 0 {
            return Err(AresError::not_found("project", project.id.as_str()));
        }
        self.get_by_id(&project.id)?.ok_or_else(|| AresError::not_found("project", project.id.as_str()))
    }

    pub fn list_all(&self) -> Result<Vec<Project>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, root_path, primary_language, domain, maturity,
                    created_at, updated_at, deleted_at
             FROM projects WHERE deleted_at IS NULL ORDER BY created_at DESC"
        ).map_err(AresError::db)?;

        let rows = stmt.query_map([], row_to_project).map_err(AresError::db)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(AresError::db)
    }

    pub fn get_memory_counts(&self, project_id: &ProjectId) -> Result<std::collections::HashMap<String, u64>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn.prepare(
            "SELECT memory_type, COUNT(*) as cnt FROM memories
             WHERE project_id = ?1 AND deleted_at IS NULL AND status = 'active'
             GROUP BY memory_type"
        ).map_err(AresError::db)?;

        let rows = stmt.query_map(params![project_id.as_str()], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as u64))
        }).map_err(AresError::db)?;

        let mut map = std::collections::HashMap::new();
        for row in rows {
            let (k, v) = row.map_err(AresError::db)?;
            map.insert(k, v);
        }
        Ok(map)
    }
}

fn row_to_project(row: &rusqlite::Row<'_>) -> Result<Project, rusqlite::Error> {
    let maturity_str: String = row.get(6)?;
    let maturity = maturity_str.parse::<ProjectMaturity>()
        .unwrap_or_default();

    Ok(Project {
        id:               ProjectId::from(row.get::<_, String>(0)?),
        name:             row.get(1)?,
        description:      row.get(2)?,
        root_path:        row.get(3)?,
        primary_language: row.get(4)?,
        domain:           row.get(5)?,
        maturity,
        created_at:       row.get(7)?,
        updated_at:       row.get(8)?,
        deleted_at:       row.get(9)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_helpers::test_store;
    use ares_core::{ProjectId, new_id};
    use ares_core::types::event::now_micros;

    fn make_project(root: &str) -> Project {
        let now = now_micros();
        Project {
            id:               ProjectId::new(),
            name:             "Test Project".into(),
            description:      "A test project".into(),
            root_path:        root.into(),
            primary_language: "typescript".into(),
            domain:           "testing".into(),
            maturity:         ProjectMaturity::Greenfield,
            created_at:       now,
            updated_at:       now,
            deleted_at:       None,
        }
    }

    #[test]
    fn create_and_get_project() {
        let (store, _dir) = test_store();
        let repo = SqliteProjectRepository::new(store);
        let project = make_project("/tmp/test-project");
        let created = repo.create(&project).unwrap();
        assert_eq!(created.name, "Test Project");

        let fetched = repo.get_by_id(&created.id).unwrap().unwrap();
        assert_eq!(fetched.id, created.id);
        assert_eq!(fetched.root_path, "/tmp/test-project");
    }

    #[test]
    fn get_by_id_returns_none_for_missing() {
        let (store, _dir) = test_store();
        let repo = SqliteProjectRepository::new(store);
        let result = repo.get_by_id(&ProjectId::from(new_id())).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn get_by_root_path_works() {
        let (store, _dir) = test_store();
        let repo = SqliteProjectRepository::new(store);
        let project = make_project("/unique/path");
        repo.create(&project).unwrap();
        let found = repo.get_by_root_path("/unique/path").unwrap();
        assert!(found.is_some());
        let not_found = repo.get_by_root_path("/nonexistent").unwrap();
        assert!(not_found.is_none());
    }

    #[test]
    fn list_all_returns_all_projects() {
        let (store, _dir) = test_store();
        let repo = SqliteProjectRepository::new(store);
        repo.create(&make_project("/path/1")).unwrap();
        repo.create(&make_project("/path/2")).unwrap();
        let all = repo.list_all().unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn get_memory_counts_returns_empty_map_initially() {
        let (store, _dir) = test_store();
        let repo = SqliteProjectRepository::new(store);
        let project = make_project("/tmp/counts");
        let created = repo.create(&project).unwrap();
        let counts = repo.get_memory_counts(&created.id).unwrap();
        assert!(counts.is_empty());
    }
}
