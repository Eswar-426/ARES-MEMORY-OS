use super::models::*;
use super::repository::SemanticRepository;
use ares_core::AresError;
use ares_store::db::Store;
use chrono::Utc;
use tracing::debug;
use uuid::Uuid;

/// Engine for semantic memory operations — extract concepts, entities,
/// relationships, generate facts, and manage confidence.
pub struct SemanticMemoryEngine {
    repo: SemanticRepository,
}

/// Known technical keywords for entity extraction.
const TECH_KEYWORDS: &[(&str, &str)] = &[
    ("jwt", "technology"),
    ("oauth", "technology"),
    ("authentication", "concept"),
    ("authorization", "concept"),
    ("database", "infrastructure"),
    ("kubernetes", "platform"),
    ("docker", "platform"),
    ("api", "concept"),
    ("rest", "protocol"),
    ("graphql", "protocol"),
    ("grpc", "protocol"),
    ("microservice", "architecture"),
    ("monolith", "architecture"),
    ("cache", "infrastructure"),
    ("redis", "technology"),
    ("postgres", "technology"),
    ("sqlite", "technology"),
    ("deploy", "operation"),
    ("deployment", "operation"),
    ("testing", "process"),
    ("ci/cd", "process"),
    ("pipeline", "concept"),
    ("encryption", "concept"),
    ("ssl", "technology"),
    ("tls", "technology"),
    ("webhook", "concept"),
    ("queue", "infrastructure"),
    ("kafka", "technology"),
    ("rabbitmq", "technology"),
    ("nginx", "technology"),
    ("load balancer", "infrastructure"),
    ("monitoring", "process"),
    ("logging", "process"),
    ("error handling", "concept"),
    ("retry", "pattern"),
    ("circuit breaker", "pattern"),
    ("timeout", "concept"),
    ("async", "concept"),
    ("concurrency", "concept"),
    ("parallelism", "concept"),
];

/// Verbs that indicate relationships between entities.
const RELATION_VERBS: &[(&str, &str)] = &[
    ("uses", "USES"),
    ("requires", "REQUIRES"),
    ("depends on", "DEPENDS_ON"),
    ("implements", "IMPLEMENTS"),
    ("extends", "EXTENDS"),
    ("replaces", "REPLACES"),
    ("connects to", "CONNECTS_TO"),
    ("integrates with", "INTEGRATES_WITH"),
    ("supports", "SUPPORTS"),
    ("enables", "ENABLES"),
    ("produces", "PRODUCES"),
    ("consumes", "CONSUMES"),
];

impl SemanticMemoryEngine {
    pub fn new(store: Store) -> Self {
        Self {
            repo: SemanticRepository::new(store),
        }
    }

    /// Extract concepts (entities + relationships + facts) from text.
    pub fn extract_concepts(
        &self,
        text: &str,
        source_episode_id: Option<&str>,
    ) -> Result<ConceptExtraction, AresError> {
        debug!("Extracting concepts from text ({} chars)", text.len());

        let entities = self.extract_entities(text);
        let relationships = self.extract_relationships(text, &entities);
        let facts = self.generate_facts(&entities, &relationships, source_episode_id);

        Ok(ConceptExtraction {
            entities,
            relationships,
            facts,
        })
    }

    /// Extract named entities from text using keyword matching.
    pub fn extract_entities(&self, text: &str) -> Vec<SemanticEntity> {
        let lower = text.to_lowercase();
        let mut entities = Vec::new();

        for (keyword, entity_type) in TECH_KEYWORDS {
            if lower.contains(keyword) {
                // Confidence based on keyword frequency
                let count = lower.matches(keyword).count();
                let confidence = (0.5 + 0.1 * count as f64).min(0.99);

                entities.push(SemanticEntity {
                    name: capitalize_first(keyword),
                    entity_type: entity_type.to_string(),
                    confidence,
                });
            }
        }

        // Deduplicate
        entities.sort_by(|a, b| a.name.cmp(&b.name));
        entities.dedup_by(|a, b| a.name == b.name);
        entities
    }

    /// Extract relationships between entities using verb pattern matching.
    pub fn extract_relationships(
        &self,
        text: &str,
        entities: &[SemanticEntity],
    ) -> Vec<SemanticRelationship> {
        let lower = text.to_lowercase();
        let mut relationships = Vec::new();

        // For each pair of entities, check if there's a relationship verb between them
        for (i, e1) in entities.iter().enumerate() {
            for e2 in entities.iter().skip(i + 1) {
                // Check if both entities appear in a sentence together
                let e1_lower = e1.name.to_lowercase();
                let e2_lower = e2.name.to_lowercase();

                if lower.contains(&e1_lower) && lower.contains(&e2_lower) {
                    // Look for relationship verbs
                    for (verb, rel_type) in RELATION_VERBS {
                        if lower.contains(verb) {
                            let confidence = (e1.confidence * e2.confidence * 0.9).min(0.99);
                            relationships.push(SemanticRelationship {
                                subject: e1.name.clone(),
                                predicate: rel_type.to_string(),
                                object: e2.name.clone(),
                                confidence,
                            });
                            break; // Only one relationship per pair
                        }
                    }

                    // If no verb found but entities co-occur, infer RELATED_TO
                    if !relationships
                        .iter()
                        .any(|r| r.subject == e1.name && r.object == e2.name)
                    {
                        relationships.push(SemanticRelationship {
                            subject: e1.name.clone(),
                            predicate: "RELATED_TO".into(),
                            object: e2.name.clone(),
                            confidence: (e1.confidence * e2.confidence * 0.5).min(0.7),
                        });
                    }
                }
            }
        }

        relationships
    }

