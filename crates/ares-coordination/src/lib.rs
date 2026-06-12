//! ares-coordination — Multi-Agent Coordination & Autonomous Organization Layer.
//!
//! Transforms ARES from a single autonomous execution engine into a coordinated
//! multi-agent operating system capable of:
//!
//! - Dynamic agent creation and specialist assignment
//! - Agent-to-agent communication via message bus
//! - Shared working memory for collaborative knowledge
//! - Task delegation with split/merge/escalate
//! - Consensus building (majority, weighted, confidence, expert override)
//! - Structured debate (proposer/opponent/judge)
//! - Verification patterns (reason-verify, generate-critique, plan-audit, code-review)
//! - Agent reputation tracking with EMA updates
//! - Swarm coordination (parallel, hierarchical, adaptive)
//! - Conflict resolution (resource contention, contradictory plans, duplicates)
//! - Safety governance (rate limits, depth limits, cost ceilings)
//! - Resource management (pre-delegation resource checks)
//! - Distributed coordination interfaces (single-node now, multi-node ready)
//! - Organizational learning (best teams, pairs, workflows)
//!
//! Core principle: **Deterministic coordination** — all coordination flows
//! produce reproducible results when run single-threaded.

pub mod conflict;
pub mod consensus;
pub mod coordination;
pub mod debate;
pub mod delegation;
pub mod distributed;
pub mod governor;
pub mod messaging;
pub mod organization;
pub mod organizational_learning;
pub mod reputation;
pub mod resource_manager;
pub mod shared_memory;
pub mod swarm;
pub mod telemetry;
pub mod verification;

#[cfg(test)]
mod tests;
