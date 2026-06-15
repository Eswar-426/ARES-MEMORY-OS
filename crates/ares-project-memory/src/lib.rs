//! ares-project-memory — Project Memory Engine for ARES MemoryOS.
//!
//! Automatically builds comprehensive project snapshots by analyzing:
//! - Architecture (monolith, microservices, modular)
//! - Languages & frameworks
//! - Dependencies (Cargo.toml, package.json, requirements.txt, go.mod)
//! - Folder structure
//! - API endpoints
//! - Database schemas
//! - Decisions, features, bugs from the memory store
//! - Recent changes from scan history
//!
//! Produces a portable `ProjectSnapshot` that feeds into the Context Generator.

pub mod analyzer;
pub mod builder;
pub mod snapshot;
pub mod summarizer;
pub mod tracker;
pub mod types;

pub use builder::MemoryBuilder;
pub use snapshot::SnapshotStore;
pub use types::ProjectSnapshot;
