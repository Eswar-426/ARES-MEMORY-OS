use super::models::*;
use super::repository::EvolutionRepository;
use crate::semantic::models::{SemanticMemoryType, SemanticQuery};
use crate::semantic::repository::SemanticRepository;
use ares_core::AresError;
use ares_store::db::Store;
use chrono::Utc;
use tracing::debug;
use uuid::Uuid;

/// Engine for knowledge evolution — confidence tracking, decay, contradiction detection, reinforcement.
pub struct KnowledgeEvolutionEngine {
    evolution_repo: EvolutionRepository,
    semantic_repo: SemanticRepository,
    config: DecayConfig,
}

impl KnowledgeEvolutionEngine {
    pub fn new(store: Store, config: DecayConfig) -> Self {
        Self {
            evolution_repo: EvolutionRepository::new(store.clone()),
            semantic_repo: SemanticRepository::new(store),
            config,
        }
    }

    pub fn with_defaults(store: Store) -> Self {
        Self::new(store, DecayConfig::default())
    }

    /// Reinforce a semantic memory — increase confidence based on new evidence.
    pub fn reinforce(
        &self,
        memory_id: &str,
        boost: f64,
        reason: &str,
        source_episode_id: Option<&str>,
    ) -> Result<KnowledgeEvolutionEntry, AresError> {
        let mem = self
            .semantic_repo
            .get(memory_id)?
            .ok_or_else(|| AresError::not_found("semantic_memory", memory_id))?;

        let new_confidence = (mem.confidence + boost).clamp(0.0, 1.0);
        self.semantic_repo
            .update_confidence(memory_id, new_confidence, mem.evidence_count + 1)?;

        let entry = KnowledgeEvolutionEntry {
            id: Uuid::now_v7().to_string(),
            semantic_memory_id: memory_id.into(),
            event_type: EvolutionEventType::Reinforcement,
            old_confidence: mem.confidence,
            new_confidence,
            reason: reason.into(),
            source_episode_id: source_episode_id.map(String::from),
            created_at: Utc::now().timestamp_micros(),
        };
        self.evolution_repo.insert(&entry)?;
        debug!(
            memory_id,
            old = mem.confidence,
            new = new_confidence,
            "Reinforced memory"
        );
        Ok(entry)
    }

    /// Apply confidence decay to a semantic memory.
    pub fn apply_decay(
        &self,
        memory_id: &str,
        days_since_last_use: f64,
    ) -> Result<Option<KnowledgeEvolutionEntry>, AresError> {
        let mem = self
            .semantic_repo
            .get(memory_id)?
            .ok_or_else(|| AresError::not_found("semantic_memory", memory_id))?;

        let decay_factor = self.config.daily_rate.powf(days_since_last_use);
        let new_confidence = (mem.confidence * decay_factor).max(0.0);

        if (mem.confidence - new_confidence).abs() < 0.001 {
            return Ok(None); // No meaningful change
        }

        let event_type = if new_confidence < self.config.min_confidence {
            EvolutionEventType::Deprecated
        } else {
            EvolutionEventType::DecayApplied
        };

        self.semantic_repo
            .update_confidence(memory_id, new_confidence, mem.evidence_count)?;

        let entry = KnowledgeEvolutionEntry {
            id: Uuid::now_v7().to_string(),
            semantic_memory_id: memory_id.into(),
            event_type,
            old_confidence: mem.confidence,
            new_confidence,
            reason: format!("Decay after {:.1} days", days_since_last_use),
            source_episode_id: None,
            created_at: Utc::now().timestamp_micros(),
        };
        self.evolution_repo.insert(&entry)?;
        Ok(Some(entry))
    }

    /// Detect contradictions among semantic memories on the same subject.
    pub fn detect_contradictions(&self) -> Result<Vec<ContradictionDetection>, AresError> {
        // Get all relationship-type memories
        let q = SemanticQuery {
            memory_type: Some(SemanticMemoryType::Relationship),
            ..Default::default()
        };
        let memories = self.semantic_repo.query(&q)?;

        let mut contradictions = Vec::new();

        // Check for contradictions: same subject + predicate but different object
        for (i, a) in memories.iter().enumerate() {
            for b in memories.iter().skip(i + 1) {
                if a.subject == b.subject
                    && a.predicate == b.predicate
                    && a.object != b.object
                    && a.confidence > 0.5
                    && b.confidence > 0.5
                {
                    contradictions.push(ContradictionDetection {
                        memory_id_a: a.id.clone(),
                        memory_id_b: b.id.clone(),
                        subject: a.subject.clone(),
                        conflict_description: format!(
                            "{} {} {} vs {} {} {}",
                            a.subject, a.predicate, a.object, b.subject, b.predicate, b.object
                        ),
                        confidence: (a.confidence + b.confidence) / 2.0,
                    });
                }
            }
        }

        Ok(contradictions)
    }

