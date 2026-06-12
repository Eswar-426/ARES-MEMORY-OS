use ares_agent_runtime::models::AgentId;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::models::{Argument, ArgumentType, Debate, DebateId, DebateOutcome, DebateRole};
use crate::governor::SafetyGovernor;

/// Engine for managing structured debates between agents.
pub struct DebateEngine {
    debates: Arc<RwLock<HashMap<DebateId, Debate>>>,
}

impl DebateEngine {
    pub fn new() -> Self {
        Self {
            debates: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start a new debate.
    pub async fn start_debate(
        &self,
        topic: impl Into<String>,
        proposer: AgentId,
        opponent: AgentId,
        judge: AgentId,
        governor: Option<&SafetyGovernor>,
    ) -> Result<DebateId, String> {
        if let Some(gov) = governor {
            let decision = gov.check_debate(0).await;
            if decision.is_denied() {
                return Err(format!("Governor denied debate: {:?}", decision));
            }
        }

        let debate = Debate::new(topic, proposer, opponent, judge);
        let id = debate.id;
        self.debates.write().await.insert(id, debate);
        Ok(id)
    }

    /// Submit an argument to a debate.
    pub async fn submit_argument(
        &self,
        debate_id: &DebateId,
        argument: Argument,
    ) -> Result<(), String> {
        let mut debates = self.debates.write().await;
        if let Some(debate) = debates.get_mut(debate_id) {
            debate.submit_argument(argument)
        } else {
            Err(format!("Debate {:?} not found", debate_id))
        }
    }

    /// Submit a proposal (convenience method).
    pub async fn submit_proposal(
        &self,
        debate_id: &DebateId,
        proposer: AgentId,
        content: impl Into<String>,
        confidence: f64,
    ) -> Result<(), String> {
        let arg = Argument::new(
            proposer,
            DebateRole::Proposer,
            ArgumentType::Proposal,
            content,
        )
        .with_confidence(confidence);
        self.submit_argument(debate_id, arg).await
    }

    /// Submit a counterargument (convenience method).
    pub async fn submit_counterargument(
        &self,
        debate_id: &DebateId,
        opponent: AgentId,
        content: impl Into<String>,
        confidence: f64,
    ) -> Result<(), String> {
        let arg = Argument::new(
            opponent,
            DebateRole::Opponent,
            ArgumentType::Counterargument,
            content,
        )
        .with_confidence(confidence);
        self.submit_argument(debate_id, arg).await
    }

    /// Submit a rebuttal (convenience method).
    pub async fn submit_rebuttal(
        &self,
        debate_id: &DebateId,
        proposer: AgentId,
        content: impl Into<String>,
        confidence: f64,
    ) -> Result<(), String> {
        let arg = Argument::new(
            proposer,
            DebateRole::Proposer,
            ArgumentType::Rebuttal,
            content,
        )
        .with_confidence(confidence);
        self.submit_argument(debate_id, arg).await
    }

    /// Render the verdict by the judge.
    pub async fn render_verdict(
        &self,
        debate_id: &DebateId,
        judge: AgentId,
        outcome: DebateOutcome,
        rationale: impl Into<String>,
    ) -> Result<DebateOutcome, String> {
        let mut debates = self.debates.write().await;
        if let Some(debate) = debates.get_mut(debate_id) {
            let verdict = Argument::new(judge, DebateRole::Judge, ArgumentType::Verdict, rationale);
            debate.submit_argument(verdict)?;
            debate.render_verdict(outcome.clone());
            Ok(outcome)
        } else {
            Err(format!("Debate {:?} not found", debate_id))
        }
    }

    /// Get the transcript of a debate.
    pub async fn get_transcript(&self, debate_id: &DebateId) -> Result<Vec<Argument>, String> {
        let debates = self.debates.read().await;
        if let Some(debate) = debates.get(debate_id) {
            Ok(debate.arguments.clone())
        } else {
            Err(format!("Debate {:?} not found", debate_id))
        }
    }

    /// Get a debate.
    pub async fn get_debate(&self, debate_id: &DebateId) -> Option<Debate> {
        self.debates.read().await.get(debate_id).cloned()
    }

    /// Get total debate count.
    pub async fn debate_count(&self) -> usize {
        self.debates.read().await.len()
    }
}

impl Default for DebateEngine {
    fn default() -> Self {
        Self::new()
    }
}
