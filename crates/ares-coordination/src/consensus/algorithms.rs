use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::models::{ConsensusResult, ConsensusRound, Vote};

/// Consensus algorithm to use for resolving votes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsensusAlgorithm {
    /// Simple majority (>50% of votes).
    MajorityVote,
    /// Votes weighted by reputation score.
    WeightedVote,
    /// Aggregated by confidence level.
    ConfidenceVote,
    /// Expert can override if confidence threshold met.
    ExpertOverride { expert_confidence_threshold: u32 }, // stored as percentage 0-100
}

impl ConsensusAlgorithm {
    /// Resolve a round using this algorithm.
    pub fn resolve(&self, round: &ConsensusRound) -> ConsensusResult {
        if round.votes.is_empty() {
            return ConsensusResult::Deadlock;
        }

        match self {
            ConsensusAlgorithm::MajorityVote => {
                self.majority_vote(&round.votes, round.participants.len())
            }
            ConsensusAlgorithm::WeightedVote => self.weighted_vote(&round.votes),
            ConsensusAlgorithm::ConfidenceVote => self.confidence_vote(&round.votes),
            ConsensusAlgorithm::ExpertOverride {
                expert_confidence_threshold,
            } => self.expert_override(&round.votes, *expert_confidence_threshold),
        }
    }

    fn majority_vote(&self, votes: &[Vote], participant_count: usize) -> ConsensusResult {
        let mut tallies: HashMap<&str, usize> = HashMap::new();
        for vote in votes {
            *tallies.entry(&vote.choice).or_insert(0) += 1;
        }

        let majority_threshold = participant_count / 2 + 1;
        if let Some((choice, count)) = tallies.iter().max_by_key(|(_, c)| **c) {
            if *count >= majority_threshold {
                ConsensusResult::Agreed(choice.to_string())
            } else {
                ConsensusResult::Deadlock
            }
        } else {
            ConsensusResult::Deadlock
        }
    }

    fn weighted_vote(&self, votes: &[Vote]) -> ConsensusResult {
        let mut weighted_tallies: HashMap<&str, f64> = HashMap::new();
        for vote in votes {
            *weighted_tallies.entry(&vote.choice).or_insert(0.0) += vote.weight;
        }

        let total_weight: f64 = votes.iter().map(|v| v.weight).sum();
        if let Some((choice, weight)) = weighted_tallies
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
        {
            if *weight > total_weight / 2.0 {
                ConsensusResult::Agreed(choice.to_string())
            } else {
                ConsensusResult::Deadlock
            }
        } else {
            ConsensusResult::Deadlock
        }
    }

    fn confidence_vote(&self, votes: &[Vote]) -> ConsensusResult {
        let mut confidence_tallies: HashMap<&str, f64> = HashMap::new();
        for vote in votes {
            *confidence_tallies.entry(&vote.choice).or_insert(0.0) += vote.confidence;
        }

        let total_confidence: f64 = votes.iter().map(|v| v.confidence).sum();
        if let Some((choice, conf)) = confidence_tallies
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
        {
            if *conf > total_confidence * 0.6 {
                ConsensusResult::Agreed(choice.to_string())
            } else {
                ConsensusResult::NeedsEscalation(format!(
                    "No choice reached 60% confidence threshold; top: {} ({:.2}%)",
                    choice,
                    conf / total_confidence * 100.0
                ))
            }
        } else {
            ConsensusResult::Deadlock
        }
    }

    fn expert_override(&self, votes: &[Vote], threshold: u32) -> ConsensusResult {
        let threshold_f = threshold as f64 / 100.0;

        // Check if any expert (high-weight voter) has sufficient confidence
        if let Some(expert_vote) = votes
            .iter()
            .find(|v| v.confidence >= threshold_f && v.weight >= 2.0)
        {
            return ConsensusResult::Agreed(expert_vote.choice.clone());
        }

        // Fall back to weighted vote
        self.weighted_vote(votes)
    }
}
