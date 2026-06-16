use crate::models::{DecisionMemory, DecisionState, DecisionId};
use crate::api::dtos::{CreateDecisionDto, DecisionResponseDto};
use crate::lifecycle::{LifecycleManager, Validator};
use crate::storage::sqlite::DecisionStorage;
use anyhow::Result;
use chrono::Utc;
use std::sync::Arc;

pub struct DecisionApi {
    storage: Arc<DecisionStorage>,
}

impl DecisionApi {
    pub fn new(storage: Arc<DecisionStorage>) -> Self {
        Self { storage }
    }

    pub fn create_decision(&self, dto: CreateDecisionDto) -> Result<DecisionResponseDto> {
        let decision = DecisionMemory {
            id: DecisionId::new_v4(),
            title: dto.title,
            context: dto.context,
            state: DecisionState::Proposed,
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            confidence: dto.confidence,
            ai_assisted: dto.provenance.source_type != crate::models::SourceType::Human,
            human_reviewed: false,
            review_due_at: None,
            approved_by: vec![],
            tags: dto.tags,
            supersedes: vec![],
            superseded_by: None,
            provenance: dto.provenance,
            reasoning: dto.reasoning,
            impact: dto.impact,
        };

        Validator::validate_for_save(&decision)?;
        
        self.storage.save_decision(&decision)?;

        Ok(DecisionResponseDto {
            id: decision.id,
            title: decision.title,
            state: decision.state,
            version: decision.version,
            created_at: decision.created_at,
        })
    }

    pub fn accept_decision(&self, id: DecisionId) -> Result<DecisionResponseDto> {
        // In a real app, we would fetch from storage first.
        // For demonstration of the integration, we'll assume we fetched it.
        unimplemented!()
    }
}
