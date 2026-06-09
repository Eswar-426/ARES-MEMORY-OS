use super::models::Constraint;
use crate::entities::models::Entity;
use crate::relationships::models::Relationship;
use ares_core::AresError;

pub struct RulesEngine;

impl RulesEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn validate_entity(
        &self,
        _entity: &Entity,
        _constraints: &[Constraint],
    ) -> Result<(), AresError> {
        // Scaffolding for validating entities against graph constraints
        Ok(())
    }

    pub fn validate_relationship(
        &self,
        _relationship: &Relationship,
        _constraints: &[Constraint],
    ) -> Result<(), AresError> {
        // Scaffolding for validating relationships against graph constraints
        Ok(())
    }
}

impl Default for RulesEngine {
    fn default() -> Self {
        Self::new()
    }
}
