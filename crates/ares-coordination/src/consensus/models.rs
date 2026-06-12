use ares_agent_runtime::models::AgentId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a consensus round.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConsensusRoundId(pub Uuid);

impl ConsensusRoundId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for ConsensusRoundId {
    fn default() -> Self {
        Self::new()
    }
}

/// A vote cast by an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub voter: AgentId,
    pub choice: String,
    pub confidence: f64,
    pub weight: f64,
    pub rationale: String,
}

impl Vote {
    pub fn new(voter: AgentId, choice: impl Into<String>, confidence: f64) -> Self {
        Self {
            voter,
            choice: choice.into(),
            confidence: confidence.clamp(0.0, 1.0),
            weight: 1.0,
            rationale: String::new(),
        }
    }

    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = weight;
        self
    }

    pub fn with_rationale(mut self, rationale: impl Into<String>) -> Self {
        self.rationale = rationale.into();
        self
    }
}

/// State of a consensus round.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoundState {
    Open,
    Closed,
    Resolved,
}

/// A consensus round tracking votes and resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusRound {
    pub id: ConsensusRoundId,
    pub topic: String,
    pub options: Vec<String>,
    pub participants: Vec<AgentId>,
    pub votes: Vec<Vote>,
    pub state: RoundState,
    pub round_number: u32,
    pub created_at: i64,
}

impl ConsensusRound {
    pub fn new(topic: impl Into<String>, options: Vec<String>, participants: Vec<AgentId>) -> Self {
        Self {
            id: ConsensusRoundId::new(),
            topic: topic.into(),
            options,
            participants,
            votes: Vec::new(),
            state: RoundState::Open,
            round_number: 1,
            created_at: chrono::Utc::now().timestamp(),
        }
    }

    pub fn cast_vote(&mut self, vote: Vote) -> Result<(), String> {
        if self.state != RoundState::Open {
            return Err("Round is not open for voting".into());
        }
        if !self.participants.contains(&vote.voter) {
            return Err("Voter is not a participant".into());
        }
        if self.votes.iter().any(|v| v.voter == vote.voter) {
            return Err("Agent has already voted".into());
        }
        if !self.options.contains(&vote.choice) {
            return Err(format!("Choice '{}' not in options", vote.choice));
        }
        self.votes.push(vote);
        Ok(())
    }

    pub fn all_voted(&self) -> bool {
        self.votes.len() >= self.participants.len()
    }

    pub fn close(&mut self) {
        self.state = RoundState::Closed;
    }

    pub fn vote_count(&self) -> usize {
        self.votes.len()
    }
}

/// Result of a consensus round.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConsensusResult {
    /// Agreement was reached.
    Agreed(String),
    /// No clear winner — deadlocked.
    Deadlock,
    /// Needs escalation to a higher authority.
    NeedsEscalation(String),
}

impl ConsensusResult {
    pub fn is_agreed(&self) -> bool {
        matches!(self, ConsensusResult::Agreed(_))
    }

    pub fn is_deadlock(&self) -> bool {
        matches!(self, ConsensusResult::Deadlock)
    }
}