    /// Generate semantic facts from extracted entities and relationships.
    pub fn generate_facts(
        &self,
        entities: &[SemanticEntity],
        relationships: &[SemanticRelationship],
        source_episode_id: Option<&str>,
    ) -> Vec<SemanticFact> {
        let mut facts = Vec::new();

        // Generate entity-based facts
        for entity in entities {
            facts.push(SemanticFact {
                statement: format!("{} is a {}", entity.name, entity.entity_type),
                confidence: entity.confidence,
                source_episode_id: source_episode_id.map(String::from),
                entities: vec![entity.name.clone()],
            });
        }

        // Generate relationship-based facts
        for rel in relationships {
            facts.push(SemanticFact {
                statement: format!("{} {} {}", rel.subject, rel.predicate, rel.object),
                confidence: rel.confidence,
                source_episode_id: source_episode_id.map(String::from),
                entities: vec![rel.subject.clone(), rel.object.clone()],
            });
        }

        facts
    }

    /// Store extracted concepts as semantic memories.
    pub fn store_extraction(
        &self,
        extraction: &ConceptExtraction,
        source_episode_id: Option<&str>,
    ) -> Result<Vec<String>, AresError> {
        let now = Utc::now().timestamp_micros();
        let mut ids = Vec::new();

        // Store entities
        for entity in &extraction.entities {
            let id = Uuid::now_v7().to_string();
            let mem = SemanticMemory {
                id: id.clone(),
                source_episode_id: source_episode_id.map(String::from),
                memory_type: SemanticMemoryType::Entity,
                subject: entity.name.clone(),
                predicate: String::new(),
                object: entity.entity_type.clone(),
                confidence: entity.confidence,
                evidence_count: 1,
                tags: vec![entity.entity_type.clone()],
                created_at: now,
                updated_at: now,
            };
            self.repo.insert(&mem)?;
            ids.push(id);
        }

        // Store relationships
        for rel in &extraction.relationships {
            let id = Uuid::now_v7().to_string();
            let mem = SemanticMemory {
                id: id.clone(),
                source_episode_id: source_episode_id.map(String::from),
                memory_type: SemanticMemoryType::Relationship,
                subject: rel.subject.clone(),
                predicate: rel.predicate.clone(),
                object: rel.object.clone(),
                confidence: rel.confidence,
                evidence_count: 1,
                tags: vec![],
                created_at: now,
                updated_at: now,
            };
            self.repo.insert(&mem)?;
            ids.push(id);
        }

        // Store facts
        for fact in &extraction.facts {
            let id = Uuid::now_v7().to_string();
            let mem = SemanticMemory {
                id: id.clone(),
                source_episode_id: source_episode_id.map(String::from),
                memory_type: SemanticMemoryType::Fact,
                subject: fact.statement.clone(),
                predicate: String::new(),
                object: String::new(),
                confidence: fact.confidence,
                evidence_count: 1,
                tags: fact.entities.clone(),
                created_at: now,
                updated_at: now,
            };
            self.repo.insert(&mem)?;
            ids.push(id);
        }

        Ok(ids)
    }

    /// Update confidence for a semantic memory.
    pub fn update_confidence(&self, id: &str, delta: f64) -> Result<(), AresError> {
        let mem = self
            .repo
            .get(id)?
            .ok_or_else(|| AresError::not_found("semantic_memory", id))?;
        let new_confidence = (mem.confidence + delta).clamp(0.0, 1.0);
        let new_count = mem.evidence_count + 1;
        self.repo.update_confidence(id, new_confidence, new_count)
    }

    /// Query semantic memories.
    pub fn query(&self, q: &SemanticQuery) -> Result<Vec<SemanticMemory>, AresError> {
        self.repo.query(q)
    }

    /// Get a semantic memory by ID.
    pub fn get(&self, id: &str) -> Result<Option<SemanticMemory>, AresError> {
        self.repo.get(id)
    }

    /// Count total semantic memories.
    pub fn count(&self) -> Result<u64, AresError> {
        self.repo.count()
    }
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::test_store;

    fn make_engine() -> (SemanticMemoryEngine, tempfile::TempDir) {
        let (store, dir) = test_store();
        (SemanticMemoryEngine::new(store), dir)
    }

