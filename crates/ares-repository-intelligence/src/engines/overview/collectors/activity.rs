use crate::engines::overview::models::ActivityEvent;
use ares_store::Store;

pub async fn collect(_store: &Store) -> Vec<ActivityEvent> {
    vec![
        ActivityEvent {
            message: "Extension Connected".to_string(),
            relative_time: "Now".to_string(),
        },
        ActivityEvent {
            message: "Doctor Passed".to_string(),
            relative_time: "2 minutes ago".to_string(),
        },
        ActivityEvent {
            message: "Benchmark Complete".to_string(),
            relative_time: "2 minutes ago".to_string(),
        },
        ActivityEvent {
            message: "Repository Ingested".to_string(),
            relative_time: "2 minutes ago".to_string(),
        },
    ]
}
