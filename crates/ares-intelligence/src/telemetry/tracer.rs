use tracing::{span, Level, Span};

pub struct Tracer;

impl Tracer {
    pub fn start_selection_span(task_id: &str) -> Span {
        span!(Level::INFO, "selection", task_id = task_id)
    }

    pub fn start_routing_span(task_id: &str) -> Span {
        span!(Level::INFO, "routing", task_id = task_id)
    }

    pub fn start_provider_span(provider_id: &str, model_id: &str) -> Span {
        span!(
            Level::INFO,
            "provider_execution",
            provider_id = provider_id,
            model_id = model_id
        )
    }

    pub fn start_evaluation_span(task_id: &str) -> Span {
        span!(Level::INFO, "evaluation", task_id = task_id)
    }

    pub fn start_learning_span(model_id: &str) -> Span {
        span!(Level::INFO, "learning", model_id = model_id)
    }
}
