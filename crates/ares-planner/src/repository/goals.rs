use crate::models::goal::{Goal, GoalPriority};
use ares_core::id::GoalId;
use ares_core::AresError;
use ares_store::db::Store;
use chrono::{TimeZone, Utc};
use rusqlite::{params, Row};
use std::sync::Arc;

pub struct SqliteGoalRepository {
    store: Arc<Store>,
}

impl SqliteGoalRepository {
    pub fn new(store: Arc<Store>) -> Self {
        Self { store }
    }

    pub fn create(&self, goal: &Goal) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        let priority_str = match goal.priority {
            GoalPriority::Low => "Low",
            GoalPriority::Medium => "Medium",
            GoalPriority::High => "High",
            GoalPriority::Critical => "Critical",
        };
        let deadline_ts = goal.deadline.map(|d| d.timestamp_millis());
        let created_at_ts = goal.created_at.timestamp_millis();
        let updated_at_ts = goal.updated_at.timestamp_millis();

        conn.execute(
            "INSERT INTO goals (id, title, description, priority, deadline, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                goal.id.as_str(),
                goal.title,
                goal.description,
                priority_str,
                deadline_ts,
                created_at_ts,
                updated_at_ts,
            ],
        )
        .map_err(AresError::db)?;

        Ok(())
    }

    pub fn get(&self, id: &GoalId) -> Result<Option<Goal>, AresError> {
        let conn = self.store.get_conn()?;
        let mut stmt = conn
            .prepare("SELECT id, title, description, priority, deadline, created_at, updated_at FROM goals WHERE id = ?1")
            .map_err(AresError::db)?;

        let mut rows = stmt.query(params![id.as_str()]).map_err(AresError::db)?;

        if let Some(row) = rows.next().map_err(AresError::db)? {
            Ok(Some(row_to_goal(row)?))
        } else {
            Ok(None)
        }
    }
}

fn row_to_goal(row: &Row<'_>) -> Result<Goal, AresError> {
    let id_str: String = row.get(0).map_err(AresError::db)?;
    let title: String = row.get(1).map_err(AresError::db)?;
    let description: Option<String> = row.get(2).map_err(AresError::db)?;
    let priority_str: String = row.get(3).map_err(AresError::db)?;
    let deadline_ts: Option<i64> = row.get(4).map_err(AresError::db)?;
    let created_at_ts: i64 = row.get(5).map_err(AresError::db)?;
    let updated_at_ts: i64 = row.get(6).map_err(AresError::db)?;

    let priority = match priority_str.as_str() {
        "Low" => GoalPriority::Low,
        "Medium" => GoalPriority::Medium,
        "High" => GoalPriority::High,
        "Critical" => GoalPriority::Critical,
        _ => GoalPriority::Medium,
    };

    let deadline = deadline_ts.map(|ts| Utc.timestamp_millis_opt(ts).unwrap());
    let created_at = Utc.timestamp_millis_opt(created_at_ts).unwrap();
    let updated_at = Utc.timestamp_millis_opt(updated_at_ts).unwrap();

    Ok(Goal {
        id: id_str.into(),
        title,
        description,
        priority,
        deadline,
        created_at,
        updated_at,
    })
}
