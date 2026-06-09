use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub id: String,
    pub topic: String,
    pub event_type: String,

    pub source: String,

    pub schema_version: u32,
    pub event_version: u32,

    pub correlation_id: String,
    pub causation_id: Option<String>,
    pub trace_id: Option<String>,
    pub partition_key: Option<String>,

    pub timestamp: DateTime<Utc>,

    pub payload: Value,

    pub metadata: HashMap<String, String>,
}

impl EventEnvelope {
    pub fn new(
        id: impl Into<String>,
        topic: impl Into<String>,
        event_type: impl Into<String>,
        payload: Value,
    ) -> Self {
        Self {
            id: id.into(),
            topic: topic.into(),
            event_type: event_type.into(),
            source: "ares".to_string(),
            schema_version: 1,
            event_version: 1,
            correlation_id: uuid::Uuid::new_v4().to_string(),
            causation_id: None,
            trace_id: None,
            partition_key: None,
            timestamp: Utc::now(),
            payload,
            metadata: HashMap::new(),
        }
    }
}
