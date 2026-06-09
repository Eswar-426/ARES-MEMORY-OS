use crate::events::envelope::EventEnvelope;
use ares_core::AresError;

pub struct SchemaValidator;

impl SchemaValidator {
    pub fn validate(event: &EventEnvelope) -> Result<(), AresError> {
        // In a real implementation, this would fetch the JSON schema
        // for `event.event_type` and validate `event.payload` against it.
        // For now, it's a stub allowing everything.
        if event.schema_version == 0 {
            return Err(AresError::validation("Schema version cannot be 0".to_string()));
        }
        Ok(())
    }
}
