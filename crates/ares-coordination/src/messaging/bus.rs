use ares_agent_runtime::models::AgentId;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

use super::types::Message;

/// The central message bus for inter-agent communication.
///
/// Uses per-agent mpsc channels for reliable delivery.
/// Thread-safe via Arc<RwLock<...>>.
pub struct MessageBus {
    /// Per-agent inbox: agent_id → sender handle.
    inboxes: Arc<RwLock<HashMap<AgentId, mpsc::Sender<Message>>>>,
    /// Buffer size for each agent's channel.
    channel_buffer: usize,
    /// All messages sent (for audit/replay).
    message_log: Arc<RwLock<Vec<Message>>>,
}

impl MessageBus {
    pub fn new() -> Self {
        Self {
            inboxes: Arc::new(RwLock::new(HashMap::new())),
            channel_buffer: 256,
            message_log: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.channel_buffer = size;
        self
    }

    /// Register an agent and return a receiver for their inbox.
    pub async fn register(&self, agent_id: AgentId) -> mpsc::Receiver<Message> {
        let (tx, rx) = mpsc::channel(self.channel_buffer);
        self.inboxes.write().await.insert(agent_id, tx);
        rx
    }

    /// Unregister an agent, dropping their channel.
    pub async fn unregister(&self, agent_id: &AgentId) {
        self.inboxes.write().await.remove(agent_id);
    }

    /// Send a direct message to a specific agent.
    pub async fn send(&self, message: Message) -> Result<(), String> {
        // Log the message
        self.message_log.write().await.push(message.clone());

        if let Some(to) = &message.to {
            let inboxes = self.inboxes.read().await;
            if let Some(sender) = inboxes.get(to) {
                sender
                    .send(message)
                    .await
                    .map_err(|e| format!("Failed to deliver message: {}", e))
            } else {
                Err(format!("Agent {:?} not registered", to))
            }
        } else {
            Err("Direct message requires a recipient".into())
        }
    }

    /// Broadcast a message to all registered agents (except sender).
    pub async fn broadcast(&self, message: Message) -> Result<usize, String> {
        self.message_log.write().await.push(message.clone());

        let inboxes = self.inboxes.read().await;
        let mut delivered = 0;

        for (agent_id, sender) in inboxes.iter() {
            if *agent_id != message.from && sender.send(message.clone()).await.is_ok() {
                delivered += 1;
            }
        }

        Ok(delivered)
    }

    /// Send a message to all members of a team.
    pub async fn send_to_team(
        &self,
        message: Message,
        team_members: &[AgentId],
    ) -> Result<usize, String> {
        self.message_log.write().await.push(message.clone());

        let inboxes = self.inboxes.read().await;
        let mut delivered = 0;

        for member_id in team_members {
            if *member_id != message.from {
                if let Some(sender) = inboxes.get(member_id) {
                    if sender.send(message.clone()).await.is_ok() {
                        delivered += 1;
                    }
                }
            }
        }

        Ok(delivered)
    }

    /// Get the count of registered agents.
    pub async fn agent_count(&self) -> usize {
        self.inboxes.read().await.len()
    }

    /// Get the total number of messages sent through the bus.
    pub async fn message_count(&self) -> usize {
        self.message_log.read().await.len()
    }

    /// Check if an agent is registered.
    pub async fn is_registered(&self, agent_id: &AgentId) -> bool {
        self.inboxes.read().await.contains_key(agent_id)
    }

    /// Get all messages in the log (for audit/testing).
    pub async fn get_message_log(&self) -> Vec<Message> {
        self.message_log.read().await.clone()
    }

    /// Clear the message log.
    pub async fn clear_log(&self) {
        self.message_log.write().await.clear();
    }
}

impl Default for MessageBus {
    fn default() -> Self {
        Self::new()
    }
}
