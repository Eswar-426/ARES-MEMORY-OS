//! ares-memory-intelligence — Memory Intelligence & Knowledge Evolution Engine.
//!
//! Provides:
//! - Episodic memory for mission experiences
//! - Semantic memory evolution (concept/entity/relationship extraction)
//! - Memory consolidation (merge, cluster, detect patterns)
//! - Knowledge evolution (confidence, contradictions, decay, reinforcement)
//! - Decision intelligence (decision history, reasoning, alternatives)
//! - Experience learning (events → experiences → lessons → principles)
//! - Memory compression (summarization, deduplication, principle extraction)
//! - Retrieval engine (similar missions, failures, successes, lessons)

pub mod agent_knowledge;
pub mod compression;
pub mod consolidation;
pub mod decision_intelligence;
pub mod episodic;
pub mod evolution;
pub mod experience;
pub mod retrieval;
pub mod semantic;
pub mod test_utils;
pub mod assembler;
pub mod facade;

#[cfg(test)]
mod tests;