    /// Record a contradiction between two memories by reducing confidence on both.
    pub fn record_contradiction(
        &self,
        memory_id_a: &str,
        memory_id_b: &str,
        reason: &str,
    ) -> Result<(), AresError> {
        // Reduce confidence on both
        let mem_a = self
            .semantic_repo
            .get(memory_id_a)?
            .ok_or_else(|| AresError::not_found("semantic_memory", memory_id_a))?;
        let mem_b = self
            .semantic_repo
            .get(memory_id_b)?
            .ok_or_else(|| AresError::not_found("semantic_memory", memory_id_b))?;

        let penalty = 0.1;
        let new_a = (mem_a.confidence - penalty).max(0.0);
        let new_b = (mem_b.confidence - penalty).max(0.0);

        self.semantic_repo
            .update_confidence(memory_id_a, new_a, mem_a.evidence_count)?;
        self.semantic_repo
            .update_confidence(memory_id_b, new_b, mem_b.evidence_count)?;

        let now = Utc::now().timestamp_micros();
        for (mid, old, new) in [
            (memory_id_a, mem_a.confidence, new_a),
            (memory_id_b, mem_b.confidence, new_b),
        ] {
            let entry = KnowledgeEvolutionEntry {
                id: Uuid::now_v7().to_string(),
                semantic_memory_id: mid.into(),
                event_type: EvolutionEventType::ContradictionDetected,
                old_confidence: old,
                new_confidence: new,
                reason: reason.into(),
                source_episode_id: None,
                created_at: now,
            };
            self.evolution_repo.insert(&entry)?;
        }

        Ok(())
    }

    /// Get evolution history for a memory.
    pub fn get_history(&self, memory_id: &str) -> Result<Vec<KnowledgeEvolutionEntry>, AresError> {
        self.evolution_repo.get_history(memory_id)
    }

