use ares_agent_runtime::models::{AgentId, ConversationId};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use super::types::{Message, MessageId};

/// State of a conversation thread.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConversationState {
    Active,
    Paused,
    Resolved,
    Archived,
}

/// A conversation thread between agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: ConversationId,
    pub topic: String,
    pub participants: HashSet<AgentId>,
    pub messages: Vec<Message>,
    pub state: ConversationState,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Conversation {
    pub fn new(topic: impl Into<String>, initiator: AgentId) -> Self {
        let now = chrono::Utc::now().timestamp();
        let mut participants = HashSet::new();
        participants.insert(initiator);

        Self {
            id: ConversationId::new(),
            topic: topic.into(),
            participants,
            messages: Vec::new(),
            state: ConversationState::Active,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_participant(&mut self, agent_id: AgentId) {
        self.participants.insert(agent_id);
    }

    pub fn remove_participant(&mut self, agent_id: &AgentId) {
        self.participants.remove(agent_id);
    }

    pub fn add_message(&mut self, message: Message) {
        self.participants.insert(message.from);
        if let Some(to) = message.to {
            self.participants.insert(to);
        }
        self.updated_at = chrono::Utc::now().timestamp();
        self.messages.push(message);
    }

    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    pub fn participant_count(&self) -> usize {
        self.participants.len()
    }

    pub fn has_participant(&self, agent_id: &AgentId) -> bool {
        self.participants.contains(agent_id)
    }

    pub fn get_messages_from(&self, agent_id: &AgentId) -> Vec<&Message> {
        self.messages
            .iter()
            .filter(|m| m.from == *agent_id)
            .collect()
    }

    pub fn get_last_message(&self) -> Option<&Message> {
        self.messages.last()
    }

    pub fn get_message_by_id(&self, id: &MessageId) -> Option<&Message> {
        self.messages.iter().find(|m| m.id == *id)
    }

    pub fn resolve(&mut self) {
        self.state = ConversationState::Resolved;
        self.updated_at = chrono::Utc::now().timestamp();
    }

    pub fn pause(&mut self) {
        self.state = ConversationState::Paused;
        self.updated_at = chrono::Utc::now().timestamp();
    }

    pub fn archive(&mut self) {
        self.state = ConversationState::Archived;
        self.updated_at = chrono::Utc::now().timestamp();
    }

    pub fn is_active(&self) -> bool {
        self.state == ConversationState::Active
    }
}

/// Manages multiple conversations.
pub struct ConversationManager {
    conversations: std::collections::HashMap<ConversationId, Conversation>,
}

impl Default for ConversationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ConversationManager {
    pub fn new() -> Self {
        Self {
            conversations: std::collections::HashMap::new(),
        }
    }

    pub fn create(&mut self, topic: impl Into<String>, initiator: AgentId) -> ConversationId {
        let conv = Conversation::new(topic, initiator);
        let id = conv.id;
        self.conversations.insert(id, conv);
        id
    }

    pub fn get(&self, id: &ConversationId) -> Option<&Conversation> {
        self.conversations.get(id)
    }

    pub fn get_mut(&mut self, id: &ConversationId) -> Option<&mut Conversation> {
        self.conversations.get_mut(id)
    }

    pub fn add_message(
        &mut self,
        conv_id: &ConversationId,
        message: Message,
    ) -> Result<(), String> {
        if let Some(conv) = self.conversations.get_mut(conv_id) {
            conv.add_message(message);
            Ok(())
        } else {
            Err(format!("Conversation {:?} not found", conv_id))
        }
    }

    pub fn get_active_conversations(&self) -> Vec<&Conversation> {
        self.conversations
            .values()
            .filter(|c| c.is_active())
            .collect()
    }

    pub fn get_conversations_for_agent(&self, agent_id: &AgentId) -> Vec<&Conversation> {
        self.conversations
            .values()
            .filter(|c| c.has_participant(agent_id))
            .collect()
    }

    pub fn conversation_count(&self) -> usize {
        self.conversations.len()
    }
}
