//! ares-world-model — World Model, Predictive Planning & Simulation Engine.
//!
//! Transforms ARES from a reactive autonomous system into a predictive
//! autonomous operating system capable of simulating future outcomes
//! before execution.
//!
//! Provides:
//! - World State snapshots (complete reality capture)
//! - Scenario generation (multiple possible futures)
//! - Deterministic simulation (cost, duration, success, risk)
//! - Multi-dimensional risk analysis
//! - Counterfactual reasoning ("what if" analysis)
//! - Outcome prediction from historical data
//! - Strategy ranking with composite scoring
//! - Historical similarity matching
//! - Prediction explainability
//! - Forecast deviation learning (predicted vs actual)
//!
//! Core principle: **No LLM dependency** — all prediction uses historical
//! memory, knowledge graph, planner history, and execution history.

pub mod explain;
pub mod forecast;
pub mod persistence;
pub mod planner_bridge;
pub mod prediction;
pub mod risk;
pub mod scenario;
pub mod simulation;
pub mod state;
pub mod telemetry;

#[cfg(test)]
mod tests;
