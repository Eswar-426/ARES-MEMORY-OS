use crate::models::profile::ModelProfile;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait ProfileRepository: Send + Sync {
    async fn save(&self, profile: &ModelProfile) -> anyhow::Result<()>;
    async fn get_by_model_id(&self, model_id: Uuid) -> anyhow::Result<Option<ModelProfile>>;
}