    /// Count evolution events.
    pub fn count_events(&self) -> Result<u64, AresError> {
        self.evolution_repo.count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::models::SemanticMemory;
    use crate::semantic::repository::make_test_semantic;
    use crate::test_utils::test_store;

    fn make_engine() -> (KnowledgeEvolutionEngine, Store, tempfile::TempDir) {
        let (store, dir) = test_store();
        let engine = KnowledgeEvolutionEngine::with_defaults(store.clone());
        (engine, store, dir)
    }

    fn setup_memory(store: &Store, id: &str, confidence: f64) {
        let repo = SemanticRepository::new(store.clone());
        let mut mem = make_test_semantic(id, SemanticMemoryType::Entity);
        mem.confidence = confidence;
        repo.insert(&mem).unwrap();
    }

    #[test]
    fn reinforce_increases_confidence() {
        let (engine, store, _dir) = make_engine();
        setup_memory(&store, "sm_reinf", 0.7);

        let entry = engine
            .reinforce("sm_reinf", 0.1, "New evidence", None)
            .unwrap();
        assert!((entry.new_confidence - 0.8).abs() < f64::EPSILON);
        assert_eq!(entry.event_type, EvolutionEventType::Reinforcement);
    }

    #[test]
    fn reinforce_clamps_at_1() {
        let (engine, store, _dir) = make_engine();
        setup_memory(&store, "sm_max", 0.95);

        let entry = engine
            .reinforce("sm_max", 0.5, "Strong evidence", None)
            .unwrap();
        assert!((entry.new_confidence - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn reinforce_nonexistent_fails() {
        let (engine, _, _dir) = make_engine();
        assert!(engine.reinforce("nope", 0.1, "test", None).is_err());
    }

    #[test]
    fn apply_decay_reduces_confidence() {
        let (engine, store, _dir) = make_engine();
        setup_memory(&store, "sm_decay", 0.8);

        let entry = engine.apply_decay("sm_decay", 100.0).unwrap();
        assert!(entry.is_some());
        let e = entry.unwrap();
        assert!(e.new_confidence < 0.8);
        assert_eq!(e.event_type, EvolutionEventType::DecayApplied);
    }

    #[test]
    fn apply_decay_deprecates_below_min() {
        let (engine, store, _dir) = make_engine();
        setup_memory(&store, "sm_dep", 0.15);

        let entry = engine.apply_decay("sm_dep", 1000.0).unwrap();
        if let Some(e) = entry {
            assert_eq!(e.event_type, EvolutionEventType::Deprecated);
        }
    }

    #[test]
    fn apply_decay_no_change_for_small_delta() {
        let (engine, store, _dir) = make_engine();
        setup_memory(&store, "sm_small", 0.8);

        // Very small time delta = negligible decay
        let entry = engine.apply_decay("sm_small", 0.001).unwrap();
        assert!(entry.is_none());
    }

    #[test]
    fn detect_contradictions_empty() {
        let (engine, _, _dir) = make_engine();
        let contradictions = engine.detect_contradictions().unwrap();
        assert!(contradictions.is_empty());
    }

    #[test]
    fn detect_contradictions_finds_conflict() {
        let (engine, store, _dir) = make_engine();
        let repo = SemanticRepository::new(store);

        let now = Utc::now().timestamp_micros();
        repo.insert(&SemanticMemory {
            id: "sm_c1".into(),
            source_episode_id: None,
            memory_type: SemanticMemoryType::Relationship,
            subject: "Auth".into(),
            predicate: "USES".into(),
            object: "JWT".into(),
            confidence: 0.9,
            evidence_count: 1,
            tags: vec![],
            created_at: now,
            updated_at: now,
        })
        .unwrap();
        repo.insert(&SemanticMemory {
            id: "sm_c2".into(),
            source_episode_id: None,
            memory_type: SemanticMemoryType::Relationship,
            subject: "Auth".into(),
            predicate: "USES".into(),
            object: "Sessions".into(),
            confidence: 0.8,
            evidence_count: 1,
            tags: vec![],
            created_at: now,
            updated_at: now,
        })
        .unwrap();

        let contradictions = engine.detect_contradictions().unwrap();
        assert_eq!(contradictions.len(), 1);
        assert_eq!(contradictions[0].subject, "Auth");
    }

    #[test]
    fn record_contradiction_reduces_both() {
        let (engine, store, _dir) = make_engine();
        let repo = SemanticRepository::new(store);

        let now = Utc::now().timestamp_micros();
        repo.insert(&SemanticMemory {
            id: "sm_rc1".into(),
            source_episode_id: None,
            memory_type: SemanticMemoryType::Relationship,
            subject: "X".into(),
            predicate: "Y".into(),
            object: "Z1".into(),
            confidence: 0.9,
            evidence_count: 1,
            tags: vec![],
            created_at: now,
            updated_at: now,
        })
        .unwrap();
        repo.insert(&SemanticMemory {
            id: "sm_rc2".into(),
            source_episode_id: None,
            memory_type: SemanticMemoryType::Relationship,
            subject: "X".into(),
            predicate: "Y".into(),
            object: "Z2".into(),
            confidence: 0.8,
            evidence_count: 1,
            tags: vec![],
            created_at: now,
            updated_at: now,
        })
        .unwrap();

        engine
            .record_contradiction("sm_rc1", "sm_rc2", "Conflicting objects")
            .unwrap();

        let a = repo.get("sm_rc1").unwrap().unwrap();
        let b = repo.get("sm_rc2").unwrap().unwrap();
        assert!((a.confidence - 0.8).abs() < f64::EPSILON);
        assert!((b.confidence - 0.7).abs() < f64::EPSILON);
    }

    #[test]
    fn get_history() {
        let (engine, store, _dir) = make_engine();
        setup_memory(&store, "sm_hst", 0.7);

        engine.reinforce("sm_hst", 0.05, "R1", None).unwrap();
        engine.reinforce("sm_hst", 0.05, "R2", None).unwrap();

        let history = engine.get_history("sm_hst").unwrap();
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn count_events() {
        let (engine, store, _dir) = make_engine();
        setup_memory(&store, "sm_ev_cnt", 0.7);
        assert_eq!(engine.count_events().unwrap(), 0);

        engine.reinforce("sm_ev_cnt", 0.1, "test", None).unwrap();
        assert_eq!(engine.count_events().unwrap(), 1);
    }
}
