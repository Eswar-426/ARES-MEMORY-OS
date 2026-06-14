//! ARES Benchmark Engine
//! 
//! Measures the performance, cost, latency, and success rate of AI agents 
//! running tasks with varying degrees of context (Baseline, Context Dump, ARES).

pub mod agent;
pub mod report;
pub mod runner;
pub mod scoring;
pub mod tools;

pub use agent::{AgentProvider, AgentType, BenchmarkMetrics};
pub use report::BenchmarkReport;
pub use runner::BenchmarkRunner;
pub use scoring::HybridScorer;