    #[test]
    fn extract_entities_from_text() {
        let (engine, _dir) = make_engine();
        let text = "We built an authentication system using JWT tokens with OAuth integration";
        let entities = engine.extract_entities(text);

        let names: Vec<&str> = entities.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"Authentication"));
        assert!(names.contains(&"Jwt"));
        assert!(names.contains(&"Oauth"));
    }

    #[test]
    fn extract_entities_empty_text() {
        let (engine, _dir) = make_engine();
        let entities = engine.extract_entities("");
        assert!(entities.is_empty());
    }

    #[test]
    fn extract_relationships_from_text() {
        let (engine, _dir) = make_engine();
        let text = "Authentication uses JWT for token management";
        let entities = engine.extract_entities(text);
        let relationships = engine.extract_relationships(text, &entities);

        assert!(!relationships.is_empty());
        let has_uses = relationships.iter().any(|r| r.predicate == "USES");
        assert!(has_uses);
    }

    #[test]
    fn extract_concepts_full_pipeline() {
        let (engine, _dir) = make_engine();
        let text = "Deploy the microservice to Kubernetes using Docker containers";
        let extraction = engine.extract_concepts(text, Some("ep_1")).unwrap();

        assert!(!extraction.entities.is_empty());
        assert!(!extraction.facts.is_empty());
    }

    #[test]
    fn generate_facts_from_entities() {
        let (engine, _dir) = make_engine();
        let entities = vec![SemanticEntity {
            name: "Redis".into(),
            entity_type: "technology".into(),
            confidence: 0.9,
        }];
        let facts = engine.generate_facts(&entities, &[], None);
        assert_eq!(facts.len(), 1);
        assert!(facts[0].statement.contains("Redis"));
        assert!(facts[0].statement.contains("technology"));
    }

    #[test]
    fn store_extraction_persists_to_db() {
        let (engine, _dir) = make_engine();
        let extraction = ConceptExtraction {
            entities: vec![SemanticEntity {
                name: "Kubernetes".into(),
                entity_type: "platform".into(),
                confidence: 0.9,
            }],
            relationships: vec![],
            facts: vec![SemanticFact {
                statement: "Kubernetes is a platform".into(),
                confidence: 0.9,
                source_episode_id: None,
                entities: vec!["Kubernetes".into()],
            }],
        };

        let ids = engine.store_extraction(&extraction, Some("ep_1")).unwrap();
        assert_eq!(ids.len(), 2); // 1 entity + 1 fact
        assert_eq!(engine.count().unwrap(), 2);
    }

    #[test]
    fn update_confidence_increases() {
        let (engine, _dir) = make_engine();
        let extraction = ConceptExtraction {
            entities: vec![SemanticEntity {
                name: "Docker".into(),
                entity_type: "platform".into(),
                confidence: 0.7,
            }],
            relationships: vec![],
            facts: vec![],
        };
        let ids = engine.store_extraction(&extraction, None).unwrap();
        let id = &ids[0];

        engine.update_confidence(id, 0.1).unwrap();
        let mem = engine.get(id).unwrap().unwrap();
        assert!((mem.confidence - 0.8).abs() < f64::EPSILON);
        assert_eq!(mem.evidence_count, 2);
    }

    #[test]
    fn update_confidence_clamps_to_bounds() {
        let (engine, _dir) = make_engine();
        let extraction = ConceptExtraction {
            entities: vec![SemanticEntity {
                name: "Redis".into(),
                entity_type: "technology".into(),
                confidence: 0.95,
            }],
            relationships: vec![],
            facts: vec![],
        };
        let ids = engine.store_extraction(&extraction, None).unwrap();
        let id = &ids[0];

        // Try to push above 1.0
        engine.update_confidence(id, 0.5).unwrap();
        let mem = engine.get(id).unwrap().unwrap();
        assert!((mem.confidence - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn update_confidence_nonexistent_fails() {
        let (engine, _dir) = make_engine();
        let result = engine.update_confidence("nonexistent", 0.1);
        assert!(result.is_err());
    }

    #[test]
    fn query_semantic_memories() {
        let (engine, _dir) = make_engine();
        let extraction = ConceptExtraction {
            entities: vec![
                SemanticEntity {
                    name: "API".into(),
                    entity_type: "concept".into(),
                    confidence: 0.85,
                },
                SemanticEntity {
                    name: "REST".into(),
                    entity_type: "protocol".into(),
                    confidence: 0.9,
                },
            ],
            relationships: vec![],
            facts: vec![],
        };
        engine.store_extraction(&extraction, None).unwrap();

        let q = SemanticQuery {
            memory_type: Some(SemanticMemoryType::Entity),
            ..Default::default()
        };
        let results = engine.query(&q).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn capitalize_first_works() {
        assert_eq!(capitalize_first("hello"), "Hello");
        assert_eq!(capitalize_first(""), "");
        assert_eq!(capitalize_first("A"), "A");
    }
}
