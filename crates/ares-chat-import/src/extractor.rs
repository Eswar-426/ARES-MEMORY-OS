use regex::Regex;
use std::sync::OnceLock;

use crate::types::{ChatMessage, ConversationRole, EntityType, ExtractedEntity};

static DECISION_PATTERN: OnceLock<Regex> = OnceLock::new();
static FEATURE_PATTERN: OnceLock<Regex> = OnceLock::new();
static BUG_PATTERN: OnceLock<Regex> = OnceLock::new();

pub struct DecisionExtractor;
pub struct FeatureExtractor;
pub struct BugExtractor;

impl DecisionExtractor {
    pub fn extract(messages: &[ChatMessage]) -> Vec<ExtractedEntity> {
        let pattern = DECISION_PATTERN.get_or_init(|| {
            Regex::new(r"(?i)(we decided to|I decided to|let's go with|going with|we'll use|I'll use|opted for) (.*?)(?:\.|\n|$)").unwrap()
        });

        let mut extracted = Vec::new();

        for msg in messages {
            if msg.role != ConversationRole::User {
                continue; // mostly trust user decisions
            }

            for cap in pattern.captures_iter(&msg.content) {
                if let Some(content_match) = cap.get(2) {
                    extracted.push(ExtractedEntity {
                        entity_type: EntityType::Decision,
                        content: content_match.as_str().trim().to_string(),
                        context: msg.content.clone(),
                        confidence: 0.8, // heuristic
                    });
                }
            }
        }

        extracted
    }
}

impl FeatureExtractor {
    pub fn extract(messages: &[ChatMessage]) -> Vec<ExtractedEntity> {
        let pattern = FEATURE_PATTERN.get_or_init(|| {
            Regex::new(r"(?i)(implement|add feature|build|create) (.*?)(?:\.|\n|$)").unwrap()
        });

        let mut extracted = Vec::new();

        for msg in messages {
            if msg.role != ConversationRole::User {
                continue;
            }

            for cap in pattern.captures_iter(&msg.content) {
                if let Some(content_match) = cap.get(2) {
                    extracted.push(ExtractedEntity {
                        entity_type: EntityType::Feature,
                        content: content_match.as_str().trim().to_string(),
                        context: msg.content.clone(),
                        confidence: 0.7,
                    });
                }
            }
        }

        extracted
    }
}

impl BugExtractor {
    pub fn extract(messages: &[ChatMessage]) -> Vec<ExtractedEntity> {
        let pattern = BUG_PATTERN.get_or_init(|| {
            Regex::new(r"(?i)(bug|fix|broken|error|exception|panic) (.*?)(?:\.|\n|$)").unwrap()
        });

        let mut extracted = Vec::new();

        for msg in messages {
            for cap in pattern.captures_iter(&msg.content) {
                if let Some(content_match) = cap.get(2) {
                    extracted.push(ExtractedEntity {
                        entity_type: EntityType::Bug,
                        content: format!(
                            "{} {}",
                            cap.get(1).unwrap().as_str(),
                            content_match.as_str().trim()
                        ),
                        context: msg.content.clone(),
                        confidence: 0.75,
                    });
                }
            }
        }

        extracted
    }
}
