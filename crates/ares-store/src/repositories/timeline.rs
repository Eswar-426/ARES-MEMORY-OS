use crate::db::Store;
use ares_core::{AresError, AresEvent, ProjectId, types::event::TimelineFilter};
use ares_core::types::pagination::{Page, Pagination};

pub struct SqliteTimelineRepository {
    store: Store,
}

impl SqliteTimelineRepository {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    pub fn list_paginated(
        &self,
        project_id: &ProjectId,
        filter: TimelineFilter,
        pagination: &Pagination,
    ) -> Result<Page<AresEvent>, AresError> {
        let conn = self.store.get_conn()?;
        let mut where_clauses = vec!["project_id = ?1".to_string()];
        let mut bind_values: Vec<Box<dyn rusqlite::ToSql>> =
            vec![Box::new(project_id.as_str().to_string())];
        let mut idx = 2usize;

        if let Some(event_types) = &filter.event_types {
            if !event_types.is_empty() {
                let placeholders: Vec<String> = (0..event_types.len()).map(|_| format!("?{idx}")).collect();
                where_clauses.push(format!("event_type IN ({})", placeholders.join(", ")));
                for et in event_types {
                    bind_values.push(Box::new(et.as_str().to_string()));
                    idx += 1;
                }
            }
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

        let _ = idx; // suppress unused warning

        let where_sql = where_clauses.join(" AND ");

        // Count total
        let count_sql = format!("SELECT COUNT(*) FROM events WHERE {where_sql}");
        let mut count_stmt = conn.prepare(&count_sql).map_err(AresError::db)?;
        let refs: Vec<&dyn rusqlite::ToSql> = bind_values.iter().map(|b| b.as_ref()).collect();
        let total: u64 = count_stmt
            .query_row(refs.as_slice(), |row| row.get(0))
            .map_err(AresError::db)?;

        // Fetch paginated
        let offset = pagination.offset();
        let limit = pagination.limit();
        let sql = format!(
            "SELECT id, project_id, event_type, payload, source, created_at
             FROM events WHERE {where_sql}
             ORDER BY created_at DESC
             LIMIT {limit} OFFSET {offset}"
        );

        let mut stmt = conn.prepare(&sql).map_err(AresError::db)?;
        let rows = stmt
            .query_map(refs.as_slice(), crate::repositories::event::row_to_event)
            .map_err(AresError::db)?;

        let items = rows.collect::<Result<Vec<_>, _>>().map_err(AresError::db)?;

        Ok(Page::new(items, total, pagination.page, pagination.page_size))
    }
}
