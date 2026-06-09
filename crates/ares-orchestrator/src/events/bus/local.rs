use super::r#trait::{EventBus, EventSubscription};
use crate::events::envelope::EventEnvelope;
use crate::events::interceptors::EventInterceptor;
use ares_core::AresError;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

type SubscriberList = Vec<(String, mpsc::Sender<EventEnvelope>)>;

pub struct LocalEventBus {
    subscribers: RwLock<HashMap<String, SubscriberList>>,
    interceptors: Vec<Arc<dyn EventInterceptor>>,
}

impl LocalEventBus {
    pub fn new(interceptors: Vec<Arc<dyn EventInterceptor>>) -> Self {
        Self {
            subscribers: RwLock::new(HashMap::new()),
            interceptors,
        }
    }

    fn matches_topic(pattern: &str, topic: &str) -> bool {
        if pattern == ">" || pattern == "*" {
            return true;
        }
        // Basic wildcard support: if pattern ends with .*, check prefix
        if let Some(prefix) = pattern.strip_suffix(".*") {
            return topic.starts_with(prefix);
        }
        pattern == topic
    }
}

#[async_trait]
impl EventBus for LocalEventBus {
    async fn publish(&self, mut event: EventEnvelope) -> Result<(), AresError> {
        // Run before_publish interceptors
        for interceptor in &self.interceptors {
            interceptor.before_publish(&mut event).await?;
        }

        let subs = self.subscribers.read().await;
        for (pattern, senders) in subs.iter() {
            if Self::matches_topic(pattern, &event.topic) {
                for (_, sender) in senders {
                    // Ignore send errors if subscriber dropped
                    let _ = sender.send(event.clone()).await;
                }
            }
        }

        // Run after_publish interceptors
        for interceptor in &self.interceptors {
            interceptor.after_publish(&event).await?;
        }

        Ok(())
    }

    async fn subscribe(&self, topic: String) -> Result<EventSubscription, AresError> {
        let (tx, rx) = mpsc::channel(100); // Backpressure: max 100 pending events
        let sub_id = uuid::Uuid::new_v4().to_string();

        let mut subs = self.subscribers.write().await;
        subs.entry(topic.clone())
            .or_default()
            .push((sub_id.clone(), tx));

        Ok(EventSubscription {
            id: sub_id,
            topic,
            receiver: rx,
        })
    }

    async fn unsubscribe(&self, subscription_id: String) -> Result<(), AresError> {
        let mut subs = self.subscribers.write().await;
        for senders in subs.values_mut() {
            senders.retain(|(id, _)| id != &subscription_id);
        }
        Ok(())
    }
}
