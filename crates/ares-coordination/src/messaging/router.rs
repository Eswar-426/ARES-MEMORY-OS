use ares_agent_runtime::models::AgentId;

use super::bus::MessageBus;
use super::types::{Message, MessageType};
use crate::organization::hierarchy::AgentHierarchy;

/// Routes messages based on type, using organizational hierarchy for escalations.
pub struct MessageRouter {
    bus: MessageBus,
}

impl MessageRouter {
    pub fn new(bus: MessageBus) -> Self {
        Self { bus }
    }

    /// Route a message based on its type.
    pub async fn route(
        &self,
        message: Message,
        hierarchy: Option<&AgentHierarchy>,
    ) -> Result<usize, String> {
        match &message.msg_type {
            MessageType::Direct | MessageType::Request | MessageType::Response(_) => {
                self.bus.send(message).await.map(|_| 1)
            }
            MessageType::Broadcast | MessageType::Notification => self.bus.broadcast(message).await,
            MessageType::TeamMessage(team_id) => {
                if let Some(h) = hierarchy {
                    if let Some(team) = h.get_team(team_id) {
                        let members: Vec<AgentId> = team.members.keys().copied().collect();
                        self.bus.send_to_team(message, &members).await
                    } else {
                        Err(format!("Team {:?} not found in hierarchy", team_id))
                    }
                } else {
                    Err("Hierarchy required for team messaging".into())
                }
            }
            MessageType::HierarchyEscalation => {
                if let Some(h) = hierarchy {
                    if let Some(node) = h.get_node(&message.from) {
                        if let Some(parent_id) = node.parent_id {
                            let escalated = Message {
                                to: Some(parent_id),
                                ..message
                            };
                            self.bus.send(escalated).await.map(|_| 1)
                        } else {
                            Err("Cannot escalate: agent has no parent".into())
                        }
                    } else {
                        Err("Agent not found in hierarchy".into())
                    }
                } else {
                    Err("Hierarchy required for escalation".into())
                }
            }
        }
    }

    /// Get a reference to the underlying bus.
    pub fn bus(&self) -> &MessageBus {
        &self.bus
    }
}
