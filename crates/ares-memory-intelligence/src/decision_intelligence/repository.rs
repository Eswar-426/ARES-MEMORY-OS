use super::models::*;
use ares_core::AresError;
use ares_store::db::Store;
use rusqlite::params;

/// SQLite-backed repository for decision history.
pub struct DecisionIntelligenceRepository {
    store: Store,
}

impl DecisionIntelligenceRepository {
    pub fn new(store: Store) -> Self {
        Self { store }
    }

    /// Insert a new decision record.
    pub fn insert(&self, rec: &DecisionRecord) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        let alternatives_json = serde_json::to_string(&rec.alternatives)
            .map_err(|e| AresError::Serialization(e.to_string()))?;
        let context_json = serde_json::to_string(&rec.context)
            .map_err(|e| AresError::Serialization(e.to_string()))?;
        let outcome_str = rec.outcome.as_ref().map(|o| o.as_str().to_string());

        conn.execute(
            "INSERT INTO decision_history (id, episode_id, decision_type, question, chosen_option,
             alternatives, reasoning, confidence, outcome, context, created_at, resolved_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                rec.id,
                rec.episode_id,
                rec.decision_type.as_str(),
                rec.question,
                rec.chosen_option,
                alternatives_json,
                rec.reasoning,
                rec.confidence,
                outcome_str,
                context_json,
                rec.created_at,
                rec.resolved_at,
            ],
        )
        .map_err(AresError::db)?;
        Ok(())
    }

    /// Get a decision by ID.
    pub fn get(&self, id: &str) -> Result<Option<DecisionRecord>, AresError> {
        let conn = self.store.get_conn()?;
        let result = conn.query_row(
            "SELECT id, episode_id, decision_type, question, chosen_option, alternatives,
                    reasoning, confidence, outcome, context, created_at, resolved_at
             FROM decision_history WHERE id = ?1",
            params![id],
            Self::row_to_record,
        );
        match result {
            Ok(r) => Ok(Some(r)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AresError::db(e)),
        }
    }

    /// Query decision history with filters.
    pub fn query(&self, q: &DecisionQuery) -> Result<Vec<DecisionRecord>, AresError> {
        let conn = self.store.get_conn()?;
        let mut sql = String::from(
            "SELECT id, episode_id, decision_type, question, chosen_option, alternatives,
                    reasoning, confidence, outcome, context, created_at, resolved_at
             FROM decision_history WHERE 1=1",
        );
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        let mut idx = 1;

        if let Some(ref eid) = q.episode_id {
            sql.push_str(&format!(" AND episode_id = ?{}", idx));
            param_values.push(Box::new(eid.clone()));
            idx += 1;
        }
        if let Some(ref dt) = q.decision_type {
            sql.push_str(&format!(" AND decision_type = ?{}", idx));
            param_values.push(Box::new(dt.as_str().to_string()));
            idx += 1;
        }
        if let Some(ref outcome) = q.outcome {
            sql.push_str(&format!(" AND outcome = ?{}", idx));
            param_values.push(Box::new(outcome.as_str().to_string()));
            idx += 1;
        }
        if let Some(ref text) = q.search_text {
            sql.push_str(&format!(
                " AND (question LIKE ?{p} OR reasoning LIKE ?{p})",
                p = idx
            ));
            param_values.push(Box::new(format!("%{}%", text)));
            idx += 1;
        }
        let _ = idx;

        sql.push_str(" ORDER BY created_at DESC");
        let limit = q.limit.unwrap_or(100);
        sql.push_str(&format!(" LIMIT {}", limit));

        let params_ref: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();
        let mut stmt = conn.prepare(&sql).map_err(AresError::db)?;
        let rows = stmt
            .query_map(params_ref.as_slice(), Self::row_to_record)
            .map_err(AresError::db)?;

        let mut records = Vec::new();
        for row in rows {
            records.push(row.map_err(AresError::db)?);
        }
        Ok(records)
    }

    /// Update the outcome of a decision.
    pub fn set_outcome(&self, id: &str, outcome: &DecisionOutcomeType) -> Result<(), AresError> {
        let conn = self.store.get_conn()?;
        let now = chrono::Utc::now().timestamp_micros();
        let rows = conn
            .execute(
                "UPDATE decision_history SET outcome = ?1, resolved_at = ?2 WHERE id = ?3",
                params![outcome.as_str(), now, id],
            )
            .map_err(AresError::db)?;
        if rows == 0 {
            return Err(AresError::not_found("decision_history", id));
        }
        Ok(())
    }

    /// Count total decisions.
    pub fn count(&self) -> Result<u64, AresError> {
        let conn = self.store.get_conn()?;
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM decision_history", [], |row| {
                row.get(0)
            })
            .map_err(AresError::db)?;
        Ok(count as u64)
    }

    fn row_to_record(row: &rusqlite::Row<'_>) -> Result<DecisionRecord, rusqlite::Error> {
        let alternatives_str: String = row.get(5)?;
        let context_str: String = row.get(9)?;
        let outcome_str: Option<String> = row.get(8)?;
        Ok(DecisionRecord {
            id: row.get(0)?,
            episode_id: row.get(1)?,
            decision_type: DecisionType::from_str_val(&row.get::<_, String>(2)?),
            question: row.get(3)?,
            chosen_option: row.get(4)?,
            alternatives: serde_json::from_str(&alternatives_str).unwrap_or_default(),
            reasoning: row.get(6)?,
            confidence: row.get(7)?,
            outcome: outcome_str.map(|s| DecisionOutcomeType::from_str_val(&s)),
            context: serde_json::from_str(&context_str).unwrap_or_default(),
            created_at: row.get(10)?,
            resolved_at: row.get(11)?,
        })
    }
}

