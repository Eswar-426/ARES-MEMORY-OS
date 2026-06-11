use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

pub struct Span {
    pub id: String,
    pub name: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
}

pub struct TracingService {
    spans: Mutex<HashMap<String, Span>>,
}

impl Default for TracingService {
    fn default() -> Self {
        Self::new()
    }
}

impl TracingService {
    pub fn new() -> Self {
        Self {
            spans: Mutex::new(HashMap::new()),
        }
    }

    pub fn start_span(&self, span_name: &str) -> String {
        let id = Uuid::now_v7().to_string();
        let span = Span {
            id: id.clone(),
            name: span_name.to_string(),
            start_time: Utc::now(),
            end_time: None,
        };
        self.spans.lock().unwrap().insert(id.clone(), span);
        id
    }

    pub fn end_span(&self, span_id: &str) {
        if let Some(span) = self.spans.lock().unwrap().get_mut(span_id) {
            span.end_time = Some(Utc::now());
        }
    }

    pub fn get_span_duration_ms(&self, span_id: &str) -> Option<i64> {
        let spans = self.spans.lock().unwrap();
        if let Some(span) = spans.get(span_id) {
            if let Some(end) = span.end_time {
                return Some(
                    end.signed_duration_since(span.start_time)
                        .num_milliseconds(),
                );
            }
        }
        None
    }
}
