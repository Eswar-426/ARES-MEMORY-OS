use ares_agent_runtime::models::{AgentId, ConversationId, MissionId, TeamId};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageId(pub Uuid);

impl MessageId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for MessageId {
    fn default() -> Self {
        Self::new()
    }
}

/// Priority levels for messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Normal,
    High,
    Critical,
}

/// Types of messages that can be sent between agents.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    /// Point-to-point message to a specific agent.
    Direct,
    /// Message to all agents in the system.
    Broadcast,
    /// Message to all members of a team.
    TeamMessage(TeamId),
    /// Escalation up the hierarchy to a parent.
    HierarchyEscalation,
    /// Request expecting a response.
    Request,
    /// Response to a prior request.
    Response(MessageId),
    /// Informational notification, no response expected.
    Notification,
}

/// A message exchanged between agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: MessageId,
    pub from: AgentId,
    pub to: Option<AgentId>,
    pub conversation_id: Option<ConversationId>,
    pub mission_id: Option<MissionId>,
    pub msg_type: MessageType,
    pub priority: Priority,
    pub subject: String,
    pub payload: String,
    pub timestamp: i64,
    pub ttl_secs: Option<u64>,
    pub ack_required: bool,
}

impl Message {
    pub fn direct(
        from: AgentId,
        to: AgentId,
        subject: impl Into<String>,
        payload: impl Into<String>,
    ) -> Self {
        Self {
            id: MessageId::new(),
            from,
            to: Some(to),
            conversation_id: None,
            mission_id: None,
            msg_type: MessageType::Direct,
            priority: Priority::Normal,
            subject: subject.into(),
            payload: payload.into(),
            timestamp: chrono::Utc::now().timestamp(),
            ttl_secs: None,
            ack_required: false,
        }
    }

    pub fn broadcast(
        from: AgentId,
        subject: impl Into<String>,
        payload: impl Into<String>,
    ) -> Self {
        Self {
            id: MessageId::new(),
            from,
            to: None,
            conversation_id: None,
            mission_id: None,
            msg_type: MessageType::Broadcast,
            priority: Priority::Normal,
            subject: subject.into(),
            payload: payload.into(),
            timestamp: chrono::Utc::now().timestamp(),
            ttl_secs: None,
            ack_required: false,
        }
    }

    pub fn team(
        from: AgentId,
        team_id: TeamId,
        subject: impl Into<String>,
        payload: impl Into<String>,
    ) -> Self {
        Self {
            id: MessageId::new(),
            from,
            to: None,
            conversation_id: None,
            mission_id: None,
            msg_type: MessageType::TeamMessage(team_id),
            priority: Priority::Normal,
            subject: subject.into(),
            payload: payload.into(),
            timestamp: chrono::Utc::now().timestamp(),
            ttl_secs: None,
            ack_required: false,
        }
    }

    pub fn request(
        from: AgentId,
        to: AgentId,
        subject: impl Into<String>,
        payload: impl Into<String>,
    ) -> Self {
        Self {
            id: MessageId::new(),
            from,
            to: Some(to),
            conversation_id: None,
            mission_id: None,
            msg_type: MessageType::Request,
            priority: Priority::Normal,
            subject: subject.into(),
            payload: payload.into(),
            timestamp: chrono::Utc::now().timestamp(),
            ttl_secs: None,
            ack_required: true,
        }
    }

    pub fn response(
        from: AgentId,
        to: AgentId,
        reply_to: MessageId,
        payload: impl Into<String>,
    ) -> Self {
        Self {
            id: MessageId::new(),
            from,
            to: Some(to),
            conversation_id: None,
            mission_id: None,
            msg_type: MessageType::Response(reply_to),
            priority: Priority::Normal,
            subject: String::new(),
            payload: payload.into(),
            timestamp: chrono::Utc::now().timestamp(),
            ttl_secs: None,
            ack_required: false,
        }
    }

    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_conversation(mut self, conversation_id: ConversationId) -> Self {
        self.conversation_id = Some(conversation_id);
        self
    }

    pub fn with_mission(mut self, mission_id: MissionId) -> Self {
        self.mission_id = Some(mission_id);
        self
    }

    pub fn with_ttl(mut self, secs: u64) -> Self {
        self.ttl_secs = Some(secs);
        self
    }

    pub fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl_secs {
            let now = chrono::Utc::now().timestamp();
            (now - self.timestamp) as u64 >= ttl
        } else {
            false
        }
    }

    pub fn is_broadcast(&self) -> bool {
        matches!(self.msg_type, MessageType::Broadcast)
    }

    pub fn is_team_message(&self) -> bool {
        matches!(self.msg_type, MessageType::TeamMessage(_))
    }
}
