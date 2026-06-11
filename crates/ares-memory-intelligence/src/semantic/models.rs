use serde::{Deserialize, Serialize};

/// Type of semantic memory entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SemanticMemoryType {
    Entity,
    Relationship,
    Fact,
    Concept,
}

impl SemanticMemoryType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Entity => "entity",
            Self::Relationship => "relationship",
            Self::Fact => "fact",
            Self::Concept => "concept",
        }
    }

    pub fn from_str_val(s: &str) -> Self {
        match s {
            "entity" => Self::Entity,
            "relationship" => Self::Relationship,
            "fact" => Self::Fact,
            "concept" => Self::Concept,
            _ => Self::Concept,
        }
    }
}

/// A semantic memory — an extracted concept, entity, relationship, or fact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticMemory {
    pub id: String,
    pub source_episode_id: Option<String>,
    pub memory_type: SemanticMemoryType,
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub confidence: f64,
    pub evidence_count: u32,
    pub tags: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// An extracted entity from episode text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticEntity {
    pub name: String,
    pub entity_type: String,
    pub confidence: f64,
}

/// An extracted relationship between entities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticRelationship {
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub confidence: f64,
}

/// A semantic fact with confidence scoring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticFact {
    pub statement: String,
    pub confidence: f64,
    pub source_episode_id: Option<String>,
    pub entities: Vec<String>,
}

/// Result of concept extraction from text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptExtraction {
    pub entities: Vec<SemanticEntity>,
    pub relationships: Vec<SemanticRelationship>,
    pub facts: Vec<SemanticFact>,
}

/// Query for searching semantic memories.
#[derive(Debug, Clone, Default)]
pub struct SemanticQuery {
    pub memory_type: Option<SemanticMemoryType>,
    pub subject: Option<String>,
    pub min_confidence: Option<f64>,
    pub limit: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_memory_type_roundtrip() {
        for t in &[
            SemanticMemoryType::Entity,
            SemanticMemoryType::Relationship,
            SemanticMemoryType::Fact,
            SemanticMemoryType::Concept,
        ] {
            let s = t.as_str();
            let back = SemanticMemoryType::from_str_val(s);
            assert_eq!(&back, t);
        }
    }

    #[test]
    fn semantic_memory_serialization() {
        let mem = SemanticMemory {
            id: "sm_1".into(),
            source_episode_id: Some("ep_1".into()),
            memory_type: SemanticMemoryType::Relationship,
            subject: "Authentication".into(),
            predicate: "USES".into(),
            object: "JWT".into(),
            confidence: 0.92,
            evidence_count: 3,
            tags: vec!["auth".into()],
            created_at: 1000000,
            updated_at: 1000000,
        };
        let json = serde_json::to_string(&mem).unwrap();
        let back: SemanticMemory = serde_json::from_str(&json).unwrap();
        assert_eq!(back.subject, "Authentication");
        assert_eq!(back.predicate, "USES");
        assert!((back.confidence - 0.92).abs() < f64::EPSILON);
    }

    #[test]
    fn semantic_entity_serialization() {
        let entity = SemanticEntity {
            name: "JWT".into(),
            entity_type: "technology".into(),
            confidence: 0.95,
        };
        let json = serde_json::to_string(&entity).unwrap();
        let back: SemanticEntity = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "JWT");
    }

    #[test]
    fn semantic_relationship_serialization() {
        let rel = SemanticRelationship {
            subject: "Auth".into(),
            predicate: "REQUIRES".into(),
            object: "Token Refresh".into(),
            confidence: 0.88,
        };
        let json = serde_json::to_string(&rel).unwrap();
        let back: SemanticRelationship = serde_json::from_str(&json).unwrap();
        assert_eq!(back.predicate, "REQUIRES");
    }

    #[test]
    fn concept_extraction_serialization() {
        let extraction = ConceptExtraction {
            entities: vec![SemanticEntity {
                name: "Kubernetes".into(),
                entity_type: "platform".into(),
                confidence: 0.9,
            }],
            relationships: vec![],
            facts: vec![SemanticFact {
                statement: "Kubernetes manages containers".into(),
                confidence: 0.85,
                source_episode_id: None,
                entities: vec!["Kubernetes".into()],
            }],
        };
        let json = serde_json::to_string(&extraction).unwrap();
        let back: ConceptExtraction = serde_json::from_str(&json).unwrap();
        assert_eq!(back.entities.len(), 1);
        assert_eq!(back.facts.len(), 1);
    }

    #[test]
    fn semantic_query_default() {
        let q = SemanticQuery::default();
        assert!(q.memory_type.is_none());
        assert!(q.subject.is_none());
        assert!(q.min_confidence.is_none());
    }
}
