#![allow(unused_assignments)]
#![allow(clippy::unnecessary_sort_by)]
#![allow(clippy::too_many_arguments)]
pub mod coverage;
pub mod drift;
pub mod evolution;
pub mod gaps;
pub mod health;
pub mod history;
pub mod impact;
pub mod models;
pub mod simulation;
pub mod storage;
pub mod trace_analysis;
pub mod traceability;

// Public API re-exports
pub use coverage::*;
pub use drift::*;
pub use evolution::*;
pub use gaps::*;
pub use health::{RequirementHealthEngine, RequirementHealthScore};
pub use history::{RequirementHistory, RequirementRevision};
pub use impact::*;
pub use models::*;
pub use simulation::*;
pub use storage::RequirementStore;
pub use trace_analysis::*;
pub use traceability::RequirementEdgeProvider;
