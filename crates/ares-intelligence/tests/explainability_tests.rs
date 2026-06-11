use ares_intelligence::explanations::models::{RejectedModel, SelectionExplanation};
use ares_intelligence::explanations::repository::ExplanationRepository;
use ares_intelligence::explanations::service::ExplanationService;
use ares_intelligence::models::capability::ModelCapability;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

struct InMemoryExplanationRepo {
    selections: RwLock<HashMap<String, SelectionExplanation>>,
}

impl InMemoryExplanationRepo {
    fn new() -> Self {
        Self {
            selections: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl ExplanationRepository for InMemoryExplanationRepo {
    async fn save_explanation(&self, explanation: SelectionExplanation) -> anyhow::Result<()> {
        let mut map = self.selections.write().await;
        map.insert(explanation.task_id.clone(), explanation);
        Ok(())
    }

    async fn get_explanation(&self, task_id: &str) -> anyhow::Result<Option<SelectionExplanation>> {
        let map = self.selections.read().await;
        Ok(map.get(task_id).cloned())
    }
}

#[tokio::test]
async fn test_record_selection_explanation() {
    let repo = Arc::new(InMemoryExplanationRepo::new());
    let service = ExplanationService::new(repo.clone());

    let explanation = SelectionExplanation {
        task_id: "task_abc_123".to_string(),
        selected_model_id: "model_winner_1".to_string(),
        required_capabilities: vec![ModelCapability::Coding, ModelCapability::Reasoning],
        reasoning: "Selected based on highest quality score".to_string(),
        rejected_models: vec![
            RejectedModel {
                model_id: "model_loser_1".to_string(),
                reason: "Insufficient capabilities: Missing Coding".to_string(),
            },
            RejectedModel {
                model_id: "model_loser_2".to_string(),
                reason: "Cost exceeds maximum budget".to_string(),
            },
        ],
    };

    service.record_selection(explanation.clone()).await.unwrap();

    let fetched = repo.get_explanation("task_abc_123").await.unwrap().unwrap();
    assert_eq!(fetched.selected_model_id, "model_winner_1");
    assert_eq!(fetched.rejected_models.len(), 2);
    assert_eq!(fetched.rejected_models[0].model_id, "model_loser_1");
    assert_eq!(
        fetched.rejected_models[1].reason,
        "Cost exceeds maximum budget"
    );
}

#[tokio::test]
async fn test_record_multiple_selection_explanations() {
    let repo = Arc::new(InMemoryExplanationRepo::new());
    let service = ExplanationService::new(repo.clone());

    for i in 0..10 {
        let explanation = SelectionExplanation {
            task_id: format!("task_{}", i),
            selected_model_id: format!("model_winner_{}", i),
            required_capabilities: vec![],
            reasoning: "Balanced selection".to_string(),
            rejected_models: vec![],
        };
        service.record_selection(explanation).await.unwrap();
    }

    for i in 0..10 {
        let task_id = format!("task_{}", i);
        let fetched = repo.get_explanation(&task_id).await.unwrap().unwrap();
        assert_eq!(fetched.selected_model_id, format!("model_winner_{}", i));
    }
}
