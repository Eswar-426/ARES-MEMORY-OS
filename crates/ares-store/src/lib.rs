//! ares-store — SQLite persistence layer for ARES MemoryOS.
//!
//! Provides:
//! - Connection pool management (r2d2 + rusqlite)
//! - Embedded schema migrations (refinery)
//! - Repository traits and implementations
//! - Full-text search (FTS5)

pub mod db;
pub mod migrations;
pub mod query;
pub mod repositories;

pub use db::Store;
pub use repositories::{
    decision::SqliteDecisionRepository, event::SqliteEventRepository, graph::SqliteGraphRepository,
    memory::SqliteMemoryRepository, project::SqliteProjectRepository,
    timeline::SqliteTimelineRepository,
    vector::SqliteVectorRepository, workflow::SqliteWorkflowRepository,
};
