use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LifecycleState {
    #[default]
    Active,
    Fresh,
    Stale,
    Decaying,
    Superseded,
    Archived,
}