#[cfg(test)]
pub fn make_test_decision(id: &str) -> DecisionRecord {
    DecisionRecord {
        id: id.into(),
        episode_id: Some("ep_1".into()),
        decision_type: DecisionType::Technical,
        question: "Which framework?".into(),
        chosen_option: "Axum".into(),
        alternatives: vec![DecisionAlternative {
            option: "Actix".into(),
            reason_rejected: "Too complex".into(),
        }],
        reasoning: "Axum is simpler and integrates well with Tokio".into(),
        confidence: 0.85,
        outcome: None,
        context: serde_json::json!({}),
        created_at: chrono::Utc::now().timestamp_micros(),
        resolved_at: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::test_store;

    #[test]
    fn insert_and_get_decision() {
        let (store, _dir) = test_store();
        let repo = DecisionIntelligenceRepository::new(store);
        let rec = make_test_decision("d_1");
        repo.insert(&rec).unwrap();

        let fetched = repo.get("d_1").unwrap().unwrap();
        assert_eq!(fetched.chosen_option, "Axum");
        assert_eq!(fetched.alternatives.len(), 1);
    }

    #[test]
    fn get_nonexistent_returns_none() {
        let (store, _dir) = test_store();
        let repo = DecisionIntelligenceRepository::new(store);
        assert!(repo.get("nope").unwrap().is_none());
    }

    #[test]
    fn query_by_decision_type() {
        let (store, _dir) = test_store();
        let repo = DecisionIntelligenceRepository::new(store);
        repo.insert(&make_test_decision("d_tech")).unwrap();

        let mut strategic = make_test_decision("d_strat");
        strategic.decision_type = DecisionType::Strategic;
        repo.insert(&strategic).unwrap();

        let q = DecisionQuery {
            decision_type: Some(DecisionType::Technical),
            ..Default::default()
        };
        let results = repo.query(&q).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "d_tech");
    }

    #[test]
    fn query_by_search_text() {
        let (store, _dir) = test_store();
        let repo = DecisionIntelligenceRepository::new(store);
        repo.insert(&make_test_decision("d_search")).unwrap();

        let q = DecisionQuery {
            search_text: Some("framework".into()),
            ..Default::default()
        };
        let results = repo.query(&q).unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn set_outcome() {
        let (store, _dir) = test_store();
        let repo = DecisionIntelligenceRepository::new(store);
        repo.insert(&make_test_decision("d_out")).unwrap();

        repo.set_outcome("d_out", &DecisionOutcomeType::Positive)
            .unwrap();
        let fetched = repo.get("d_out").unwrap().unwrap();
        assert_eq!(fetched.outcome, Some(DecisionOutcomeType::Positive));
        assert!(fetched.resolved_at.is_some());
    }

    #[test]
    fn set_outcome_nonexistent_fails() {
        let (store, _dir) = test_store();
        let repo = DecisionIntelligenceRepository::new(store);
        let result = repo.set_outcome("nope", &DecisionOutcomeType::Negative);
        assert!(result.is_err());
    }

    #[test]
    fn count_decisions() {
        let (store, _dir) = test_store();
        let repo = DecisionIntelligenceRepository::new(store);
        assert_eq!(repo.count().unwrap(), 0);
        repo.insert(&make_test_decision("d_c1")).unwrap();
        repo.insert(&make_test_decision("d_c2")).unwrap();
        assert_eq!(repo.count().unwrap(), 2);
    }

    #[test]
    fn query_by_outcome() {
        let (store, _dir) = test_store();
        let repo = DecisionIntelligenceRepository::new(store);
        let mut rec = make_test_decision("d_pos");
        rec.outcome = Some(DecisionOutcomeType::Positive);
        repo.insert(&rec).unwrap();

        repo.insert(&make_test_decision("d_none")).unwrap();

        let q = DecisionQuery {
            outcome: Some(DecisionOutcomeType::Positive),
            ..Default::default()
        };
        let results = repo.query(&q).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "d_pos");
    }
}
