use crate::models::model::Model;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait ModelRepository: Send + Sync {
    async fn save(&self, model: &Model) -> anyhow::Result<()>;
    async fn get_by_id(&self, id: Uuid) -> anyhow::Result<Option<Model>>;
}
