//! ares-store — SQLite persistence layer for ARES MemoryOS.
//!
//! Provides:
//! - Connection pool management (r2d2 + rusqlite)
//! - Embedded schema migrations (refinery)
//! - Repository traits and implementations
//! - Full-text search (FTS5)

pub mod db;
pub mod metrics;
pub mod migrations;
pub mod overview;
pub mod query;
pub mod repositories;

pub use db::Store;
pub use repositories::{
    decision::SqliteDecisionRepository, event::SqliteEventRepository, gaps::GapAlert,
    gaps::GapType, gaps::HealthScore, gaps::SqliteGapRepository, graph::SqliteGraphRepository,
    memory::SqliteMemoryRepository, plan::SqlitePlanRepository, project::SqliteProjectRepository,
    timeline::SqliteTimelineRepository, vector::SqliteVectorRepository,
    workflow::SqliteWorkflowRepository,
};
