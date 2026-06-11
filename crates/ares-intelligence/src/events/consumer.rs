use super::models::IntelligenceEvent;

pub struct EventConsumer;

impl Default for EventConsumer {
    fn default() -> Self {
        Self::new()
    }
}

impl EventConsumer {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self
    }

    #[allow(dead_code)]
    pub fn handle_event(&self, event: IntelligenceEvent) {
        let _ = event;
        // Placeholder for event processing
    }
}
