use crate::models::GovernanceEvent;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct GovernanceEventPublisher {
    sender: broadcast::Sender<GovernanceEvent>,
}

impl GovernanceEventPublisher {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    pub fn publish(&self, event: GovernanceEvent) {
        // It's okay if there are no receivers currently
        let _ = self.sender.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<GovernanceEvent> {
        self.sender.subscribe()
    }
}
