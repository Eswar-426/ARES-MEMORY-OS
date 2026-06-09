use crate::events::envelope::EventEnvelope;
use std::collections::HashMap;

use tokio::sync::{broadcast, RwLock};

pub struct WsHub {
    channels: RwLock<HashMap<String, broadcast::Sender<EventEnvelope>>>,
}

impl Default for WsHub {
    fn default() -> Self {
        Self::new()
    }
}

impl WsHub {
    pub fn new() -> Self {
        Self {
            channels: RwLock::new(HashMap::new()),
        }
    }

    pub async fn broadcast(&self, topic: &str, event: EventEnvelope) {
        let channels = self.channels.read().await;
        if let Some(tx) = channels.get(topic) {
            let _ = tx.send(event);
        }
    }
}
