use serde::{Deserialize, Serialize};
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSchemaWrapper<T> {
    pub schema_version: String,
    pub event_version: String,
    pub event_type: String,
    pub timestamp: String,
    pub data: T,
}

impl<T> EventSchemaWrapper<T> {
    pub fn new(event_type: &str, data: T) -> Self {
        Self {
            schema_version: "1.0".to_string(),
            event_version: "1.0".to_string(),
            event_type: event_type.to_string(),
            timestamp: Utc::now().to_rfc3339(),
            data,
        }
    }
}
