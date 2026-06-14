use crate::extractor::{BugExtractor, DecisionExtractor, FeatureExtractor};
use crate::formats::get_parser;
use crate::types::{EntityType, ExtractedEntity, ImportContext};
use ares_core::id::ProjectId;
use ares_core::types::memory::{CreateMemoryInput, ImportanceLevel, MemorySource, MemoryType};
use ares_store::repositories::graph::SqliteGraphRepository;
use ares_store::repositories::memory::SqliteMemoryRepository;
use std::sync::Arc;

pub struct ImportPipeline {
    memory_repo: Arc<SqliteMemoryRepository>,
    _graph_repo: Arc<SqliteGraphRepository>,
}

impl ImportPipeline {
    pub fn new(
        memory_repo: Arc<SqliteMemoryRepository>,
        _graph_repo: Arc<SqliteGraphRepository>,
    ) -> Self {
        Self {
            memory_repo,
            _graph_repo,
        }
    }

    pub async fn process_import(
        &self,
        raw_content: &str,
        context: &ImportContext,
    ) -> Result<usize, anyhow::Error> {
        let parser = get_parser(&context.format);
        let conversation = parser.parse(raw_content)?;

        let mut entities = Vec::new();
        entities.extend(DecisionExtractor::extract(&conversation.messages));
        entities.extend(FeatureExtractor::extract(&conversation.messages));
        entities.extend(BugExtractor::extract(&conversation.messages));

        let project_id = ProjectId::from(context.project_id.as_str());

        let mut count = 0;
        for entity in entities {
            let input = self.create_memory_input_from_entity(&project_id, &entity)?;
            self.memory_repo
                .create(input)
                .map_err(|e| anyhow::anyhow!("DB error: {}", e))?;
            count += 1;
        }

        Ok(count)
    }

    fn create_memory_input_from_entity(
        &self,
        project_id: &ProjectId,
        entity: &ExtractedEntity,
    ) -> Result<CreateMemoryInput, anyhow::Error> {
        let title = match entity.entity_type {
            EntityType::Decision => "Chat Decision",
            EntityType::Feature => "Chat Feature",
            EntityType::Bug => "Chat Bug",
        };

        let memory_type = match entity.entity_type {
            EntityType::Decision => MemoryType::Decision,
            EntityType::Feature => MemoryType::Feature,
            EntityType::Bug => MemoryType::Bug,
        };

        Ok(CreateMemoryInput {
            project_id: project_id.clone(),
            memory_type,
            title: title.to_string(),
            content: serde_json::json!({ "text": entity.content.clone() }),
            confidence: Some(entity.confidence),
            importance: Some(ImportanceLevel::Medium),
            source: Some(MemorySource::Human),
            ai_assisted: Some(false),
        })
    }
}
