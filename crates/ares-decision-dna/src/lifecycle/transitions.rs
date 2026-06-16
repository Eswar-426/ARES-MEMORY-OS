use crate::models::{DecisionMemory, DecisionState, DecisionId};
use anyhow::{bail, Result};
use chrono::Utc;

pub struct LifecycleManager;

impl LifecycleManager {
    pub fn propose(mut decision: DecisionMemory) -> Result<DecisionMemory> {
        if decision.state != DecisionState::Proposed {
            bail!("Decision must start in Proposed state");
        }
        decision.updated_at = Utc::now();
        Ok(decision)
    }

    pub fn accept(mut decision: DecisionMemory) -> Result<DecisionMemory> {
        match decision.state {
            DecisionState::Proposed => {
                decision.state = DecisionState::Accepted;
                decision.updated_at = Utc::now();
                decision.version += 1;
                Ok(decision)
            }
            _ => bail!("Can only accept a Proposed decision"),
        }
    }

    pub fn reject(mut decision: DecisionMemory) -> Result<DecisionMemory> {
        match decision.state {
            DecisionState::Proposed => {
                decision.state = DecisionState::Rejected;
                decision.updated_at = Utc::now();
                decision.version += 1;
                Ok(decision)
            }
            _ => bail!("Can only reject a Proposed decision"),
        }
    }

    pub fn supersede(mut old_decision: DecisionMemory, new_decision_id: DecisionId) -> Result<DecisionMemory> {
        match old_decision.state {
            DecisionState::Accepted | DecisionState::Proposed => {
                old_decision.state = DecisionState::Superseded;
                old_decision.superseded_by = Some(new_decision_id);
                old_decision.updated_at = Utc::now();
                old_decision.version += 1;
                Ok(old_decision)
            }
            _ => bail!("Cannot supersede a decision in state: {:?}", old_decision.state),
        }
    }

    pub fn deprecate(mut decision: DecisionMemory) -> Result<DecisionMemory> {
        match decision.state {
            DecisionState::Accepted => {
                decision.state = DecisionState::Deprecated;
                decision.updated_at = Utc::now();
                decision.version += 1;
                Ok(decision)
            }
            _ => bail!("Can only deprecate an Accepted decision"),
        }
    }
}
