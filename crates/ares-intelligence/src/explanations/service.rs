use super::models::SelectionExplanation;
use super::repository::ExplanationRepository;
use std::sync::Arc;

pub struct ExplanationService {
    repo: Arc<dyn ExplanationRepository>,
}

impl ExplanationService {
    #[allow(dead_code)]
    pub fn new(repo: Arc<dyn ExplanationRepository>) -> Self {
        Self { repo }
    }

    #[allow(dead_code)]
    pub async fn record_selection(&self, explanation: SelectionExplanation) -> anyhow::Result<()> {
        self.repo.save_explanation(explanation).await
    }
}
