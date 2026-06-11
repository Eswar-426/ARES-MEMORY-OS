use crate::models::routing::RoutingDecision;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait RoutingRepository: Send + Sync {
    async fn save(&self, decision: &RoutingDecision) -> anyhow::Result<()>;
    async fn get_by_task_id(&self, task_id: Uuid) -> anyhow::Result<Option<RoutingDecision>>;
}
