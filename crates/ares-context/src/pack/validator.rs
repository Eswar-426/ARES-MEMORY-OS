use crate::models::ContextPack;
use std::collections::HashSet;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Duplicate nodes detected: {0}")]
    DuplicateNodes(String),
    #[error("Context size exceeds limits (max: {0}, actual: {1})")]
    SizeExceeded(usize, usize),
    #[error("Confidence score missing or invalid: {0}")]
    InvalidConfidence(f32),
    #[error("Orphan files referenced without corresponding nodes: {0}")]
    OrphanFiles(String),
}

pub struct ContextPackValidator;

impl ContextPackValidator {
    pub fn validate(pack: &ContextPack) -> Result<(), ValidationError> {
        let mut seen_nodes = HashSet::new();
        let mut duplicate_nodes = Vec::new();

        for node in &pack.relevant_nodes {
            if !seen_nodes.insert(node.id.as_str().to_string()) {
                duplicate_nodes.push(node.id.as_str().to_string());
            }
        }

        if !duplicate_nodes.is_empty() {
            return Err(ValidationError::DuplicateNodes(duplicate_nodes.join(", ")));
        }

        if pack.confidence_score <= 0.0 || pack.confidence_score > 1.0 {
            return Err(ValidationError::InvalidConfidence(pack.confidence_score));
        }

        // Additional validations can be added here

        Ok(())
    }
}
