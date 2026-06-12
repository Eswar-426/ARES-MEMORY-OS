use ares_agent_runtime::models::AgentId;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::algorithms::ConsensusAlgorithm;
use super::models::{ConsensusResult, ConsensusRound, ConsensusRoundId, RoundState, Vote};
use crate::governor::SafetyGovernor;

/// Engine for running consensus rounds among agents.
pub struct ConsensusEngine {
    rounds: Arc<RwLock<HashMap<ConsensusRoundId, ConsensusRound>>>,
    results: Arc<RwLock<HashMap<ConsensusRoundId, ConsensusResult>>>,
}

impl ConsensusEngine {
    pub fn new() -> Self {
        Self {
            rounds: Arc::new(RwLock::new(HashMap::new())),
            results: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start a new consensus round.
    pub async fn propose(
        &self,
        topic: impl Into<String>,
        options: Vec<String>,
        participants: Vec<AgentId>,
        governor: Option<&SafetyGovernor>,
    ) -> Result<ConsensusRoundId, String> {
        if let Some(gov) = governor {
            let decision = gov.check_consensus(0).await;
            if decision.is_denied() {
                return Err(format!("Governor denied consensus: {:?}", decision));
            }
        }

        let round = ConsensusRound::new(topic, options, participants);
        let id = round.id;
        self.rounds.write().await.insert(id, round);
        Ok(id)
    }

    /// Cast a vote in a round.
    pub async fn cast_vote(&self, round_id: &ConsensusRoundId, vote: Vote) -> Result<(), String> {
        let mut rounds = self.rounds.write().await;
        if let Some(round) = rounds.get_mut(round_id) {
            round.cast_vote(vote)
        } else {
            Err(format!("Round {:?} not found", round_id))
        }
    }

    /// Resolve a round using the specified algorithm.
    pub async fn resolve(
        &self,
        round_id: &ConsensusRoundId,
        algorithm: &ConsensusAlgorithm,
    ) -> Result<ConsensusResult, String> {
        let mut rounds = self.rounds.write().await;
        if let Some(round) = rounds.get_mut(round_id) {
            round.close();
            let result = algorithm.resolve(round);
            round.state = RoundState::Resolved;
            self.results.write().await.insert(*round_id, result.clone());
            Ok(result)
        } else {
            Err(format!("Round {:?} not found", round_id))
        }
    }

    /// Get the result of a resolved round.
    pub async fn get_result(&self, round_id: &ConsensusRoundId) -> Option<ConsensusResult> {
        self.results.read().await.get(round_id).cloned()
    }

    /// Get a round.
    pub async fn get_round(&self, round_id: &ConsensusRoundId) -> Option<ConsensusRound> {
        self.rounds.read().await.get(round_id).cloned()
    }

    /// Get total rounds created.
    pub async fn round_count(&self) -> usize {
        self.rounds.read().await.len()
    }
}

impl Default for ConsensusEngine {
    fn default() -> Self {
        Self::new()
    }
}
