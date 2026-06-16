pub mod models;
pub mod storage;
pub mod history;
pub mod traceability;
pub mod health;
pub mod gaps;
pub mod integration;

// Public API re-exports
pub use models::*;
pub use storage::RequirementStore;
pub use history::{RequirementHistory, RequirementRevision};
pub use traceability::RequirementEdgeProvider;
pub use health::{RequirementHealthEngine, RequirementHealthScore};
pub use gaps::{RequirementGapDetector, RequirementGap};
pub use integration::{RequirementSummary, RequirementCoverage, RequirementHealthSnapshot};
