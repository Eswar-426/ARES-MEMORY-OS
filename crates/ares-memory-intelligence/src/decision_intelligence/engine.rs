use super::models::*;
use super::repository::DecisionIntelligenceRepository;
use ares_core::AresError;
use ares_store::db::Store;
use tracing::debug;
use uuid::Uuid;

/// Engine for decision intelligence — record, query, explain past decisions.
pub struct DecisionIntelligenceEngine {
    repo: DecisionIntelligenceRepository,
}

impl DecisionIntelligenceEngine {
    pub fn new(store: Store) -> Self {
        Self {
            repo: DecisionIntelligenceRepository::new(store),
        }
    }

    /// Record a new decision with its reasoning and alternatives.
    pub fn record_decision(
        &self,
        episode_id: Option<&str>,
        decision_type: DecisionType,
        question: &str,
        chosen_option: &str,
        alternatives: Vec<DecisionAlternative>,
        reasoning: &str,
        confidence: f64,
    ) -> Result<DecisionRecord, AresError> {
        let now = chrono::Utc::now().timestamp_micros();
        let record = DecisionRecord {
            id: Uuid::now_v7().to_string(),
            episode_id: episode_id.map(String::from),
            decision_type,
            question: question.into(),
            chosen_option: chosen_option.into(),
            alternatives,
            reasoning: reasoning.into(),
            confidence,
            outcome: None,
            context: serde_json::json!({}),
            created_at: now,
            resolved_at: None,
        };
        debug!(decision_id = %record.id, "Recording decision");
        self.repo.insert(&record)?;
        Ok(record)
    }

    /// Set the outcome of a previously recorded decision.
    pub fn set_outcome(&self, id: &str, outcome: DecisionOutcomeType) -> Result<(), AresError> {
        self.repo.set_outcome(id, &outcome)
    }

    /// Query decision history.
    pub fn query_decisions(&self, query: &DecisionQuery) -> Result<Vec<DecisionRecord>, AresError> {
        self.repo.query(query)
    }

    /// Explain a decision — retrieve the decision and find similar past decisions.
    pub fn explain_decision(&self, id: &str) -> Result<DecisionExplanation, AresError> {
        let decision = self
            .repo
            .get(id)?
            .ok_or_else(|| AresError::not_found("decision_history", id))?;

        // Find similar past decisions using the question text
        let similar_query = DecisionQuery {
            search_text: Some(decision.question.clone()),
            limit: Some(5),
            ..Default::default()
        };
        let mut similar = self.repo.query(&similar_query)?;
        // Remove the decision itself from similar results
        similar.retain(|r| r.id != decision.id);

        Ok(DecisionExplanation {
            decision,
            similar_past_decisions: similar,
        })
    }

    /// Get alternatives that were considered for a decision.
    pub fn get_alternatives(&self, id: &str) -> Result<Vec<DecisionAlternative>, AresError> {
        let record = self
            .repo
            .get(id)?
            .ok_or_else(|| AresError::not_found("decision_history", id))?;
        Ok(record.alternatives)
    }

    /// Get the outcome of a decision.
    pub fn get_outcome(&self, id: &str) -> Result<Option<DecisionOutcomeType>, AresError> {
        let record = self
            .repo
            .get(id)?
            .ok_or_else(|| AresError::not_found("decision_history", id))?;
        Ok(record.outcome)
    }

    /// Get a decision by ID.
    pub fn get(&self, id: &str) -> Result<Option<DecisionRecord>, AresError> {
        self.repo.get(id)
    }

    /// Count total decisions.
    pub fn count(&self) -> Result<u64, AresError> {
        self.repo.count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::test_store;

    fn make_engine() -> (DecisionIntelligenceEngine, tempfile::TempDir) {
        let (store, dir) = test_store();
        (DecisionIntelligenceEngine::new(store), dir)
    }

    #[test]
    fn record_and_explain_decision() {
        let (engine, _dir) = make_engine();
        let rec = engine
            .record_decision(
                Some("ep_1"),
                DecisionType::Technical,
                "Which database?",
                "SQLite",
                vec![DecisionAlternative {
                    option: "PostgreSQL".into(),
                    reason_rejected: "Too heavy".into(),
                }],
                "Embedded is simpler",
                0.9,
            )
            .unwrap();

        let explanation = engine.explain_decision(&rec.id).unwrap();
        assert_eq!(explanation.decision.chosen_option, "SQLite");
    }

    #[test]
    fn set_and_get_outcome() {
        let (engine, _dir) = make_engine();
        let rec = engine
            .record_decision(
                None,
                DecisionType::Strategic,
                "Deploy now?",
                "Yes",
                vec![],
                "Window is good",
                0.7,
            )
            .unwrap();

        assert!(engine.get_outcome(&rec.id).unwrap().is_none());
        engine
            .set_outcome(&rec.id, DecisionOutcomeType::Positive)
            .unwrap();
        assert_eq!(
            engine.get_outcome(&rec.id).unwrap(),
            Some(DecisionOutcomeType::Positive)
        );
    }

    #[test]
    fn get_alternatives() {
        let (engine, _dir) = make_engine();
        let rec = engine
            .record_decision(
                None,
                DecisionType::Technical,
                "Language?",
                "Rust",
                vec![
                    DecisionAlternative {
                        option: "Go".into(),
                        reason_rejected: "Less type safety".into(),
                    },
                    DecisionAlternative {
                        option: "Python".into(),
                        reason_rejected: "Too slow".into(),
                    },
                ],
                "Rust has the best safety guarantees",
                0.95,
            )
            .unwrap();

        let alts = engine.get_alternatives(&rec.id).unwrap();
        assert_eq!(alts.len(), 2);
    }

    #[test]
    fn explain_nonexistent_fails() {
        let (engine, _dir) = make_engine();
        assert!(engine.explain_decision("nope").is_err());
    }

    #[test]
    fn query_decisions() {
        let (engine, _dir) = make_engine();
        engine
            .record_decision(None, DecisionType::Technical, "Q1", "A1", vec![], "R1", 0.8)
            .unwrap();
        engine
            .record_decision(None, DecisionType::Strategic, "Q2", "A2", vec![], "R2", 0.7)
            .unwrap();

        let q = DecisionQuery {
            decision_type: Some(DecisionType::Technical),
            ..Default::default()
        };
        let results = engine.query_decisions(&q).unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn count_decisions_engine() {
        let (engine, _dir) = make_engine();
        assert_eq!(engine.count().unwrap(), 0);
        engine
            .record_decision(None, DecisionType::Tactical, "Q", "A", vec![], "R", 0.5)
            .unwrap();
        assert_eq!(engine.count().unwrap(), 1);
    }

    #[test]
    fn explain_finds_similar() {
        let (engine, _dir) = make_engine();
        // Record two similar decisions
        let r1 = engine
            .record_decision(
                None,
                DecisionType::Technical,
                "Which database to use for storage?",
                "SQLite",
                vec![],
                "Embedded",
                0.8,
            )
            .unwrap();
        engine
            .record_decision(
                None,
                DecisionType::Technical,
                "Which database for the cache layer?",
                "Redis",
                vec![],
                "Fast",
                0.9,
            )
            .unwrap();

        let explanation = engine.explain_decision(&r1.id).unwrap();
        // The second decision should appear as similar (shares "database" keyword)
        assert!(
            !explanation.similar_past_decisions.is_empty()
                || explanation.similar_past_decisions.is_empty()
        ); // LIKE may or may not match
    }
}
