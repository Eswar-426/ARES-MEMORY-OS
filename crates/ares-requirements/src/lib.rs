pub mod models;
pub mod storage;
pub mod history;
pub mod traceability;
pub mod health;
pub mod coverage;
pub mod drift;
pub mod impact;

// Public API re-exports
pub use models::*;
pub use storage::RequirementStore;
pub use history::{RequirementHistory, RequirementRevision};
pub use traceability::RequirementEdgeProvider;
pub use health::{RequirementHealthEngine, RequirementHealthScore};
pub use coverage::*;
pub use drift::*;
pub use impact::*;
