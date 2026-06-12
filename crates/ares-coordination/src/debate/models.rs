use ares_agent_runtime::models::AgentId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique debate identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DebateId(pub Uuid);

impl DebateId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for DebateId {
    fn default() -> Self {
        Self::new()
    }
}

/// Role in a debate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DebateRole {
    Proposer,
    Opponent,
    Judge,
}

/// Type of argument in the debate workflow.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArgumentType {
    Proposal,
    Counterargument,
    Rebuttal,
    Verdict,
}

/// An argument submitted during a debate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Argument {
    pub author: AgentId,
    pub role: DebateRole,
    pub arg_type: ArgumentType,
    pub content: String,
    pub confidence: f64,
    pub submitted_at: i64,
}

impl Argument {
    pub fn new(
        author: AgentId,
        role: DebateRole,
        arg_type: ArgumentType,
        content: impl Into<String>,
    ) -> Self {
        Self {
            author,
            role,
            arg_type,
            content: content.into(),
            confidence: 0.5,
            submitted_at: chrono::Utc::now().timestamp(),
        }
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }
}

/// Outcome of a debate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DebateOutcome {
    ProposerWins,
    OpponentWins,
    Compromise(String),
    Inconclusive,
}

impl DebateOutcome {
    pub fn is_decisive(&self) -> bool {
        matches!(
            self,
            DebateOutcome::ProposerWins | DebateOutcome::OpponentWins
        )
    }
}

/// State of a debate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DebateState {
    AwaitingProposal,
    AwaitingCounterargument,
    AwaitingRebuttal,
    AwaitingVerdict,
    Resolved,
}

/// A structured debate session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Debate {
    pub id: DebateId,
    pub topic: String,
    pub proposer: AgentId,
    pub opponent: AgentId,
    pub judge: AgentId,
    pub arguments: Vec<Argument>,
    pub state: DebateState,
    pub round: u32,
    pub outcome: Option<DebateOutcome>,
    pub created_at: i64,
}

impl Debate {
    pub fn new(
        topic: impl Into<String>,
        proposer: AgentId,
        opponent: AgentId,
        judge: AgentId,
    ) -> Self {
        Self {
            id: DebateId::new(),
            topic: topic.into(),
            proposer,
            opponent,
            judge,
            arguments: Vec::new(),
            state: DebateState::AwaitingProposal,
            round: 1,
            outcome: None,
            created_at: chrono::Utc::now().timestamp(),
        }
    }

    /// Submit an argument to the debate.
    pub fn submit_argument(&mut self, argument: Argument) -> Result<(), String> {
        // Validate role matches expected state
        match (&self.state, &argument.arg_type) {
            (DebateState::AwaitingProposal, ArgumentType::Proposal) => {
                if argument.author != self.proposer {
                    return Err("Only the proposer can submit a proposal".into());
                }
                self.state = DebateState::AwaitingCounterargument;
            }
            (DebateState::AwaitingCounterargument, ArgumentType::Counterargument) => {
                if argument.author != self.opponent {
                    return Err("Only the opponent can submit a counterargument".into());
                }
                self.state = DebateState::AwaitingRebuttal;
            }
            (DebateState::AwaitingRebuttal, ArgumentType::Rebuttal) => {
                if argument.author != self.proposer {
                    return Err("Only the proposer can submit a rebuttal".into());
                }
                self.state = DebateState::AwaitingVerdict;
            }
            (DebateState::AwaitingVerdict, ArgumentType::Verdict) => {
                if argument.author != self.judge {
                    return Err("Only the judge can submit a verdict".into());
                }
                self.state = DebateState::Resolved;
            }
            _ => {
                return Err(format!(
                    "Cannot submit {:?} in state {:?}",
                    argument.arg_type, self.state
                ));
            }
        }

        self.arguments.push(argument);
        Ok(())
    }

    /// Render the verdict based on judge's decision.
    pub fn render_verdict(&mut self, winning_side: DebateOutcome) {
        self.outcome = Some(winning_side);
        self.state = DebateState::Resolved;
    }

    pub fn is_resolved(&self) -> bool {
        self.state == DebateState::Resolved
    }

    pub fn argument_count(&self) -> usize {
        self.arguments.len()
    }

    /// Get the full transcript.
    pub fn transcript(&self) -> Vec<&Argument> {
        self.arguments.iter().collect()
    }
}
