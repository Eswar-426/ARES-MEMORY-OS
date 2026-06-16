use crate::models::{DecisionMemory, DecisionState};
use anyhow::{bail, Result};

pub struct Validator;

impl Validator {
    pub fn validate_for_save(decision: &DecisionMemory) -> Result<()> {
        if decision.title.trim().is_empty() {
            bail!("Decision title cannot be empty");
        }
        if decision.context.trim().is_empty() {
            bail!("Decision context cannot be empty");
        }
        if !(0.0..=1.0).contains(&decision.confidence) {
            bail!("Confidence must be between 0.0 and 1.0");
        }

        match decision.state {
            DecisionState::Superseded => {
                if decision.superseded_by.is_none() {
                    bail!("Superseded decision must have a superseded_by ID");
                }
            }
            _ => {
                if decision.superseded_by.is_some() {
                    bail!("Only Superseded decisions can have superseded_by set");
                }
            }
        }

        Ok(())
    }
}
