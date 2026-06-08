// Services module — implemented Week 4-8
pub mod context_builder;
pub mod context_pipeline;
pub mod context_service;
pub mod contradiction_detector;
pub mod decision_intelligence;
pub mod decision_service;
pub mod graph_service;
pub mod hybrid_ranking;
pub mod memory_ranking;
pub mod memory_service;
pub mod retrieval;
pub mod scanner_service;
pub mod semantic_retrieval;

#[cfg(test)]
mod context_builder_tests;
#[cfg(test)]
mod contradiction_tests;
#[cfg(test)]
mod decision_intelligence_tests;
#[cfg(test)]
mod memory_ranking_tests;
#[cfg(test)]
mod performance_tests;
#[cfg(test)]
mod retrieval_tests;
